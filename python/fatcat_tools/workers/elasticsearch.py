
import json
import sys

import elasticsearch
import requests
from confluent_kafka import Consumer, KafkaException
from fatcat_openapi_client import ApiClient, ChangelogEntry, ContainerEntity, ReleaseEntity

from fatcat_tools import entity_from_json, public_api
from fatcat_tools.transforms import (
    changelog_to_elasticsearch,
    container_to_elasticsearch,
    release_to_elasticsearch,
)
from fatcat_web.search import get_elastic_container_stats

from .worker_common import FatcatWorker


class ElasticsearchReleaseWorker(FatcatWorker):
    """
    Consumes from release-updates topic and pushes into (presumably local)
    elasticsearch.

    Uses a consumer group to manage offset.
    """

    def __init__(self, kafka_hosts, consume_topic, poll_interval=10.0, offset=None,
            elasticsearch_backend="http://localhost:9200", elasticsearch_index="fatcat",
            elasticsearch_release_index="fatcat_releases",
            batch_size=200, api_host="https://api.fatcat.wiki/v0", query_stats=False):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic)
        self.consumer_group = "elasticsearch-updates3"
        self.batch_size = batch_size
        self.poll_interval = poll_interval
        self.elasticsearch_backend = elasticsearch_backend
        self.elasticsearch_index = elasticsearch_index
        self.elasticsearch_release_index = elasticsearch_release_index
        self.entity_type = ReleaseEntity
        self.transform_func = release_to_elasticsearch
        self.api_host = api_host
        self.query_stats = query_stats

    def run(self):
        ac = ApiClient()
        api = public_api(self.api_host)

        # only used by container indexing query_stats code path
        es_client = elasticsearch.Elasticsearch(self.elasticsearch_backend)

        def fail_fast(err, partitions):
            if err is not None:
                print("Kafka consumer commit error: {}".format(err), file=sys.stderr)
                print("Bailing out...", file=sys.stderr)
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)
            for p in partitions:
                # check for partition-specific commit errors
                if p.error:
                    print("Kafka consumer commit error: {}".format(p.error), file=sys.stderr)
                    print("Bailing out...", file=sys.stderr)
                    # TODO: should it be sys.exit(-1)?
                    raise KafkaException(p.error)
            #print("Kafka consumer commit successful")
            pass

        def on_rebalance(consumer, partitions):
            for p in partitions:
                if p.error:
                    raise KafkaException(p.error)
            print("Kafka partitions rebalanced: {} / {}".format(
                consumer, partitions), file=sys.stderr)

        consumer_conf = self.kafka_config.copy()
        consumer_conf.update({
            'group.id': self.consumer_group,
            'on_commit': fail_fast,
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
                    print("... no Kafka consumer partitions assigned yet", file=sys.stderr)
                print("... nothing new from kafka, try again (interval: {}".format(self.poll_interval), file=sys.stderr)
                continue
            print("... got {} kafka messages".format(len(batch)), file=sys.stderr)
            # first check errors on entire batch...
            for msg in batch:
                if msg.error():
                    raise KafkaException(msg.error())
            # ... then process
            bulk_actions = []
            for msg in batch:
                json_str = msg.value().decode('utf-8')
                entity = entity_from_json(json_str, self.entity_type, api_client=ac)
                assert isinstance(entity, self.entity_type)
                if self.entity_type == ChangelogEntry:
                    key = entity.index
                    # might need to fetch from API
                    if not (entity.editgroup and entity.editgroup.editor): # pylint: disable=no-member # (TODO)
                        entity = api.get_changelog_entry(entity.index)
                else:
                    key = entity.ident  # pylint: disable=no-member # (TODO)

                if self.entity_type != ChangelogEntry and entity.state == 'wip':
                    print(f"WARNING: skipping state=wip entity: {self.entity_type.__name__} {entity.ident}", file=sys.stderr)
                    continue

                if self.entity_type == ContainerEntity and self.query_stats:
                    stats = get_elastic_container_stats(
                        entity.ident,
                        es_client=es_client,
                        es_index=self.elasticsearch_release_index,
                        merge_shadows=True,
                    )
                    doc_dict = container_to_elasticsearch(entity, stats=stats)
                else:
                    doc_dict = self.transform_func(entity)

                # TODO: handle deletions from index
                bulk_actions.append(json.dumps({
                    "index": { "_id": key, },
                }))
                bulk_actions.append(json.dumps(doc_dict))

            # if only WIP entities, then skip
            if not bulk_actions:
                for msg in batch:
                    consumer.store_offsets(message=msg)
                continue

            print("Upserting, eg, {} (of {} {} in elasticsearch)".format(key, len(batch), self.entity_type.__name__), file=sys.stderr)
            elasticsearch_endpoint = "{}/{}/_bulk".format(
                self.elasticsearch_backend,
                self.elasticsearch_index)
            resp = requests.post(elasticsearch_endpoint,
                headers={"Content-Type": "application/x-ndjson"},
                data="\n".join(bulk_actions) + "\n")
            resp.raise_for_status()
            if resp.json()['errors']:
                desc = "Elasticsearch errors from post to {}:".format(elasticsearch_endpoint)
                print(desc, file=sys.stderr)
                print(resp.content, file=sys.stderr)
                raise Exception(desc)
            for msg in batch:
                # offsets are *committed* (to brokers) automatically, but need
                # to be marked as processed here
                consumer.store_offsets(message=msg)


class ElasticsearchContainerWorker(ElasticsearchReleaseWorker):

    def __init__(self, kafka_hosts, consume_topic, poll_interval=10.0, offset=None,
            query_stats=False, elasticsearch_release_index="fatcat_release",
            elasticsearch_backend="http://localhost:9200", elasticsearch_index="fatcat",
            batch_size=200):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic,
                         poll_interval=poll_interval,
                         offset=offset,
                         elasticsearch_backend=elasticsearch_backend,
                         elasticsearch_index=elasticsearch_index,
                         elasticsearch_release_index=elasticsearch_release_index,
                         query_stats=query_stats,
                         batch_size=batch_size)
        # previous group got corrupted (by pykafka library?)
        self.consumer_group = "elasticsearch-updates3"
        self.entity_type = ContainerEntity
        self.transform_func = container_to_elasticsearch


class ElasticsearchChangelogWorker(ElasticsearchReleaseWorker):
    """
    Pulls changelog messages from Kafka, runs transformations and indexes them.

    Note: Very early versions of changelog entries did not contain details
    about the editor or extra fields.
    """
    def __init__(self, kafka_hosts, consume_topic, poll_interval=10.0, offset=None,
            elasticsearch_backend="http://localhost:9200", elasticsearch_index="fatcat_changelog",
            batch_size=200):
        super().__init__(kafka_hosts=kafka_hosts,
                         consume_topic=consume_topic)
        self.consumer_group = "elasticsearch-updates3"
        self.batch_size = batch_size
        self.poll_interval = poll_interval
        self.elasticsearch_backend = elasticsearch_backend
        self.elasticsearch_index = elasticsearch_index
        self.entity_type = ChangelogEntry
        self.transform_func = changelog_to_elasticsearch
