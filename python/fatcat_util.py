#!/usr/bin/env python3

"""
TODO:
- roll 'shell' stuff into this command
- create, submit, accept editgroups
- create entity from JSON (?)
"""

import sys
import json
import argparse

import fatcat_client
from fatcat_client.rest import ApiException
from fatcat_client import ReleaseEntity, ContainerEntity, ChangelogEntry
from fatcat_tools import uuid2fcid, fcid2uuid, entity_from_json, \
    entity_to_dict, public_api, authenticated_api


def run_uuid2fcid(args):
    print(uuid2fcid(args.uuid))

def run_fcid2uuid(args):
    print(fcid2uuid(args.fcid))

def run_editgroup_accept(args):
    print(fcid2uuid(args.fcid))

def run_editgroup_accept(args):
    args.api.accept_editgroup(args.editgroup_id)

def run_editgroup_submit(args):
    eg = args.api.get_editgroup(args.editgroup_id)
    args.api.update_editgroup(args.editgroup_id, eg, submit=True)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debugging interface")
    parser.add_argument('--host-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    subparsers = parser.add_subparsers()

    sub_uuid2fcid = subparsers.add_parser('uuid2fcid')
    sub_uuid2fcid.set_defaults(func=run_uuid2fcid)
    sub_uuid2fcid.add_argument('uuid',
        help="UUID to transform")

    sub_fcid2uuid = subparsers.add_parser('fcid2uuid')
    sub_fcid2uuid.set_defaults(func=run_fcid2uuid)
    sub_fcid2uuid.add_argument('fcid',
        help="FCID to transform (into UUID)")

    sub_editgroup_accept = subparsers.add_parser('editgroup-accept')
    sub_editgroup_accept.set_defaults(func=run_editgroup_accept)
    sub_editgroup_accept.add_argument('editgroup_id',
        help="editgroup to accept")

    sub_editgroup_submit = subparsers.add_parser('editgroup-submit')
    sub_editgroup_submit.set_defaults(func=run_editgroup_submit)
    sub_editgroup_submit.add_argument('editgroup_id',
        help="editgroup to submit")

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = authenticated_api(args.host_url)
    args.func(args)

if __name__ == '__main__':
    main()
