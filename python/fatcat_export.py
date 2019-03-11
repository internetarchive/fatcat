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

from citeproc import CitationStylesStyle, CitationStylesBibliography
from citeproc import Citation, CitationItem
from citeproc import formatter
from citeproc.source.json import CiteProcJSON
from citeproc_styles import get_style_filepath

import fatcat_client
from fatcat_client.rest import ApiException
from fatcat_client import ReleaseEntity, ContainerEntity, ChangelogEntry
from fatcat_tools import uuid2fcid, entity_from_json, entity_to_dict, \
    release_to_elasticsearch, container_to_elasticsearch, \
    changelog_to_elasticsearch, public_api, release_to_csl


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

def run_citeproc_releases(args):
    for line in args.json_input:
        line = line.strip()
        if not line:
            continue
        entity = entity_from_json(line, ReleaseEntity, api_client=args.api.api_client)
        csl_json = release_to_csl(entity)
        # XXX:
        csl_json['id'] = "release:" + (entity.ident or "unknown")
        if args.style == "csl-json":
            args.json_output.write(json.dumps(csl_json) + "\n")
            continue
        bib_src = CiteProcJSON([csl_json])
        form = formatter.plain
        if args.html:
            form = formatter.html
        style_path = get_style_filepath(args.style)
        bib_style = CitationStylesStyle(style_path, validate=False)
        bib = CitationStylesBibliography(bib_style, bib_src, form)
        bib.register(Citation([CitationItem(csl_json['id'])]))
        # XXX:
        #args.json_output.write(
        #    json.dumps(release_to_csl(entity)) + '\n')
        lines = bib.bibliography()[0]
        if args.style == "bibtex":
            for l in lines:
                if l.startswith(" @"):
                    args.json_output.write("\n@")
                elif l.startswith(" "):
                    #print("line: START|{}|END".format(l))
                    args.json_output.write("\n  " + l)
                else:
                    args.json_output.write(l)
        else:
            args.json_output.write(''.join(lines) + "\n")
        print()

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

    sub_citeproc_releases = subparsers.add_parser('citeproc-releases')
    sub_citeproc_releases.set_defaults(func=run_citeproc_releases)
    sub_citeproc_releases.add_argument('json_input',
        help="JSON-per-line of release entities",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_citeproc_releases.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))
    sub_citeproc_releases.add_argument('--style',
        help="citation style to output",
        default='csl-json')
    sub_citeproc_releases.add_argument('--html',
        action='store_true',
        help="output HTML, not plain text")

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
