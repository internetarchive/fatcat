
import json
import time
from confluent_kafka import Consumer, Producer, KafkaException

from .worker_common import FatcatWorker, most_recent_message


class ChangelogWorker(FatcatWorker):
    """
    Periodically polls the fatcat API looking for new changelogs. When they are
    found, fetch them and push (as JSON) into a Kafka topic.
    """

    def __init__(self, api, kafka_hosts, produce_topic, poll_interval=10.0, offset=None):
        super().__init__(kafka_hosts=kafka_hosts,
                         produce_topic=produce_topic,
                         api=api)
        self.poll_interval = poll_interval
        self.offset = offset    # the fatcat changelog offset, not the kafka offset

    def run(self):

        # On start, try to consume the most recent from the topic, and using
        # that as the starting offset. Note that this is a single-partition
        # topic
        if self.offset is None:
            print("Checking for most recent changelog offset...")
            msg = most_recent_message(self.produce_topic, self.kafka_config)
            if msg:
                self.offset = json.loads(msg.decode('utf-8'))['index']
            else:
                self.offset = 0
            print("Most recent changelog index in Kafka seems to be {}".format(self.offset))

        def fail_fast(err, msg):
            if err is not None:
                print("Kafka producer delivery error: {}".format(err))
                print("Bailing out...")
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)

        producer = Producer(self.kafka_config)

        while True:
            latest = int(self.api.get_changelog(limit=1)[0].index)
            if latest > self.offset:
                print("Fetching changelogs from {} through {}".format(
                    self.offset+1, latest))
            for i in range(self.offset+1, latest+1):
                cle = self.api.get_changelog_entry(i)
                obj = self.api.api_client.sanitize_for_serialization(cle)
                producer.produce(
                    self.produce_topic,
                    json.dumps(obj).encode('utf-8'),
                    key=str(i),
                    on_delivery=fail_fast,
                    #NOTE timestamp could be timestamp=cle.timestamp (?)
                )
                self.offset = i
            producer.poll(0)
            print("Sleeping {} seconds...".format(self.poll_interval))
            time.sleep(self.poll_interval)


class EntityUpdatesWorker(FatcatWorker):
    """
    Consumes from the changelog topic and publishes expanded entities (fetched
    from API) to update topics.

    For now, only release updates are published.
    """

    def __init__(self, api, kafka_hosts, consume_topic, release_topic, file_topic, container_topic, poll_interval=5.0):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic,
                         api=api)
        self.release_topic = release_topic
        self.file_topic = file_topic
        self.container_topic = container_topic
        self.poll_interval = poll_interval
        self.consumer_group = "entity-updates"

    def run(self):

        def fail_fast(err, msg):
            if err is not None:
                print("Kafka producer delivery error: {}".format(err))
                print("Bailing out...")
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)

        def on_commit(err, partitions):
            if err is not None:
                print("Kafka consumer commit error: {}".format(err))
                print("Bailing out...")
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)
            for p in partitions:
                # check for partition-specific commit errors
                print(p)
                if p.error:
                    print("Kafka consumer commit error: {}".format(p.error))
                    print("Bailing out...")
                    # TODO: should it be sys.exit(-1)?
                    raise KafkaException(err)
            print("Kafka consumer commit successful")
            pass

        def on_rebalance(consumer, partitions):
            for p in partitions:
                if p.error:
                    raise KafkaException(p.error)
            print("Kafka partitions rebalanced: {} / {}".format(
                consumer, partitions))

        consumer_conf = self.kafka_config.copy()
        consumer_conf.update({
            'group.id': self.consumer_group,
            'on_commit': fail_fast,
            'delivery.report.only.error': True,
            # messages don't have offset marked as stored until pushed to
            # elastic, but we do auto-commit stored offsets to broker
            'enable.auto.commit': True,
            'enable.auto.offset.store': False,
            # user code timeout; if no poll after this long, assume user code
            # hung and rebalance (default: 5min)
            'max.poll.interval.ms': 180000,
            'default.topic.config': {
                'auto.offset.reset': 'latest',
            },
        })
        consumer = Consumer(consumer_conf)

        producer_conf = self.kafka_config.copy()
        producer_conf.update({
            'default.topic.config': {
                'request.required.acks': -1,
            },
        })
        producer = Producer(producer_conf)

        consumer.subscribe([self.consume_topic],
            on_assign=on_rebalance,
            on_revoke=on_rebalance,
        )
        print("Kafka consuming {}".format(self.consume_topic))

        while True:
            msg = consumer.poll(self.poll_interval)
            if not msg:
                print("nothing new from kafka (poll_interval: {} sec)".format(self.poll_interval))
                continue
            if msg.error():
                raise KafkaException(msg.error())

            cle = json.loads(msg.value().decode('utf-8'))
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
                release_ids.extend(file_entity.release_ids or [])
                file_dict = self.api.api_client.sanitize_for_serialization(file_entity)
                producer.produce(
                    self.file_topic,
                    json.dumps(file_dict).encode('utf-8'),
                    key=ident.encode('utf-8'),
                    on_delivery=fail_fast,
                )
            for ident in set(container_ids):
                container = self.api.get_container(ident)
                container_dict = self.api.api_client.sanitize_for_serialization(container)
                producer.produce(
                    self.container_topic,
                    json.dumps(container_dict).encode('utf-8'),
                    key=ident.encode('utf-8'),
                    on_delivery=fail_fast,
                )
            for ident in set(release_ids):
                release = self.api.get_release(ident, expand="files,filesets,webcaptures,container")
                work_ids.append(release.work_id)
                release_dict = self.api.api_client.sanitize_for_serialization(release)
                producer.produce(
                    self.release_topic,
                    json.dumps(release_dict).encode('utf-8'),
                    key=ident.encode('utf-8'),
                    on_delivery=fail_fast,
                )
            # TODO: actually update works
            consumer.store_offsets(message=msg)

