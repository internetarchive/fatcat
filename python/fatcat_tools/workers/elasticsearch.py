
import json
import time
import requests
from pykafka.common import OffsetType

from fatcat_client import ReleaseEntity, ContainerEntity, ApiClient
from fatcat_tools import *
from .worker_common import FatcatWorker


class ElasticsearchReleaseWorker(FatcatWorker):
    """
    Consumes from release-updates topic and pushes into (presumably local)
    elasticsearch.

    Uses a consumer group to manage offset.
    """

    def __init__(self, kafka_hosts, consume_topic, poll_interval=10.0, offset=None,
            elasticsearch_backend="http://localhost:9200", elasticsearch_index="fatcat"):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic)
        self.consumer_group = "elasticsearch-updates"
        self.elasticsearch_backend = elasticsearch_backend
        self.elasticsearch_index = elasticsearch_index
        self.entity_type = ReleaseEntity
        self.elasticsearch_document_name = "release"
        self.transform_func = release_to_elasticsearch

    def run(self):
        consume_topic = self.kafka.topics[self.consume_topic]
        ac = ApiClient()

        consumer = consume_topic.get_balanced_consumer(
            consumer_group=self.consumer_group,
            managed=True,
            fetch_message_max_bytes=10000000, # up to ~10 MBytes
            auto_commit_enable=True,
            auto_commit_interval_ms=30000, # 30 seconds
            compacted_topic=True,
        )

        for msg in consumer:
            json_str = msg.value.decode('utf-8')
            # HACK: work around a bug where container entities got published to
            # release_v03 topic
            if self.elasticsearch_document_name == "release":
                entity_dict = json.loads(json_str)
                if entity_dict.get('name') and not entity_dict.get('title'):
                    continue
            entity = entity_from_json(json_str, self.entity_type, api_client=ac)
            #print(entity)
            elasticsearch_endpoint = "{}/{}/{}/{}".format(
                self.elasticsearch_backend,
                self.elasticsearch_index,
                self.elasticsearch_document_name,
                entity.ident)
            print("Updating document: {}".format(elasticsearch_endpoint))
            resp = requests.post(elasticsearch_endpoint, json=self.transform_func(entity))
            resp.raise_for_status()
            #consumer.commit_offsets()


class ElasticsearchContainerWorker(ElasticsearchReleaseWorker):

    def __init__(self, kafka_hosts, consume_topic, poll_interval=10.0, offset=None,
            elasticsearch_backend="http://localhost:9200", elasticsearch_index="fatcat"):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic,
                         poll_interval=poll_interval,
                         offset=offset,
                         elasticsearch_backend=elasticsearch_backend,
                         elasticsearch_index=elasticsearch_index)
        # previous group got corrupted (by pykafka library?)
        self.consumer_group = "elasticsearch-updates2"
        self.entity_type = ContainerEntity
        self.elasticsearch_document_name = "container"
        self.transform_func = container_to_elasticsearch

