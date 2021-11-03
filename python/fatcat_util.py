#!/usr/bin/env python3

"""
TODO:
- roll 'shell' stuff into this command
- create, submit, accept editgroups
- create entity from JSON (?)
"""

import argparse
import sys

from fatcat_tools import authenticated_api, fcid2uuid, uuid2fcid


def run_uuid2fcid(args):
    print(uuid2fcid(args.uuid))

def run_fcid2uuid(args):
    print(fcid2uuid(args.fcid))

def run_editgroup_accept(args):
    args.api.accept_editgroup(args.editgroup_id)

def run_editgroup_submit(args):
    eg = args.api.get_editgroup(args.editgroup_id)
    args.api.update_editgroup(args.editgroup_id, eg, submit=True)

def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument('--fatcat-api-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    subparsers = parser.add_subparsers()

    sub_uuid2fcid = subparsers.add_parser('uuid2fcid',
        help="convert a standard UUID (as string) to fatcat ident format")
    sub_uuid2fcid.set_defaults(func=run_uuid2fcid)
    sub_uuid2fcid.add_argument('uuid',
        help="UUID to transform")

    sub_fcid2uuid = subparsers.add_parser('fcid2uuid',
        help="convert a fatcat ident string to standard UUID format")
    sub_fcid2uuid.set_defaults(func=run_fcid2uuid)
    sub_fcid2uuid.add_argument('fcid',
        help="FCID to transform (into UUID)")

    sub_editgroup_accept = subparsers.add_parser('editgroup-accept',
        help="accept an editgroup (by ident)")
    sub_editgroup_accept.set_defaults(func=run_editgroup_accept)
    sub_editgroup_accept.add_argument('editgroup_id',
        help="editgroup to accept")

    sub_editgroup_submit = subparsers.add_parser('editgroup-submit',
        help="submit an editgroup for review (by ident)")
    sub_editgroup_submit.set_defaults(func=run_editgroup_submit)
    sub_editgroup_submit.add_argument('editgroup_id',
        help="editgroup to submit")

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    args.api = authenticated_api(args.fatcat_api_url)
    args.func(args)

if __name__ == '__main__':
    main()
