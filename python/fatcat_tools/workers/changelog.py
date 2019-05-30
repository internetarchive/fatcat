
import json
import time
from pykafka.common import OffsetType

from .worker_common import FatcatWorker, most_recent_message


class ChangelogWorker(FatcatWorker):
    """
    Periodically polls the fatcat API looking for new changelogs. When they are
    found, fetch them and push (as JSON) into a Kafka topic.
    """

    def __init__(self, api, kafka_hosts, produce_topic, poll_interval=10.0, offset=None):
        # TODO: should be offset=0
        super().__init__(kafka_hosts=kafka_hosts,
                         produce_topic=produce_topic,
                         api=api)
        self.poll_interval = poll_interval
        self.offset = offset    # the fatcat changelog offset, not the kafka offset

    def run(self):
        topic = self.kafka.topics[self.produce_topic]
        # On start, try to consume the most recent from the topic, and using
        # that as the starting offset. Note that this is a single-partition
        # topic
        if self.offset is None:
            print("Checking for most recent changelog offset...")
            msg = most_recent_message(topic)
            if msg:
                self.offset = json.loads(msg.decode('utf-8'))['index']
            else:
                self.offset = 1

        with topic.get_producer(
                max_request_size=self.produce_max_request_size,
                ) as producer:
            while True:
                latest = int(self.api.get_changelog(limit=1)[0].index)
                if latest > self.offset:
                    print("Fetching changelogs from {} through {}".format(
                        self.offset+1, latest))
                for i in range(self.offset+1, latest+1):
                    cle = self.api.get_changelog_entry(i)
                    obj = self.api.api_client.sanitize_for_serialization(cle)
                    producer.produce(
                        message=json.dumps(obj).encode('utf-8'),
                        partition_key=None,
                        timestamp=None,
                        #NOTE could be (???): timestamp=cle.timestamp,
                    )
                    self.offset = i
                print("Sleeping {} seconds...".format(self.poll_interval))
                time.sleep(self.poll_interval)


class EntityUpdatesWorker(FatcatWorker):
    """
    Consumes from the changelog topic and publishes expanded entities (fetched
    from API) to update topics.

    For now, only release updates are published.
    """

    def __init__(self, api, kafka_hosts, consume_topic, release_topic, file_topic, container_topic):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic,
                         api=api)
        self.release_topic = release_topic
        self.file_topic = file_topic
        self.container_topic = container_topic
        self.consumer_group = "entity-updates"

    def run(self):
        changelog_topic = self.kafka.topics[self.consume_topic]
        release_topic = self.kafka.topics[self.release_topic]
        file_topic = self.kafka.topics[self.file_topic]
        container_topic = self.kafka.topics[self.container_topic]

        consumer = changelog_topic.get_balanced_consumer(
            consumer_group=self.consumer_group,
            managed=True,
            auto_offset_reset=OffsetType.LATEST,
            reset_offset_on_start=False,
            fetch_message_max_bytes=4000000, # up to ~4MBytes
            auto_commit_enable=True,
            auto_commit_interval_ms=30000, # 30 seconds
            compacted_topic=True,
        )

        # using a sync producer to try and avoid racey loss of delivery (aka,
        # if consumer group updated but produce didn't stick)
        release_producer = release_topic.get_sync_producer(
            max_request_size=self.produce_max_request_size,
        )
        file_producer = file_topic.get_sync_producer(
            max_request_size=self.produce_max_request_size,
        )
        container_producer = container_topic.get_sync_producer(
            max_request_size=self.produce_max_request_size,
        )

        for msg in consumer:
            cle = json.loads(msg.value.decode('utf-8'))
            #print(cle)
            print("processing changelog index {}".format(cle['index']))
            release_ids = []
            file_ids = []
            container_ids = []
            work_ids = []
            release_edits = cle['editgroup']['edits']['releases']
            for re in release_edits:
                release_ids.append(re['ident'])
            file_edits = cle['editgroup']['edits']['files']
            for e in file_edits:
                file_ids.append(e['ident'])
            container_edits = cle['editgroup']['edits']['containers']
            for e in container_edits:
                container_ids.append(e['ident'])
            work_edits = cle['editgroup']['edits']['works']
            for e in work_edits:
                work_ids.append(e['ident'])

            # TODO: do these fetches in parallel using a thread pool?
            for ident in set(file_ids):
                file_entity = self.api.get_file(ident, expand=None)
                # update release when a file changes
                # TODO: fetch old revision as well, and only update
                # releases for which list changed
                release_ids.extend(e['release_ids'])
                file_dict = self.api.api_client.sanitize_for_serialization(file_entity)
                file_producer.produce(
                    message=json.dumps(file_dict).encode('utf-8'),
                    partition_key=ident.encode('utf-8'),
                    timestamp=None,
                )
            for ident in set(container_ids):
                container = self.api.get_container(ident)
                container_dict = self.api.api_client.sanitize_for_serialization(container)
                container_producer.produce(
                    message=json.dumps(container_dict).encode('utf-8'),
                    partition_key=ident.encode('utf-8'),
                    timestamp=None,
                )
            for ident in set(release_ids):
                release = self.api.get_release(ident, expand="files,filesets,webcaptures,container")
                work_ids.append(release.work_id)
                release_dict = self.api.api_client.sanitize_for_serialization(release)
                release_producer.produce(
                    message=json.dumps(release_dict).encode('utf-8'),
                    partition_key=ident.encode('utf-8'),
                    timestamp=None,
                )
            # TODO: actually update works

            #consumer.commit_offsets()
