#!/usr/bin/env python3
"""
Takes a tsv filepath (see extract_issn_urls.py) or lines on stdin and dumps to
stdout.

The stdin thing means you can:

    parallel --bar --pipepart -a road_oa_issn_urls.tsv ./check_issn_urls.py > url_status.tsv

For each URL, do a request and record:

    ISSN, URL (or SURT?), HTTP first status, HTTP final status, final URL

HTTP status will be -1 if domain does not even resolve.

"local HTTP status" is the HTTP status code modulo same-domain redirects. This
is intended to accomodate HTTPS upgrades, changes in web app URL schemes, etc.
Will be the same as HTTP status if redirect is non-local.

TODO: detect domain squating/spam?
"""

import os
import sys
import requests

def check_url(url):
    #print("Fetching: %s" % url)
    try:
        resp = requests.get(url)
    except:
        return (url, "-1", "-1", '-')

    if len(resp.history) > 0:
        first_status = resp.history[0].status_code
    else:
        first_status = resp.status_code
    return map(str, (url, first_status, resp.status_code, resp.url))

def run(tsvfile):
    for line in tsvfile:
        records = line.split('\t')
        issnl = records[0]
        url = records[1].strip()
        print(issnl + '\t' + '\t'.join(check_url(url)))

if __name__=="__main__":
    if len(sys.argv) != 2:
        f = sys.stdin
    else:
        f = open(sys.argv[1], 'r')
    run(f)
