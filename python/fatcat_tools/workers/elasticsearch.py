
import json
import time
import requests
from confluent_kafka import Consumer, Producer, KafkaException

from fatcat_openapi_client import ReleaseEntity, ContainerEntity, ApiClient
from fatcat_tools import *
from .worker_common import FatcatWorker


class ElasticsearchReleaseWorker(FatcatWorker):
    """
    Consumes from release-updates topic and pushes into (presumably local)
    elasticsearch.

    Uses a consumer group to manage offset.
    """

    def __init__(self, kafka_hosts, consume_topic, poll_interval=10.0, offset=None,
            elasticsearch_backend="http://localhost:9200", elasticsearch_index="fatcat",
            batch_size=200):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic)
        self.consumer_group = "elasticsearch-updates3"
        self.batch_size = batch_size
        self.poll_interval = poll_interval
        self.elasticsearch_backend = elasticsearch_backend
        self.elasticsearch_index = elasticsearch_index
        self.entity_type = ReleaseEntity
        self.elasticsearch_document_name = "release"
        self.transform_func = release_to_elasticsearch

    def run(self):
        ac = ApiClient()

        def fail_fast(err, partitions):
            if err is not None:
                print("Kafka consumer commit error: {}".format(err))
                print("Bailing out...")
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)
            for p in partitions:
                # check for partition-specific commit errors
                if p.error:
                    print("Kafka consumer commit error: {}".format(p.error))
                    print("Bailing out...")
                    # TODO: should it be sys.exit(-1)?
                    raise KafkaException(err)
            #print("Kafka consumer commit successful")
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
            'max.poll.interval.ms': 60000,
            'default.topic.config': {
                'auto.offset.reset': 'latest',
            },
        })
        consumer = Consumer(consumer_conf)
        consumer.subscribe([self.consume_topic],
            on_assign=on_rebalance,
            on_revoke=on_rebalance,
        )

        while True:
            batch = consumer.consume(
                num_messages=self.batch_size,
                timeout=self.poll_interval)
            if not batch:
                if not consumer.assignment():
                    print("... no Kafka consumer partitions assigned yet")
                print("... nothing new from kafka, try again (interval: {}".format(self.poll_interval))
                continue
            print("... got {} kafka messages".format(len(batch)))
            # first check errors on entire batch...
            for msg in batch:
                if msg.error():
                    raise KafkaException(msg.error())
            # ... then process
            bulk_actions = []
            for msg in batch:
                json_str = msg.value().decode('utf-8')
                entity = entity_from_json(json_str, ReleaseEntity, api_client=ac)
                print("Upserting: release/{}".format(entity.ident))
                bulk_actions.append(json.dumps({
                    "index": { "_id": entity.ident, },
                }))
                bulk_actions.append(json.dumps(
                    release_to_elasticsearch(entity)))
            elasticsearch_endpoint = "{}/{}/release/_bulk".format(
                self.elasticsearch_backend,
                self.elasticsearch_index)
            resp = requests.post(elasticsearch_endpoint,
                headers={"Content-Type": "application/x-ndjson"},
                data="\n".join(bulk_actions) + "\n")
            resp.raise_for_status()
            if resp.json()['errors']:
                desc = "Elasticsearch errors from post to {}:".format(elasticsearch_endpoint)
                print(desc)
                print(resp.content)
                raise Exception(desc)
            for msg in batch:
                # offsets are *committed* (to brokers) automatically, but need
                # to be marked as processed here
                consumer.store_offsets(message=msg)



class ElasticsearchContainerWorker(ElasticsearchReleaseWorker):

    def __init__(self, kafka_hosts, consume_topic, poll_interval=10.0, offset=None,
            elasticsearch_backend="http://localhost:9200", elasticsearch_index="fatcat",
            batch_size=200):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic,
                         poll_interval=poll_interval,
                         offset=offset,
                         elasticsearch_backend=elasticsearch_backend,
                         elasticsearch_index=elasticsearch_index,
                         batch_size=batch_size)
        # previous group got corrupted (by pykafka library?)
        self.consumer_group = "elasticsearch-updates3"
        self.entity_type = ContainerEntity
        self.elasticsearch_document_name = "container"
        self.transform_func = container_to_elasticsearch


