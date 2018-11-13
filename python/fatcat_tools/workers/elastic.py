
import json
import time
import requests
from fatcat_tools.workers.worker_common import FatcatWorker
from fatcat_client import ReleaseEntity
from fatcat_tools.transforms import *
from pykafka.common import OffsetType


class FatcatElasticReleaseWorker(FatcatWorker):
    """
    Consumes from release-updates topic and pushes into (presumably local)
    elasticsearch.

    Uses a consumer group to manage offset.
    """

    def __init__(self, kafka_hosts, consume_topic, poll_interval=10.0, offset=None,
            elastic_backend="http://localhost:9200", elastic_index="fatcat"):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic,
                         api_host_url=None)
        self.consumer_group = "elastic-updates"
        self.elastic_backend = elastic_backend
        self.elastic_index = elastic_index

    def run(self):
        consume_topic = self.kafka.topics[self.consume_topic]

        consumer = consume_topic.get_balanced_consumer(
            consumer_group=self.consumer_group,
            managed=True,
        )

        for msg in consumer:
            json_str = msg.value.decode('utf-8')
            release = entity_from_json(json_str, ReleaseEntity)
            #print(release)
            elastic_endpoint = "{}/{}/release/{}".format(
                self.elastic_backend,
                self.elastic_index,
                release.ident)
            print("Updating document: {}".format(elastic_endpoint))
            resp = requests.post(elastic_endpoint, json=release.to_elastic_dict())
            assert resp.status_code in (200, 201)
            consumer.commit_offsets()
