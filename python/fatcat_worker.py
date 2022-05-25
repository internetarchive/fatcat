#!/usr/bin/env python3

import argparse
import sys

import sentry_sdk

from fatcat_tools import public_api
from fatcat_tools.workers import (
    ChangelogWorker,
    ElasticsearchChangelogWorker,
    ElasticsearchContainerWorker,
    ElasticsearchFileWorker,
    ElasticsearchReleaseWorker,
    EntityUpdatesWorker,
)


def run_changelog(args: argparse.Namespace) -> None:
    topic = f"fatcat-{args.env}.changelog"
    worker = ChangelogWorker(
        args.api, args.kafka_hosts, topic, poll_interval=args.poll_interval
    )
    worker.run()


def run_entity_updates(args: argparse.Namespace) -> None:
    changelog_topic = f"fatcat-{args.env}.changelog"
    release_topic = f"fatcat-{args.env}.release-updates-v03"
    file_topic = f"fatcat-{args.env}.file-updates"
    container_topic = f"fatcat-{args.env}.container-updates"
    work_ident_topic = f"fatcat-{args.env}.work-ident-updates"
    ingest_file_request_topic = f"sandcrawler-{args.env}.ingest-file-requests-daily"
    worker = EntityUpdatesWorker(
        args.api,
        args.kafka_hosts,
        changelog_topic,
        release_topic=release_topic,
        file_topic=file_topic,
        container_topic=container_topic,
        work_ident_topic=work_ident_topic,
        ingest_file_request_topic=ingest_file_request_topic,
    )
    worker.run()


def run_elasticsearch_release(args: argparse.Namespace) -> None:
    consume_topic = f"fatcat-{args.env}.release-updates-v03"
    worker = ElasticsearchReleaseWorker(
        args.kafka_hosts,
        consume_topic,
        elasticsearch_backend=args.elasticsearch_backend,
        elasticsearch_index=args.elasticsearch_index,
    )
    worker.run()


def run_elasticsearch_container(args: argparse.Namespace) -> None:
    consume_topic = f"fatcat-{args.env}.container-updates"
    worker = ElasticsearchContainerWorker(
        args.kafka_hosts,
        consume_topic,
        query_stats=args.query_stats,
        elasticsearch_release_index="fatcat_release",
        elasticsearch_backend=args.elasticsearch_backend,
        elasticsearch_index=args.elasticsearch_index,
    )
    worker.run()


def run_elasticsearch_file(args: argparse.Namespace) -> None:
    consume_topic = f"fatcat-{args.env}.file-updates"
    worker = ElasticsearchFileWorker(
        args.kafka_hosts,
        consume_topic,
        elasticsearch_backend=args.elasticsearch_backend,
        elasticsearch_index=args.elasticsearch_index,
    )
    worker.run()


def run_elasticsearch_changelog(args: argparse.Namespace) -> None:
    consume_topic = f"fatcat-{args.env}.changelog"
    worker = ElasticsearchChangelogWorker(
        args.kafka_hosts,
        consume_topic,
        elasticsearch_backend=args.elasticsearch_backend,
        elasticsearch_index=args.elasticsearch_index,
    )
    worker.run()


def main() -> None:
    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument(
        "--api-host-url", default="http://localhost:9411/v0", help="fatcat API host/port to use"
    )
    parser.add_argument(
        "--kafka-hosts",
        default="localhost:9092",
        help="list of Kafka brokers (host/port) to use",
    )
    parser.add_argument(
        "--env", default="dev", help="Kafka topic namespace to use (eg, prod, qa, dev)"
    )
    subparsers = parser.add_subparsers()

    sub_changelog = subparsers.add_parser(
        "changelog", help="poll fatcat API for changelog entries, push to kafka"
    )
    sub_changelog.set_defaults(func=run_changelog)
    sub_changelog.add_argument(
        "--poll-interval",
        help="how long to wait between polling (seconds)",
        default=5.0,
        type=float,
    )

    sub_entity_updates = subparsers.add_parser(
        "entity-updates",
        help="poll kafka for changelog entries; push entity changes to various kafka topics",
    )
    sub_entity_updates.set_defaults(func=run_entity_updates)

    sub_elasticsearch_release = subparsers.add_parser(
        "elasticsearch-release",
        help="consume kafka feed of new/updated releases, transform and push to search",
    )
    sub_elasticsearch_release.set_defaults(func=run_elasticsearch_release)
    sub_elasticsearch_release.add_argument(
        "--elasticsearch-backend",
        help="elasticsearch backend to connect to",
        default="http://localhost:9200",
    )
    sub_elasticsearch_release.add_argument(
        "--elasticsearch-index",
        help="elasticsearch index to push into",
        default="fatcat_release_v03",
    )

    sub_elasticsearch_container = subparsers.add_parser(
        "elasticsearch-container",
        help="consume kafka feed of new/updated containers, transform and push to search",
    )
    sub_elasticsearch_container.set_defaults(func=run_elasticsearch_container)
    sub_elasticsearch_container.add_argument(
        "--elasticsearch-backend",
        help="elasticsearch backend to connect to",
        default="http://localhost:9200",
    )
    sub_elasticsearch_container.add_argument(
        "--elasticsearch-index",
        help="elasticsearch index to push into",
        default="fatcat_container",
    )
    sub_elasticsearch_container.add_argument(
        "--query-stats",
        action="store_true",
        help="whether to query release search index for container stats",
    )

    sub_elasticsearch_file = subparsers.add_parser(
        "elasticsearch-file",
        help="consume kafka feed of new/updated files, transform and push to search",
    )
    sub_elasticsearch_file.set_defaults(func=run_elasticsearch_file)
    sub_elasticsearch_file.add_argument(
        "--elasticsearch-backend",
        help="elasticsearch backend to connect to",
        default="http://localhost:9200",
    )
    sub_elasticsearch_file.add_argument(
        "--elasticsearch-index",
        help="elasticsearch index to push into",
        default="fatcat_file",
    )

    sub_elasticsearch_changelog = subparsers.add_parser(
        "elasticsearch-changelog",
        help="consume changelog kafka feed, transform and push to search",
    )
    sub_elasticsearch_changelog.set_defaults(func=run_elasticsearch_changelog)
    sub_elasticsearch_changelog.add_argument(
        "--elasticsearch-backend",
        help="elasticsearch backend to connect to",
        default="http://localhost:9200",
    )
    sub_elasticsearch_changelog.add_argument(
        "--elasticsearch-index",
        help="elasticsearch index to push into",
        default="fatcat_changelog",
    )

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = public_api(args.api_host_url)
    sentry_sdk.init(environment=args.env)
    args.func(args)


if __name__ == "__main__":
    main()
