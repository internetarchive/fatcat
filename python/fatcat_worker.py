#!/usr/bin/env python3

import sys
import argparse
import datetime
import raven

from fatcat_tools import public_api
from fatcat_tools.workers import ChangelogWorker, EntityUpdatesWorker, ElasticsearchReleaseWorker, ElasticsearchContainerWorker

# Yep, a global. Gets DSN from `SENTRY_DSN` environment variable
sentry_client = raven.Client()


def run_changelog(args):
    topic = "fatcat-{}.changelog".format(args.env)
    worker = ChangelogWorker(args.api, args.kafka_hosts, topic,
        poll_interval=args.poll_interval)
    worker.run()

def run_entity_updates(args):
    changelog_topic = "fatcat-{}.changelog".format(args.env)
    release_topic = "fatcat-{}.release-updates-v03".format(args.env)
    file_topic = "fatcat-{}.file-updates".format(args.env)
    container_topic = "fatcat-{}.container-updates".format(args.env)
    ingest_file_request_topic = "sandcrawler-{}.ingest-file-requests".format(args.env)
    worker = EntityUpdatesWorker(args.api, args.kafka_hosts,
        changelog_topic,
        release_topic=release_topic,
        file_topic=file_topic,
        container_topic=container_topic,
        ingest_file_request_topic=ingest_file_request_topic,
    )
    worker.run()

def run_elasticsearch_release(args):
    consume_topic = "fatcat-{}.release-updates-v03".format(args.env)
    worker = ElasticsearchReleaseWorker(args.kafka_hosts, consume_topic,
        elasticsearch_backend=args.elasticsearch_backend,
        elasticsearch_index=args.elasticsearch_index)
    worker.run()

def run_elasticsearch_container(args):
    consume_topic = "fatcat-{}.container-updates".format(args.env)
    worker = ElasticsearchContainerWorker(args.kafka_hosts, consume_topic,
        elasticsearch_backend=args.elasticsearch_backend,
        elasticsearch_index=args.elasticsearch_index)
    worker.run()

def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument('--api-host-url',
        default="http://localhost:9411/v0",
        help="fatcat API host/port to use")
    parser.add_argument('--kafka-hosts',
        default="localhost:9092",
        help="list of Kafka brokers (host/port) to use")
    parser.add_argument('--env',
        default="dev",
        help="Kafka topic namespace to use (eg, prod, qa, dev)")
    subparsers = parser.add_subparsers()

    sub_changelog = subparsers.add_parser('changelog',
        help="poll fatcat API for changelog entries, push to kafka")
    sub_changelog.set_defaults(func=run_changelog)
    sub_changelog.add_argument('--poll-interval',
        help="how long to wait between polling (seconds)",
        default=5.0, type=float)

    sub_entity_updates = subparsers.add_parser('entity-updates',
        help="poll kafka for changelog entries; push entity changes to various kafka topics")
    sub_entity_updates.set_defaults(func=run_entity_updates)

    sub_elasticsearch_release = subparsers.add_parser('elasticsearch-release',
        help="consume kafka feed of new/updated releases, transform and push to search")
    sub_elasticsearch_release.set_defaults(func=run_elasticsearch_release)
    sub_elasticsearch_release.add_argument('--elasticsearch-backend',
        help="elasticsearch backend to connect to",
        default="http://localhost:9200")
    sub_elasticsearch_release.add_argument('--elasticsearch-index',
        help="elasticsearch index to push into",
        default="fatcat_release_v03")

    sub_elasticsearch_container = subparsers.add_parser('elasticsearch-container',
        help="consume kafka feed of new/updated containers, transform and push to search")
    sub_elasticsearch_container.set_defaults(func=run_elasticsearch_container)
    sub_elasticsearch_container.add_argument('--elasticsearch-backend',
        help="elasticsearch backend to connect to",
        default="http://localhost:9200")
    sub_elasticsearch_container.add_argument('--elasticsearch-index',
        help="elasticsearch index to push into",
        default="fatcat_container")

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = public_api(args.api_host_url)
    args.func(args)

if __name__ == '__main__':
    main()
