#!/usr/bin/env python3

"""
Run this script and pass a list of filenames (with or without .html) to stdin,
and this will output JSON objects to stdout. Eg:

    fd .html | ./dblp_html_extract.py | pv -l > dblp_container_meta.json

Requires virtualenv with selectolax.
"""

import sys
import json

from selectolax.parser import HTMLParser


def parse_html(path: str) -> dict:
    """
    Parses from HTML:

    - key
    - title
    - issns (list)
    - wikidata_qid
    - homepage_url
    - acronym (?)

    TODO: publisher?
    """
    key = path.replace('.html', '')
    if not len(key.split('/')) == 2:
        print(key, file=sys.stderr)
        return {}
    meta = dict(dblp_prefix=key, issns=[])

    try:
        with open(path, 'r') as html_file:
            doc = HTMLParser(html_file.read())
    except FileNotFoundError:
        return {}

    elem = doc.css_first('header#headline h1')
    if elem and elem.text():
        meta['title'] = elem.text()
        if meta['title'].endswith(')') and meta['title'].count('(') == 1:
            meta['acronym'] = meta['title'].split('(')[-1][:-1]
            meta['title'] = meta['title'].split('(')[0].strip()

    # <a href="https://portal.issn.org/resource/issn/2624-8212" itemprop="sameAs">
    # <a href="https://www.wikidata.org/entity/Q15753736" itemprop="sameAs">
    elems = doc.css('header#headline a[itemprop="sameAs"]') or []
    for elem in elems:
        if not elem.attributes.get('href'):
            continue
        url = elem.attributes['href']
        if "://portal.issn.org/" in url:
            issn = url.split('/')[-1].strip()
            if len(issn) == 9:
                meta['issns'].append(issn)
            else:
                print(issn, file=sys.stderr)
        elif "://www.wikidata.org/entity/Q" in url:
            meta['wikidata_qid'] = url.split('/')[-1]
            assert 'Q' in meta['wikidata_qid']

    # <a href="https://journals.sagepub.com/home/hfs" itemprop="url"><img alt="" src="https://dblp.org/img/home.dark.16x16.png" class="icon" />web page @ sagepub.com</a>
    elem = doc.css_first('header#headline a[itemprop="url"]')
    if elem and elem.attributes.get('href'):
        meta['homepage_url'] = elem.attributes['href']
            
    return meta

def run() -> None:
    for path in sys.stdin:
        path = path.strip()
        if not path:
            continue
        if not path.endswith(".html"):
            path += ".html"
        obj = parse_html(path)
        if obj:
            print(json.dumps(obj, sort_keys=True))

if __name__=='__main__':
    run()
