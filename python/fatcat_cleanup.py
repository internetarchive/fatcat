#!/usr/bin/env python3

import os
import sys
import argparse
import raven

from fatcat_tools import authenticated_api
from fatcat_tools.importers import JsonLinePusher
from fatcat_tools.cleanups import *


# Yep, a global. Gets DSN from `SENTRY_DSN` environment variable
sentry_client = raven.Client()


def run_files(args):
    fmi = FileCleaner(args.api,
        dry_run_mode=args.dry_run,
        edit_batch_size=args.batch_size,
        editgroup_description=args.editgroup_description_override)
    JsonLinePusher(fmi, args.json_file).run()

def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument('--fatcat-api-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    parser.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)
    parser.add_argument('--editgroup-description-override',
        help="editgroup description override",
        default=None, type=str)
    parser.add_argument('--dry-run',
        help="dry-run mode (don't actually update)",
        default=False, type=bool)
    subparsers = parser.add_subparsers()

    sub_files = subparsers.add_parser('files',
        help="attempt metadata cleanups over a list of file entities")
    sub_files.set_defaults(
        func=run_files,
        auth_var="FATCAT_AUTH_WORKER_CLEANUP",
    )
    sub_files.add_argument('json_file',
        help="files JSON file to import from",
        default=sys.stdin, type=argparse.FileType('r'))

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    # allow editgroup description override via env variable (but CLI arg takes
    # precedence)
    if not args.editgroup_description_override \
            and os.environ.get('FATCAT_EDITGROUP_DESCRIPTION'):
        args.editgroup_description_override = os.environ.get('FATCAT_EDITGROUP_DESCRIPTION')

    args.api = authenticated_api(
        args.fatcat_api_url,
        # token is an optional kwarg (can be empty string, None, etc)
        token=os.environ.get(args.auth_var))
    args.func(args)

if __name__ == '__main__':
    main()
