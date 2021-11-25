#!/usr/bin/env python3

"""
This script can be used to transform duplicate file entity hash export rows
into JSON objects which can be passed to the file entity merger.

The input is expected to be a TSV with two columns: a hash value in the first
column, and a fatcat file entity ident (in UUID format, not "fatcat ident"
encoded) in the second column. The rows are assumed to be sorted by hash value
(the first column), and duplicate values (same hash, differing UUID) are
contiguous.

File hashes aren't really "external identifiers" (ext_id), but we treat them as
such here.

Script is pretty simple, should be possible to copy and reuse for release,
container, creator entity duplicates.
"""

import json, sys
from typing import Optional
import base64, uuid

EXTID_TYPE = "sha1"

def uuid2fcid(s: str) -> str:
    """
    Converts a uuid.UUID object to a fatcat identifier (base32 encoded string)
    """     
    raw = uuid.UUID(s).bytes
    return base64.b32encode(raw)[:26].lower().decode("utf-8")

def print_group(extid, dupe_ids):
    if len(dupe_ids) < 2:
        return
    group = dict(
        entity_type="file",
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
        (row_extid, row_uuid) = l.split("\t")[0:2]
        if EXTID_TYPE == "sha1":
            assert len(row_extid) == 40
        else:
            raise Exception(f"extid type not supported yet: {EXTID_TYPE}")
        row_id = uuid2fcid(row_uuid)
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
