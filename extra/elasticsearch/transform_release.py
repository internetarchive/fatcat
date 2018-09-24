#!/usr/bin/env python3

import sys
import json

def transform(m):

    if m['state'] != 'active':
        return None

    # First, the easy ones (direct copy)
    t = dict(
        ident = m['ident'],
        revision = m['revision'],
        title = m['title'],
        release_date = m.get('release_date'),
        release_type = m.get('release_type'),
        release_status = m.get('release_status'),
        language = m.get('language'),
        doi = m.get('doi'),
        pmid = m.get('pmid'),
        pmcid = m.get('pmcid'),
        isbn13 = m.get('isbn13'),
        core_id = m.get('core_id'),
        wikidata_qid = m.get('wikidata_qid')
    )

    container = m.get('container')
    container_is_kept = False
    if container:
        t['publisher'] = countainer.get('publisher')
        t['container_name'] = countainer.get('name')
        t['container_issnl'] = countainer.get('issnl')
        container_extra = container.get('extra')
        if container_extra:
            t['container_is_oa'] = container_extra.get('is_oa')
            container_is_kept = container_extra.get('is_kept', False)
            t['container_is_longtail_oa'] = container_extra.get('is_longtail_oa')
    else:
        t['publisher'] = m.get('publisher')
        t['container_name'] = m.get('container_name')

    files = m.get('files', [])
    t['file_count'] = len(files)
    in_wa = False
    in_ia = False
    t['file_pdf_url'] = None
    for f in files:
        is_pdf = 'pdf' in f.get('mimetype', '')
        for url in f.get('urls', []):
            if url.get('rel', '') == 'webarchive':
                in_wa = True
            if '//web.archive.org/' in url['url'] or '//archive.org/' in url['url']:
                in_ia = True
                if is_pdf:
                    t['file_pdf_url'] = url['url']
            if not t['file_pdf_url'] and is_pdf:
                t['file_pdf_url'] = url['url']
    t['file_in_webarchive'] = in_wa
    t['file_in_ia'] = in_ia

    extra = m.get('extra', dict())
    if extra:
        t['in_shadow'] = extra.get('in_shadow')
    t['any_abstract'] = bool(t.get('abstracts'))
    t['is_kept'] = container_is_kept or extra.get('is_kept', False)

    t['ref_count'] = len(m.get('refs', []))
    t['contrib_count'] = len(m.get('contribs', []))
    contrib_names = []
    for c in m.get('contribs', []):
        if c.get('raw_name'):
            contrib_names.append(c.get('raw_name'))
    t['contrib_names'] = contrib_names
    return t

def run():
    for line in sys.stdin:
        obj = transform(json.loads(line))
        if obj:
            print(json.dumps(obj))

if __name__=="__main__":
    run()
