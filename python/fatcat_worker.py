#!/usr/bin/env python3

import sys
import argparse
import datetime
import raven

from fatcat_tools import public_api
from fatcat_tools.workers import ChangelogWorker, EntityUpdatesWorker, ElasticsearchReleaseWorker

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
    worker = EntityUpdatesWorker(args.api, args.kafka_hosts,
        changelog_topic,
        release_topic=release_topic,
        file_topic=file_topic,
        container_topic=container_topic,
    )
    worker.run()

def run_elasticsearch_release(args):
    consume_topic = "fatcat-{}.release-updates-v03".format(args.env)
    worker = ElasticsearchReleaseWorker(args.kafka_hosts, consume_topic,
        elasticsearch_backend=args.elasticsearch_backend,
        elasticsearch_index=args.elasticsearch_index)
    worker.run()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debug logging")
    parser.add_argument('--api-host-url',
        default="http://localhost:9411/v0",
        help="fatcat API host/port to use")
    parser.add_argument('--kafka-hosts',
        default="localhost:9092",
        help="list of Kafka brokers (host/port) to use")
    parser.add_argument('--env',
        default="qa",
        help="Kafka topic namespace to use (eg, prod, qa)")
    subparsers = parser.add_subparsers()

    sub_changelog = subparsers.add_parser('changelog')
    sub_changelog.set_defaults(func=run_changelog)
    sub_changelog.add_argument('--poll-interval',
        help="how long to wait between polling (seconds)",
        default=10.0, type=float)

    sub_entity_updates = subparsers.add_parser('entity-updates')
    sub_entity_updates.set_defaults(func=run_entity_updates)

    sub_elasticsearch_release = subparsers.add_parser('elasticsearch-release')
    sub_elasticsearch_release.set_defaults(func=run_elasticsearch_release)
    sub_elasticsearch_release.add_argument('--elasticsearch-backend',
        help="elasticsearch backend to connect to",
        default="http://localhost:9200")
    sub_elasticsearch_release.add_argument('--elasticsearch-index',
        help="elasticsearch index to push into",
        default="fatcat_release_v03")

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = public_api(args.api_host_url)
    args.func(args)

if __name__ == '__main__':
    main()
