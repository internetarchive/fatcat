#!/usr/bin/env python3

"""
Utility script for doing bulk conversion/transforms of entity JSON schema to
other formats
"""

import argparse
import json
import sys

import elasticsearch
from fatcat_openapi_client import ChangelogEntry, ContainerEntity, FileEntity, ReleaseEntity

from fatcat_tools import public_api
from fatcat_tools.transforms import (
    changelog_to_elasticsearch,
    citeproc_csl,
    container_to_elasticsearch,
    entity_from_json,
    file_to_elasticsearch,
    release_to_csl,
    release_to_elasticsearch,
)
from fatcat_web.search import get_elastic_container_stats


def run_elasticsearch_releases(args):
    for line in args.json_input:
        line = line.strip()
        if not line:
            continue
        entity = entity_from_json(line, ReleaseEntity, api_client=args.api.api_client)
        if entity.state != 'active':
            continue
        args.json_output.write(
            json.dumps(release_to_elasticsearch(entity)) + '\n')

def run_elasticsearch_containers(args):
    es_client = elasticsearch.Elasticsearch(args.fatcat_elasticsearch_url)
    es_release_index = "fatcat_release"
    for line in args.json_input:
        line = line.strip()
        if not line:
            continue
        entity = entity_from_json(line, ContainerEntity, api_client=args.api.api_client)
        if entity.state != 'active':
            continue

        if args.query_stats:
            es_doc = container_to_elasticsearch(
                entity,
                stats=get_elastic_container_stats(
                    entity.ident,
                    es_client=es_client,
                    es_index=es_release_index,
                    merge_shadows=True,
                ),
            )
        else:
            es_doc = container_to_elasticsearch(entity)

        args.json_output.write(json.dumps(es_doc) + '\n')

def run_elasticsearch_files(args):
    for line in args.json_input:
        line = line.strip()
        if not line:
            continue
        entity = entity_from_json(line, FileEntity, api_client=args.api.api_client)
        if entity.state != 'active':
            continue
        args.json_output.write(
            json.dumps(file_to_elasticsearch(entity)) + '\n')

def run_elasticsearch_changelogs(args):
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
        if entity.state != 'active':
            continue
        csl_json = release_to_csl(entity)
        csl_json['id'] = "release:" + (entity.ident or "unknown")
        out = citeproc_csl(csl_json, args.style, args.html)
        args.json_output.write(out + "\n")

def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument('--fatcat-api-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    parser.add_argument('--fatcat-elasticsearch-url',
        default="http://localhost:9200",
        help="connect to this host/port")
    subparsers = parser.add_subparsers()

    sub_elasticsearch_releases = subparsers.add_parser('elasticsearch-releases',
        help="convert fatcat release JSON schema to elasticsearch release schema")
    sub_elasticsearch_releases.set_defaults(func=run_elasticsearch_releases)
    sub_elasticsearch_releases.add_argument('json_input',
        help="JSON-per-line of release entities",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_elasticsearch_releases.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))

    sub_elasticsearch_containers = subparsers.add_parser('elasticsearch-containers',
        help="convert fatcat container JSON schema to elasticsearch container schema")
    sub_elasticsearch_containers.set_defaults(func=run_elasticsearch_containers)
    sub_elasticsearch_containers.add_argument('json_input',
        help="JSON-per-line of container entities",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_elasticsearch_containers.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))
    sub_elasticsearch_containers.add_argument('--query-stats',
        action='store_true',
        help="whether to query release search index for container stats")

    sub_elasticsearch_files = subparsers.add_parser('elasticsearch-files',
        help="convert fatcat file JSON schema to elasticsearch file schema")
    sub_elasticsearch_files.set_defaults(func=run_elasticsearch_files)
    sub_elasticsearch_files.add_argument('json_input',
        help="JSON-per-line of file entities",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_elasticsearch_files.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))

    sub_elasticsearch_changelogs = subparsers.add_parser('elasticsearch-changelogs',
        help="convert fatcat changelog JSON schema to elasticsearch changelog schema")
    sub_elasticsearch_changelogs.set_defaults(func=run_elasticsearch_changelogs)
    sub_elasticsearch_changelogs.add_argument('json_input',
        help="JSON-per-line of changelog entries",
        default=sys.stdin, type=argparse.FileType('r'))
    sub_elasticsearch_changelogs.add_argument('json_output',
        help="where to send output",
        default=sys.stdout, type=argparse.FileType('w'))

    sub_citeproc_releases = subparsers.add_parser('citeproc-releases',
        help="convert fatcat release schema to any standard citation format using citeproc/CSL")
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

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = public_api(args.fatcat_api_url)
    args.func(args)

if __name__ == '__main__':
    main()
