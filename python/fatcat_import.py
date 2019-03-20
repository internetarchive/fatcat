#!/usr/bin/env python3

import os, sys, argparse
from fatcat_tools import authenticated_api
from fatcat_tools.importers import *


def run_crossref(args):
    fci = CrossrefImporter(args.api,
        args.issn_map_file,
        extid_map_file=args.extid_map_file,
        edit_batch_size=args.batch_size,
        bezerk_mode=args.bezerk_mode)
    if args.kafka_mode:
        KafkaJsonPusher(fci, args.kafka_hosts, args.kafka_env, "api-crossref", "fatcat-import").run()
    else:
        JsonLinePusher(fci, args.json_file).run()

def run_orcid(args):
    foi = OrcidImporter(args.api,
        edit_batch_size=args.batch_size)
    JsonLinePusher(foi, args.json_file).run()

def run_journal_metadata(args):
    fii = JournalMetadataImporter(args.api,
        edit_batch_size=args.batch_size)
    JsonLinePusher(fii, args.json_file).run()

def run_matched(args):
    fmi = MatchedImporter(args.api,
        edit_batch_size=args.batch_size)
    JsonLinePusher(fmi, args.json_file).run()

def run_grobid_metadata(args):
    fmi = GrobidMetadataImporter(args.api,
        edit_batch_size=args.batch_size,
        longtail_oa=args.longtail_oa,
        bezerk_mode=args.bezerk_mode)
    LinePusher(fmi, args.tsv_file).run()

def run_wayback_static(args):
    api = args.api

    # find the release
    if args.release_id:
        release_id = args.release_id
    elif args.extid:
        idtype = args.extid.split(':')[0]
        extid = ':'.join(args.extid.split(':')[1:])
        if idtype == "doi":
            release_id = api.lookup_release(doi=extid).ident
        elif idtype == "pmid":
            release_id = api.lookup_release(pmid=extid).ident
        elif idtype == "wikidata":
            release_id = api.lookup_release(wikidata_qid=extid).ident
        else:
            raise NotImplementedError("extid type: {}".format(idtype))
    else:
        raise Exception("need either release_id or extid argument")

    # create it
    (editgroup_id, wc) = auto_wayback_static(api, release_id, args.wayback_url,
        editgroup_id=args.editgroup_id)
    if not wc:
        return
    print("release_id: {}".format(release_id))
    print("editgroup_id: {}".format(editgroup_id))
    print("edit id: {}".format(wc.ident))
    print("link: https://fatcat.wiki/webcapture/{}".format(wc.ident))

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debugging interface")
    parser.add_argument('--host-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    parser.add_argument('--kafka-hosts',
        default="localhost:9092",
        help="list of Kafka brokers (host/port) to use")
    parser.add_argument('--kafka-env',
        default="qa",
        help="Kafka topic namespace to use (eg, prod, qa)")
    parser.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)
    subparsers = parser.add_subparsers()

    sub_crossref = subparsers.add_parser('crossref')
    sub_crossref.set_defaults(
        func=run_crossref,
        auth_var="FATCAT_AUTH_WORKER_CROSSREF",
    )
    sub_crossref.add_argument('json_file',
        help="crossref JSON file to import from",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_crossref.add_argument('issn_map_file',
        help="ISSN to ISSN-L mapping file",
        default=None, type=argparse.FileType('r'))
    sub_crossref.add_argument('--extid-map-file',
        help="DOI-to-other-identifiers sqlite3 database",
        default=None, type=str)
    sub_crossref.add_argument('--kafka-mode',
        action='store_true',
        help="consume from kafka topic (not stdin)")
    sub_crossref.add_argument('--bezerk-mode',
        action='store_true',
        help="don't lookup existing DOIs, just insert (clobbers; only for fast bootstrap)")

    sub_orcid = subparsers.add_parser('orcid')
    sub_orcid.set_defaults(
        func=run_orcid,
        auth_var="FATCAT_AUTH_WORKER_ORCID"
    )
    sub_orcid.add_argument('json_file',
        help="orcid JSON file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))

    sub_journal_metadata = subparsers.add_parser('journal-metadata')
    sub_journal_metadata.set_defaults(
        func=run_journal_metadata,
        auth_var="FATCAT_AUTH_WORKER_JOURNAL_METADATA",
    )
    sub_journal_metadata.add_argument('json_file',
        help="Journal JSON metadata file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))

    sub_matched = subparsers.add_parser('matched')
    sub_matched.set_defaults(
        func=run_matched,
        auth_var="FATCAT_API_AUTH_TOKEN",
    )
    sub_matched.add_argument('json_file',
        help="JSON file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_matched.add_argument('--bezerk-mode',
        action='store_true',
        help="don't lookup existing files, just insert (clobbers; only for fast bootstrap)")

    sub_grobid_metadata = subparsers.add_parser('grobid-metadata')
    sub_grobid_metadata.set_defaults(
        func=run_grobid_metadata,
        auth_var="FATCAT_API_AUTH_TOKEN",
    )
    sub_grobid_metadata.add_argument('tsv_file',
        help="TSV file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_grobid_metadata.add_argument('--group-size',
        help="editgroup group size to use",
        default=75, type=int)
    sub_grobid_metadata.add_argument('--longtail-oa',
        action='store_true',
        help="if this is an import of longtail OA content (sets an 'extra' flag)")
    sub_grobid_metadata.add_argument('--bezerk-mode',
        action='store_true',
        help="don't lookup existing files, just insert (clobbers; only for fast bootstrap)")

    sub_wayback_static = subparsers.add_parser('wayback-static')
    sub_wayback_static.set_defaults(
        func=run_wayback_static,
        auth_var="FATCAT_API_AUTH_TOKEN",
    )
    sub_wayback_static.add_argument('wayback_url',
        type=str,
        help="URL of wayback capture to extract from")
    sub_wayback_static.add_argument('--extid',
        type=str,
        help="external identifier for release lookup")
    sub_wayback_static.add_argument('--release-id',
        type=str,
        help="release entity identifier")
    sub_wayback_static.add_argument('--editgroup-id',
        type=str,
        help="use existing editgroup (instead of creating a new one)")

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = authenticated_api(
        args.host_url,
        # token is an optional kwarg (can be empty string, None, etc)
        token=os.environ.get(args.auth_var))
    args.func(args)

if __name__ == '__main__':
    main()
