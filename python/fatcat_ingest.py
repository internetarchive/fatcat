#!/usr/bin/env python3

"""
Intended to be a command line interface to "Save Paper Now" and ingest
request/response.
"""

import argparse
import json
import sys
from collections import Counter

import elasticsearch
import raven
from elasticsearch_dsl import Q, Search

from fatcat_tools import kafka_fail_fast, public_api, simple_kafka_producer
from fatcat_tools.transforms import release_ingest_request

# Yep, a global. Gets DSN from `SENTRY_DSN` environment variable
sentry_client = raven.Client()


def _init_search(args: argparse.Namespace) -> Search:

    # ensure API connection works
    args.api.get_changelog()

    client = elasticsearch.Elasticsearch(args.elasticsearch_endpoint)
    search = Search(using=client, index=args.elasticsearch_index)
    return search


def _run_search_dump(args: argparse.Namespace, search: Search) -> None:

    if args.dry_run:
        print("=== THIS IS A DRY RUN ===")

    kafka_producer = None
    if args.kafka_request_topic:
        ingest_file_request_topic = args.kafka_request_topic
    else:
        ingest_file_request_topic = "sandcrawler-{}.ingest-file-requests-daily".format(args.env)
    if args.enqueue_kafka:
        print(
            "Will send ingest requests to kafka topic: {}".format(ingest_file_request_topic),
            file=sys.stderr,
        )
        kafka_producer = simple_kafka_producer(args.kafka_hosts)

    if args.limit is not None:
        search = search[: args.limit]

    if args.before_year:
        search = search.filter("exists", field="release_year").filter(
            "range", release_date=dict(lt=args.before_year)
        )
    if args.after_year:
        search = search.filter("exists", field="release_year").filter(
            "range", release_date=dict(gte=args.after_year)
        )

    if not args.allow_non_oa:
        search = search.filter("term", is_oa=True)

    if args.release_types:
        release_types = args.release_types.split(",")
        search = search.filter("terms", release_type=release_types)
    else:
        search = search.filter(
            "bool", must_not=[Q("terms", release_type=["stub", "component"])]
        )

    counts = Counter({"ingest_request": 0, "elasticsearch_release": 0, "estimate": 0})
    search = search.params()
    counts["estimate"] = search.count()
    print(
        "Expecting {} release objects in search queries".format(counts["estimate"]),
        file=sys.stderr,
    )

    # don't try to clean up scroll if we are connected to public server (behind
    # nginx proxy that disallows DELETE)
    if args.elasticsearch_endpoint in (
        "https://search.fatcat.wiki",
        "https://search.qa.fatcat.wiki",
    ):
        search = search.params(clear_scroll=False)

    results = search.scan()
    for esr in results:
        if args.limit and counts["ingest_request"] >= args.limit:
            break
        counts["elasticsearch_release"] += 1
        release = args.api.get_release(esr.ident)
        ingest_request = release_ingest_request(
            release,
            ingest_request_source="fatcat-ingest",
            ingest_type=args.ingest_type,
        )
        if not ingest_request:
            continue
        if args.force_recrawl:
            ingest_request["force_recrawl"] = True
        counts["ingest_request"] += 1
        if args.dry_run:
            continue
        if kafka_producer is not None:
            kafka_producer.produce(
                ingest_file_request_topic,
                json.dumps(ingest_request).encode("utf-8"),
                # key=None,
                on_delivery=kafka_fail_fast,
            )
            counts["kafka"] += 1
        else:
            print(json.dumps(ingest_request))
    if kafka_producer is not None:
        kafka_producer.flush()
    print(counts, file=sys.stderr)
    if args.dry_run:
        print("=== THIS WAS A DRY RUN ===")


def run_ingest_container(args: argparse.Namespace) -> None:
    """
    This command queries elasticsearch for releases from a given container (eg,
    journal), and prepares ingest requests for them.

    By default it filters to releases which don't have any fulltext files
    archived in IA, and dumps the ingest requests as JSON.
    """

    search = _init_search(args).filter("term", in_ia=False)

    # filter/query by container
    if args.container_id:
        search = search.filter("term", container_id=args.container_id)
    elif args.issnl:
        search = search.filter("term", container_issnl=args.issnl)
    elif args.publisher:
        search = search.query("match", publisher=args.publisher)
    elif args.name:
        search = search.query("match", container_name=args.name)
    else:
        print(
            "You must supply at least one query/filter parameter! Eg, ISSN-L", file=sys.stderr
        )
        sys.exit(-1)

    return _run_search_dump(args, search)


def run_ingest_query(args: argparse.Namespace) -> None:
    """
    Accepts a free-form Lucene query language string. Intended to work the same
    way as searches in the fatcat web interface.
    """

    search = (
        _init_search(args)
        .filter("term", in_ia=False)
        .query(
            "query_string",
            query=args.query,
            default_operator="AND",
            analyze_wildcard=True,
            lenient=True,
            fields=["title^5", "contrib_names^2", "container_title"],
        )
    )

    return _run_search_dump(args, search)


def run_ingest_extid(args: argparse.Namespace) -> None:
    """
    Selects release entities where the external identifier (extid) exists
    """

    search = _init_search(args).filter("term", in_ia=False).filter("exists", field=args.extid)

    return _run_search_dump(args, search)


def main() -> None:
    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument(
        "--fatcat-api-url", default="http://localhost:9411/v0", help="connect to this host/port"
    )
    parser.add_argument(
        "--enqueue-kafka",
        action="store_true",
        help="send ingest requests directly to sandcrawler kafka topic for processing",
    )
    parser.add_argument(
        "--kafka-hosts",
        default="localhost:9092",
        help="list of Kafka brokers (host/port) to use",
    )
    parser.add_argument("--kafka-request-topic", help="exact Kafka ingest request topic to use")
    parser.add_argument(
        "--elasticsearch-endpoint",
        default="https://search.fatcat.wiki",
        help="elasticsearch API. internal endpoint preferred, but public is default",
    )
    parser.add_argument(
        "--elasticsearch-index", default="fatcat_release", help="elasticsearch index to query"
    )
    parser.add_argument(
        "--env", default="dev", help="Kafka topic namespace to use (eg, prod, qa, dev)"
    )
    parser.add_argument(
        "--limit", default=None, type=int, help="Max number of search hits to return"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="runs through creating all ingest requests, but doesn't actually output or enqueue",
    )
    parser.add_argument(
        "--before-year",
        type=str,
        help="filters results to only with release_year before this (not inclusive)",
    )
    parser.add_argument(
        "--after-year",
        type=str,
        help="filters results to only with release_year after this (inclusive)",
    )
    parser.add_argument(
        "--release-types",
        type=str,
        help="filters results to specified release-types, separated by commas. By default, 'stub' is filtered out.",
    )
    parser.add_argument(
        "--allow-non-oa",
        action="store_true",
        help="By default, we limit to OA releases. This removes that filter",
    )
    parser.add_argument(
        "--force-recrawl",
        action="store_true",
        help="Tell ingest worker to skip GWB history lookup and do SPNv2 crawl",
    )
    parser.add_argument(
        "--ingest-type", default="pdf", help="What medium to ingest (pdf, xml, html)"
    )
    subparsers = parser.add_subparsers()

    sub_container = subparsers.add_parser(
        "container", help="Create ingest requests for releases from a specific container"
    )
    sub_container.set_defaults(func=run_ingest_container)
    sub_container.add_argument("--container-id", help="fatcat container entity ident")
    sub_container.add_argument("--issnl", help="ISSN-L of container entity")
    sub_container.add_argument("--publisher", help="publisher name")
    sub_container.add_argument("--name", help="container name")

    sub_query = subparsers.add_parser(
        "query", help="Create ingest requests for releases from a specific query"
    )
    sub_query.set_defaults(func=run_ingest_query)
    sub_query.add_argument("query", help="search query (same DSL as web interface search)")

    sub_extid = subparsers.add_parser(
        "extid", help="Create ingest requests for releases that have given extid defined"
    )
    sub_extid.set_defaults(func=run_ingest_extid)
    sub_extid.add_argument("extid", help="extid short name (as included in ES release schema)")

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = public_api(args.fatcat_api_url)
    args.func(args)


if __name__ == "__main__":
    main()
