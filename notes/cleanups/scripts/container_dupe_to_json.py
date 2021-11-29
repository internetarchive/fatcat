#!/usr/bin/env python3

"""
This script can be used to transform duplicate container entity rows into JSON
objects which can be passed to the container entity merger.

It is initially used to de-dupe ISSN-Ls. The script is based on
`file_dupe_to_json.py`.
"""

import json, sys
from typing import Optional

EXTID_TYPE = "issnl"


def print_group(extid, dupe_ids):
    if len(dupe_ids) < 2:
        return
    group = dict(
        entity_type="container",
        primary_id=None,
        duplicate_ids=dupe_ids,
        evidence=dict(
            extid=extid,
            extid_type=EXTID_TYPE,
        ),
    )
    print(json.dumps(group, sort_keys=True))

def run():
    last_extid = None
    dupe_ids = []
    for l in sys.stdin:
        l = l.strip()
        if not l:
            continue
        (row_extid, row_id) = l.split("\t")[0:2]
        if EXTID_TYPE == "issnl":
            assert len(row_extid) == 9
        else:
            raise Exception(f"extid type not supported yet: {EXTID_TYPE}")
        if row_extid == last_extid:
            dupe_ids.append(row_id)
            continue
        elif dupe_ids:
            print_group(last_extid, dupe_ids)
        last_extid = row_extid
        dupe_ids = [row_id]
    if last_extid and dupe_ids:
        print_group(last_extid, dupe_ids)


if __name__=="__main__":
    run()
