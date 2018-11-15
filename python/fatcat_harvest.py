#!/usr/bin/env python3

import sys
import argparse
import datetime
from fatcat_tools.harvest import HarvestCrossrefWorker, HarvestDataciteWorker

def run_crossref(args):
    worker = HarvestCrossrefWorker(
        args.kafka_hosts,
        produce_topic="fatcat-{}.crossref".format(args.env),
        state_topic="fatcat-{}.crossref-state".format(args.env),
        contact_email=args.contact_email,
        start_date=args.start_date,
        end_date=args.end_date)
    worker.run_once()

def run_datacite(args):
    worker = HarvestDataciteWorker(
        args.kafka_hosts,
        produce_topic="fatcat-{}.datacite".format(args.env),
        state_topic="fatcat-{}.datacite-state".format(args.env),
        contact_email=args.contact_email,
        start_date=args.start_date,
        end_date=args.end_date)
    worker.run_once()

def mkdate(raw):
    return datetime.datetime.strptime(raw, "%Y-%m-%d").date()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debug logging")
    parser.add_argument('--kafka-hosts',
        default="localhost:9092",
        help="list of Kafka brokers (host/port) to use")
    parser.add_argument('--env',
        default="qa",
        help="Kafka topic namespace to use (eg, prod, qa)")
    parser.add_argument('--start-date',
        default=None, type=mkdate,
        help="begining of harvest period")
    parser.add_argument('--end-date',
        default=None, type=mkdate,
        help="end of harvest period")
    parser.add_argument('--contact-email',
        default="undefined",    # better?
        help="contact email to use in API header")
    subparsers = parser.add_subparsers()

    sub_crossref = subparsers.add_parser('crossref')
    sub_crossref.set_defaults(func=run_crossref)

    sub_datacite = subparsers.add_parser('datacite')
    sub_datacite.set_defaults(func=run_datacite)

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)
    args.func(args)

if __name__ == '__main__':
    main()
