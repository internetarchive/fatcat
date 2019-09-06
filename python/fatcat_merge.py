#!/usr/bin/env python3

"""
Tools for merging entities in various ways.

    group-releases: pull all release entities under a single work
        => merges work entities
    merge-releases: merge release entities together
        => groups files/filesets/webcaptures
    merge-containers: merge container entities
    merge-files: merge file entities

Input format is usually JSON lines with keys:

    idents (required): array of string identifiers
    primary (optional): single string identifier

"""

import os, sys, argparse
from fatcat_tools import authenticated_api
from fatcat_tools.mergers import *
from fatcat_tools.importers import JsonLinePusher


def run_group_releases(args):
    rg = ReleaseGrouper(args.api,
        edit_batch_size=args.batch_size,
        dry_run_mode=args.dry_run)
    JsonLinePusher(rg, args.json_file).run()

def run_merge_releases(args):
    rm = ReleaseMerger(args.api,
        edit_batch_size=args.batch_size,
        dry_run_mode=args.dry_run)
    JsonLinePusher(rg, args.json_file).run()

def run_merge_containers(args):
    cm = ReleaseMerger(args.api,
        edit_batch_size=args.batch_size,
        dry_run_mode=args.dry_run)
    JsonLinePusher(cm, args.json_file).run()

def run_merge_files(args):
    fm = FileMerger(args.api,
        edit_batch_size=args.batch_size,
        dry_run_mode=args.dry_run)
    JsonLinePusher(fm, args.json_file).run()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--host-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    parser.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)
    parser.add_argument('--editgroup-description-override',
        help="editgroup description override",
        default=None, type=str)
    parser.add_argument('--dry-run',
        action='store_true',
        help="don't actually commit merges, just count what would have been")
    parser.set_defaults(
        auth_var="FATCAT_AUTH_API_TOKEN",
    )
    subparsers = parser.add_subparsers()

    sub_group_releases = subparsers.add_parser('group-releases')
    sub_group_releases.set_defaults(func=run_group_releases)
    sub_group_releases.add_argument('json_file',
        help="source of merge lines to process (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))

    sub_merge_releases = subparsers.add_parser('merge-releases')
    sub_merge_releases.set_defaults(func=run_merge_releases)
    sub_merge_releases.add_argument('json_file',
        help="source of merge lines to process (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))

    sub_merge_files = subparsers.add_parser('merge-files')
    sub_merge_files.set_defaults(func=run_merge_files)
    sub_merge_files.add_argument('json_file',
        help="source of merge lines to process (or stdin)",
        default=sys.stdin, type=argparse.FileType('r'))

    sub_merge_containers = subparsers.add_parser('merge-containers')
    sub_merge_containers.set_defaults(func=run_merge_containers)
    sub_merge_containers.add_argument('json_file',
        help="source of merge lines to process (or stdin)",
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
        args.host_url,
        # token is an optional kwarg (can be empty string, None, etc)
        token=os.environ.get(args.auth_var))
    args.func(args)

if __name__ == '__main__':
    main()
