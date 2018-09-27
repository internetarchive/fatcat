#!/usr/bin/env python3

import sys
import json
import datetime

MAX_ABSTRACT_BYTES=4096

def parse_grobid_json(obj):
    
    if not obj.get('title'):
        return None

    release = dict()
    extra = dict()

    if obj.get('abstract') and len(obj.get('abstract')) < MAX_ABSTRACT_BYTES:
        abobj = dict(
            mimetype="text/plain",
            language=None,
            content=obj.get('abstract').strip())
        abstracts = [abobj]
    else:
        abstracts = None

    contribs = []
    for a in obj.get('authors', []):
        c = dict(raw_name=a, role="author")
        contribs.append(c)

    refs = []
    for raw in obj.get('citations', []):
        extra = dict()
        ref = dict()
        ref['key'] = raw.get('id')
        if raw.get('title'):
            ref['title'] = raw['title'].strip()
        if raw.get('date'):
            try:
                year = int(raw['date'].strip()[:4])
                ref['year'] = year
            except:
                pass
        for key in ('volume', 'url', 'issue', 'publisher'):
            if raw.get(key):
                extra[key] = raw[key].strip()
        if raw.get('authors'):
            extra['authors'] = [a['name'] for a in raw['authors']]
        if extra:
            extra = dict(grobid=extra)
        else:
            extra = None
        ref['extra'] = extra
        refs.append(ref)

    release_type = "journal-article"
    release_date = None
    if raw.get('date'):
        # TODO: only returns year, ever? how to handle?
        release_date = datetime.datetime(year=raw['date'], month=1, day=1)

    if raw.get('doi'):
        extra['doi'] = raw['doi']
    if raw['journal'].get('name'):
        extra['container_name'] = raw['journal']['name']
    
    extra['is_longtail_oa'] = True

    # TODO: ISSN/eISSN handling? or just journal name lookup?

    if extra:
        extra = dict(grobid=extra)
    else:
        extra = None

    return dict(
        title=obj['title'].strip(),
        contribs=contribs,
        publisher=obj['journal'].get('publisher'),
        volume=obj['journal'].get('volume'),
        issue=obj['journal'].get('issue'),
        abstracts=abstracts,
        extra=extra)

def run():
    for line in sys.stdin:
        obj = json.loads(line)
        out = parse_grobid_json(obj)
        if out:
            print(out)

if __name__=="__main__":
    run()
