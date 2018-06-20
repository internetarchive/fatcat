#!/usr/bin/env python3

import sys
import argparse
from fatcat.raw_api_client import RawFatcatApiClient
from fatcat.orcid_importer import FatcatOrcidImporter

def run_import_crossref(args):
    fcc = FatcatCrossrefClient(args.host_url)
    fcc.import_crossref_file(args.json_file)
    # create_containers=args.create_containers

def run_import_orcid(args):
    foi = FatcatOrcidImporter(args.host_url)
    foi.process_batch(args.json_file, size=args.batch_size)

def health(args):
    rfac = RawFatcatApiClient(args.host_url)
    print(rfac.health())

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debugging interface")
    parser.add_argument('--host-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    subparsers = parser.add_subparsers()

    sub_import_crossref = subparsers.add_parser('import-crossref')
    sub_import_crossref.set_defaults(func=run_import_crossref)
    sub_import_crossref.add_argument('json_file',
        help="crossref JSON file to import from")
    # TODO:
    #sub_import_crossref.add_argument('--create-containers',
    #    action='store_true',
    #    help="if true, create containers based on ISSN")

    sub_import_orcid = subparsers.add_parser('import-orcid')
    sub_import_orcid.set_defaults(func=run_import_orcid)
    sub_import_orcid.add_argument('json_file',
        help="orcid JSON file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_import_orcid.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)

    sub_health = subparsers.add_parser('health')
    sub_health.set_defaults(func=health)

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)
    args.func(args)

if __name__ == '__main__':
    main()
