#!/usr/bin/env python3

"""
Note: this is *not* the tool used to generate "official" metadata dumps; that
tool is written in rust and runs on the production infrastructure for speed.
These scripts are just a demonstration of how the API *could* be scraped
without permission by an third party.
"""

import sys
import json
import argparse
import fatcat_client
from fatcat_client.rest import ApiException
from fatcat_client import ReleaseEntity, ContainerEntity, ChangelogEntry
from fatcat_tools import uuid2fcid, entity_from_json, entity_to_dict, \
    release_to_elasticsearch, container_to_elasticsearch, \
    changelog_to_elasticsearch, public_api


def run_export_releases(args):
    for line in args.ident_file:
        ident = uuid2fcid(line.split()[0])
        release = args.api.get_release(ident=ident, expand="all")
        args.json_output.write(
            json.dumps(entity_to_dict(release), api_client=args.api.api_client) + "\n")

def run_transform_releases(args):
    for line in args.json_input:
        line = line.strip()
        if not line:
            continue
        entity = entity_from_json(line, ReleaseEntity, api_client=args.api.api_client)
        args.json_output.write(
            json.dumps(release_to_elasticsearch(entity)) + '\n')

def run_transform_containers(args):
    for line in args.json_input:
        line = line.strip()
        if not line:
            continue
        entity = entity_from_json(line, ContainerEntity, api_client=args.api.api_client)
        args.json_output.write(
            json.dumps(container_to_elasticsearch(entity)) + '\n')

def run_transform_changelogs(args):
    for line in args.json_input:
        line = line.strip()
        if not line:
            continue
        entity = entity_from_json(line, ChangelogEntry, api_client=args.api.api_client)
        args.json_output.write(
            json.dumps(changelog_to_elasticsearch(entity)) + '\n')

def run_export_changelog(args):
    end = args.end
    if end is None:
        latest = args.api.get_changelog(limit=1)[0]
        end = latest.index

    for i in range(args.start, end):
        entry = args.api.get_changelog_entry(index=i)
        args.json_output.write(
            json.dumps(entity_to_dict(entry, api_client=args.api.api_client)) + "\n")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debugging interface")
    parser.add_argument('--host-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    subparsers = parser.add_subparsers()

    sub_releases = subparsers.add_parser('releases')
    sub_releases.set_defaults(func=run_export_releases)
    sub_releases.add_argument('ident_file',
        help="TSV list of fatcat release idents to dump",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_releases.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))

    sub_transform_releases = subparsers.add_parser('transform-releases')
    sub_transform_releases.set_defaults(func=run_transform_releases)
    sub_transform_releases.add_argument('json_input',
        help="JSON-per-line of release entities",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_transform_releases.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))

    sub_transform_containers = subparsers.add_parser('transform-containers')
    sub_transform_containers.set_defaults(func=run_transform_containers)
    sub_transform_containers.add_argument('json_input',
        help="JSON-per-line of container entities",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_transform_containers.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))

    sub_transform_changelogs = subparsers.add_parser('transform-changelogs')
    sub_transform_changelogs.set_defaults(func=run_transform_changelogs)
    sub_transform_changelogs.add_argument('json_input',
        help="JSON-per-line of changelog entries",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_transform_changelogs.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))

    sub_changelog = subparsers.add_parser('changelog')
    sub_changelog.set_defaults(func=run_export_changelog)
    sub_changelog.add_argument('--start',
        help="index to start dumping at",
        default=1, type=int)
    sub_changelog.add_argument('--end',
        help="index to stop dumping at (else detect most recent)",
        default=None, type=int)
    sub_changelog.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = public_api(args.host_url)
    args.func(args)

if __name__ == '__main__':
    main()
