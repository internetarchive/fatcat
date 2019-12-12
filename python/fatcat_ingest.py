#!/usr/bin/env python3

"""
Intended to be a command line interface to "Save Paper Now" and ingest
request/response.
"""

import sys
import json
import argparse
from collections import Counter

from fatcat_tools import public_api, simple_kafka_producer, kafka_fail_fast
from fatcat_tools.transforms import release_ingest_request
import elasticsearch
from elasticsearch_dsl import Search


def run_ingest_container(args):
    """
    This command queries elasticsearch for releases from a given container (eg,
    journal), and prepares ingest requests for them.

    By default it filters to releases which don't have any fulltext files
    archived in IA, and dumps the ingest requests as JSON.
    """

    # ensure API connection works
    args.api.get_changelog()

    kafka_producer = None
    ingest_file_request_topic = "sandcrawler-{}.ingest-file-requests".format(args.env)
    if args.enqueue_kafka:
        print("Will send ingest requests to kafka topic: {}".format(ingest_file_request_topic), file=sys.stderr)
        kafka_producer = simple_kafka_producer(args.kafka_hosts)

    client = elasticsearch.Elasticsearch(args.elasticsearch_endpoint)

    s = Search(using=client, index="fatcat_release") \
        .filter("term", in_ia=False) \
        .filter("term", is_oa=True)

    # filter/query by container
    if args.container_id:
        s = s.filter("term", container_id=args.container_id)
    elif args.issnl:
        s = s.filter("term", container_issnl=args.issnl)
    elif args.publisher:
        s = s.query("match", publisher=args.publisher)
    elif args.name:
        s = s.query("match", container_name=args.name)
    else:
        print("You must supply at least one query/filter parameter! Eg, ISSN-L", file=sys.stderr)
        sys.exit(-1)

    counts = Counter({'ingest_request': 0, 'elasticsearch_release': 0, 'estimate': 0})
    counts['estimate'] = s.count()
    print("Expecting {} release objects in search queries".format(counts['estimate']), file=sys.stderr)

    # don't try to clean up scroll if we are connected to public server (behind
    # nginx proxy that disallows DELETE)
    if args.elasticsearch_endpoint in (
            'https://search.fatcat.wiki',
            'https://search.qa.fatcat.wiki'):
        s = s.params(clear_scroll=False)

    results = s.scan()
    for esr in results:
        counts['elasticsearch_release'] += 1
        release = args.api.get_release(esr.ident)
        ingest_request = release_ingest_request(
            release,
            oa_only=False,
            ingest_request_source="fatcat-ingest-container",
        )
        if not ingest_request:
            continue
        if kafka_producer != None:
            kafka_producer.produce(
                ingest_file_request_topic,
                json.dumps(ingest_request).encode('utf-8'),
                #key=None,
                on_delivery=kafka_fail_fast,
            )
            counts['kafka'] += 1
        # also printing to stdout when in kafka mode; could skip?
        print(json.dumps(ingest_request))
        counts['ingest_request'] += 1
    if kafka_producer != None:
        kafka_producer.flush()
    print(counts, file=sys.stderr)

def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument('--fatcat-api-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    parser.add_argument('--enqueue-kafka',
        action='store_true',
        help="send ingest requests directly to sandcrawler kafka topic for processing")
    parser.add_argument('--kafka-hosts',
        default="localhost:9092",
        help="list of Kafka brokers (host/port) to use")
    parser.add_argument('--elasticsearch-endpoint',
        default="https://search.fatcat.wiki",
        help="elasticsearch API. internal endpoint prefered, but public is default")
    parser.add_argument('--env',
        default="dev",
        help="Kafka topic namespace to use (eg, prod, qa, dev)")
    subparsers = parser.add_subparsers()

    sub_ingest_container = subparsers.add_parser('ingest-container',
        help="Create ingest requests for releases from a specific container")
    sub_ingest_container.set_defaults(func=run_ingest_container)
    sub_ingest_container.add_argument('--container-id',
        help="fatcat container entity ident")
    sub_ingest_container.add_argument('--issnl',
        help="ISSN-L of container entity")
    sub_ingest_container.add_argument('--publisher',
        help="publisher name")
    sub_ingest_container.add_argument('--name',
        help="container name")

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = public_api(args.fatcat_api_url)
    args.func(args)

if __name__ == '__main__':
    main()
