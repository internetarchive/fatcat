#!/usr/bin/env python3

from typing import Optional
import json, sys


def transform(row: dict) -> Optional[dict]:
    if row.get('mimetype') not in [None, 'application/pdf']:
        return None
    if row.get('state') != 'active':
        return None
    base_url = None
    for url in (row.get('urls') or []):
        url = url['url']
        if '://web.archive.org/' not in url and '://archive.org/' not in url:
            base_url = url
            break
    if not base_url:
        return None
    if not row.get('sha1'):
        return None
    return dict(
        base_url=base_url,
        ingest_type="pdf",
        link_source="fatcat",
        link_source_id=f"file_{row['ident']}",
        ingest_request_source="file-backfill",
        ext_ids=dict(
            sha1=row['sha1'],
        ),
    )


def run():
    for l in sys.stdin:
        if not l.strip():
            continue
        row = json.loads(l)
        request = transform(row)
        if request:
            print(json.dumps(request, sort_keys=True))

if __name__=="__main__":
    run()
