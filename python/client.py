#!/usr/bin/env python3

import sys
import argparse
from fatcat.raw_api_client import RawFatcatApiClient
from fatcat.crossref_importer import FatcatCrossrefImporter
from fatcat.orcid_importer import FatcatOrcidImporter
from fatcat.manifest_importer import FatcatManifestImporter

def run_import_crossref(args):
    fcc = FatcatCrossrefClient(args.host_url)
    fcc.import_crossref_file(
        args.json_file,
        issn_map_file=args.issn_map_file,
        create_containers=(not args.no_create_containers))

def run_import_orcid(args):
    foi = FatcatOrcidImporter(args.host_url)
    foi.process_batch(args.json_file, size=args.batch_size)

def run_import_issn(args):
    fii = FatcatIssnImporter(args.host_url)
    fii.process_batch(args.csv_file, size=args.batch_size)

def run_import_manifest(args):
    fmi = FatcatManifestImporter(args.host_url)
    fmi.process_db(args.db_path, size=args.batch_size)

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
    sub_import_crossref.add_argument('issn_map_file',
        help="ISSN to ISSN-L mapping file")
    sub_import_crossref.add_argument('--no-create-containers',
        action='store_true',
        help="skip creation of new container entities based on ISSN")

    sub_import_orcid = subparsers.add_parser('import-orcid')
    sub_import_orcid.set_defaults(func=run_import_orcid)
    sub_import_orcid.add_argument('json_file',
        help="orcid JSON file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_import_orcid.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)

    sub_import_issn = subparsers.add_parser('import-issn')
    sub_import_issn.set_defaults(func=run_import_issn)
    sub_import_issn.add_argument('csv_file',
        help="Journal ISSN CSV metadata file to import from (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_import_issn.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)

    sub_import_manifest = subparsers.add_parser('import-manifest')
    sub_import_manifest.set_defaults(func=run_import_manifest)
    sub_import_manifest.add_argument('db_path',
        help="sqlite3 database to import from",
        type=str)
    sub_import_manifest.add_argument('--batch-size',
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
