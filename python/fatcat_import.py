#!/usr/bin/env python3

import sys
import argparse
from fatcat_tools.importers import CrossrefImporter, OrcidImporter, \
    IssnImporter, MatchedImporter, GrobidMetadataImporter 


def run_crossref(args):
    fci = CrossrefImporter(args.host_url, args.issn_map_file,
        args.extid_map_file, create_containers=(not args.no_create_containers))
    fci.process_batch(args.json_file, size=args.batch_size)
    fci.describe_run()

def run_orcid(args):
    foi = OrcidImporter(args.host_url)
    foi.process_batch(args.json_file, size=args.batch_size)
    foi.describe_run()

def run_issn(args):
    fii = IssnImporter(args.host_url)
    fii.process_csv_batch(args.csv_file, size=args.batch_size)
    fii.describe_run()

def run_matched(args):
    fmi = MatchedImporter(args.host_url,
        skip_file_update=args.no_file_update)
    fmi.process_batch(args.json_file, size=args.batch_size)
    fmi.describe_run()

def run_grobid_metadata(args):
    fmi = GrobidMetadataImporter(args.host_url)
    fmi.process_source(args.tsv_file, group_size=args.group_size)
    fmi.describe_run()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debugging interface")
    parser.add_argument('--host-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    subparsers = parser.add_subparsers()

    sub_crossref = subparsers.add_parser('crossref')
    sub_crossref.set_defaults(func=run_crossref)
    sub_crossref.add_argument('json_file',
        help="crossref JSON file to import from",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_crossref.add_argument('issn_map_file',
        help="ISSN to ISSN-L mapping file",
        default=None, type=argparse.FileType('r'))
    sub_crossref.add_argument('extid_map_file',
        help="DOI-to-other-identifiers sqlite3 database",
        default=None, type=str)
    sub_crossref.add_argument('--no-create-containers',
        action='store_true',
        help="skip creation of new container entities based on ISSN")
    sub_crossref.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)

    sub_orcid = subparsers.add_parser('orcid')
    sub_orcid.set_defaults(func=run_orcid)
    sub_orcid.add_argument('json_file',
        help="orcid JSON file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_orcid.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)

    sub_issn = subparsers.add_parser('issn')
    sub_issn.set_defaults(func=run_issn)
    sub_issn.add_argument('csv_file',
        help="Journal ISSN CSV metadata file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_issn.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)

    sub_matched = subparsers.add_parser('matched')
    sub_matched.set_defaults(func=run_matched)
    sub_matched.add_argument('json_file',
        help="JSON file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_matched.add_argument('--no-file-update',
        action='store_true',
        help="don't lookup existing files, just insert (only for bootstrap)")
    sub_matched.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)

    sub_grobid_metadata = subparsers.add_parser('grobid-metadata')
    sub_grobid_metadata.set_defaults(func=run_grobid_metadata)
    sub_grobid_metadata.add_argument('tsv_file',
        help="TSV file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_grobid_metadata.add_argument('--group-size',
        help="editgroup group size to use",
        default=75, type=int)

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)
    args.func(args)

if __name__ == '__main__':
    main()
