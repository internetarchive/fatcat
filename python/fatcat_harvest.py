#!/usr/bin/env python3

import sys
import argparse
import datetime
import raven
from fatcat_tools.harvest import HarvestCrossrefWorker, HarvestDataciteWorker,\
    HarvestArxivWorker, HarvestDoajArticleWorker, HarvestDoajJournalWorker,\
    PubmedFTPWorker

# Yep, a global. Gets DSN from `SENTRY_DSN` environment variable
sentry_client = raven.Client()


def run_crossref(args):
    worker = HarvestCrossrefWorker(
        kafka_hosts=args.kafka_hosts,
        produce_topic="fatcat-{}.api-crossref".format(args.env),
        state_topic="fatcat-{}.api-crossref-state".format(args.env),
        contact_email=args.contact_email,
        start_date=args.start_date,
        end_date=args.end_date)
    worker.run(continuous=args.continuous)

def run_datacite(args):
    worker = HarvestDataciteWorker(
        kafka_hosts=args.kafka_hosts,
        produce_topic="fatcat-{}.api-datacite".format(args.env),
        state_topic="fatcat-{}.api-datacite-state".format(args.env),
        contact_email=args.contact_email,
        start_date=args.start_date,
        end_date=args.end_date)
    worker.run(continuous=args.continuous)

def run_arxiv(args):
    worker = HarvestArxivWorker(
        kafka_hosts=args.kafka_hosts,
        produce_topic="fatcat-{}.oaipmh-arxiv".format(args.env),
        state_topic="fatcat-{}.oaipmh-arxiv-state".format(args.env),
        start_date=args.start_date,
        end_date=args.end_date)
    worker.run(continuous=args.continuous)

def run_pubmed(args):
    # worker = HarvestPubmedWorker(
    #     kafka_hosts=args.kafka_hosts,
    #     produce_topic="fatcat-{}.oaipmh-pubmed".format(args.env),
    #     state_topic="fatcat-{}.oaipmh-pubmed-state".format(args.env),
    #     start_date=args.start_date,
    #     end_date=args.end_date)
    # worker.run(continuous=args.continuous)
    worker = PubmedFTPWorker(
        kafka_hosts=args.kafka_hosts,
        produce_topic="fatcat-{}.ftp-pubmed".format(args.env),
        state_topic="fatcat-{}.ftp-pubmed-state".format(args.env),
        start_date=args.start_date,
        end_date=args.end_date)
    worker.run(continuous=args.continuous)

def run_doaj_article(args):
    worker = HarvestDoajArticleWorker(
        kafka_hosts=args.kafka_hosts,
        produce_topic="fatcat-{}.oaipmh-doaj-article".format(args.env),
        state_topic="fatcat-{}.oaipmh-doaj-article-state".format(args.env),
        start_date=args.start_date,
        end_date=args.end_date)
    worker.run(continuous=args.continuous)

def run_doaj_journal(args):
    worker = HarvestDoajJournalWorker(
        kafka_hosts=args.kafka_hosts,
        produce_topic="fatcat-{}.oaipmh-doaj-journal".format(args.env),
        state_topic="fatcat-{}.oaipmh-doaj-journal-state".format(args.env),
        start_date=args.start_date,
        end_date=args.end_date)
    worker.run(continuous=args.continuous)


def mkdate(raw):
    return datetime.datetime.strptime(raw, "%Y-%m-%d").date()

def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument('--kafka-hosts',
        default="localhost:9092",
        help="list of Kafka brokers (host/port) to use")
    parser.add_argument('--env',
        default="dev",
        help="Kafka topic namespace to use (eg, prod, qa, dev)")
    parser.add_argument('--start-date',
        default=None, type=mkdate,
        help="begining of harvest period")
    parser.add_argument('--end-date',
        default=None, type=mkdate,
        help="end of harvest period")
    parser.add_argument('--contact-email',
        default="undefined",    # better?
        help="contact email to use in API header")
    parser.add_argument('--continuous',
        action='store_true',
        help="continue harvesting indefinitely in a loop?")
    subparsers = parser.add_subparsers()

    sub_crossref = subparsers.add_parser('crossref',
        help="harvest DOI metadata from Crossref API (JSON)")
    sub_crossref.set_defaults(func=run_crossref)

    sub_datacite = subparsers.add_parser('datacite',
        help="harvest DOI metadata from Datacite API (JSON)")
    sub_datacite.set_defaults(func=run_datacite)

    sub_arxiv = subparsers.add_parser('arxiv',
        help="harvest metadata from arxiv.org OAI-PMH endpoint (XML)")
    sub_arxiv.set_defaults(func=run_arxiv)

    sub_pubmed = subparsers.add_parser('pubmed',
        help="harvest MEDLINE/PubMed metadata from daily FTP updates (XML)")
    sub_pubmed.set_defaults(func=run_pubmed)

    # DOAJ stuff disabled because API range-requests are broken
    #sub_doaj_article = subparsers.add_parser('doaj-article')
    #sub_doaj_article.set_defaults(func=run_doaj_article)
    #sub_doaj_journal = subparsers.add_parser('doaj-journal')
    #sub_doaj_journal.set_defaults(func=run_doaj_journal)

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)
    args.func(args)

if __name__ == '__main__':
    main()
