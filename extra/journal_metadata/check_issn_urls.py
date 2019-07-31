#!/usr/bin/env python3
"""
Check journal homepage status (live web and wayback)


Takes a tsv filepath or lines on stdin and dumps to stdout. The stdin thing means you can:

    parallel -j100 --bar --pipepart -a urls_to_crawl.tsv ./check_issn_urls.py > url_status.json

Input columns (no header):

    ISSN-L, URL

For each URL, do a request and record, as JSON:

    issnl: passthrough
    url: passthrough
    status_code: initial HTTP crawl status
    terminal_url: final URL (or original if no redirects)
    terminal_status_code: final URL (or original if no redirects)
    terminal_content_type: content type (mimetype)
    platform_software: slug of hosting platform, if detected
    issnl_in_body: whether raw issnl appears in body text
    blocked: whether we think crawler was "blocked"
    gwb_url_success_dt: latest wayback datetime that an HTTP 200 exists
    gwb_terminal_url_success_dt: latest wayback datetime that an HTTP 200 exists

HTTP status will be -1 if domain does not even resolve.
"""

import os
import sys
import json
import requests


def sniff_platform(resp):
    """
    This function would try to figure out what software platform (eg, OJS) the
    site is running.
    TODO: unimplemented
    """
    # these are mostly here to filter out huge platforms and stop sniffing
    domain_map = {
        'jstor.org/': 'jstor',
        'springer.com/': 'springer',
        'springerlink.com/': 'springer',
        'tandfonline.com/': 't_and_f',
        'elsevier.com/': 'elsevier',
        'wiley.com/': 'wiley',
        'sciencedirect.com/': 'elsevier',
        'sagepub.com/': 'sage',
        'hypotheses.org/': 'hypothesis',
        'tandf.co.uk/': 't_and_f',
        'scielo': 'scielo',
    }
    for domain, platform in domain_map.items():
        if domain in resp.url:
            return platform
    if '<meta name="generator" content="Open Journal Systems' in resp.text:
        return "ojs"
    return None

def sniff_blocked(resp):
    """
    This function would try to figure out if we got blocked: soft-block, hard
    block, etc.
    TODO: unimplemented
    """
    if resp.status_code in (403, 420):
        return True
    # JSTOR does this
    if 'Our systems have detected unusual traffic activity from your network. Please complete this reCAPTCHA' in resp.text:
        return True
    if resp.status_code == 416 and 'something about your browser made us think you were a bot' in resp.text:
        return True
    return None

def check_gwb(url, match_type='exact'):
    if '//web.archive.org/' in url:
        return None
    resp = requests.get('https://web.archive.org/cdx/search/cdx', params={
        'url': url,
        'matchType': match_type,
        'limit': -1,
        'filter': 'statuscode:200'
    })
    line = resp.text.strip().split('\n')[0]
    if line:
        dt = line.split()[1]
        int(dt)
        return dt
    else:
        return None
    

def check_url(issnl, url):
    #print("Fetching: %s" % url)
    info = dict(issnl=issnl, url=url)
    try:
        resp = requests.get(url, timeout=30., headers={'User-Agent': 'ia_bot/0.0 (python requests) journal-live-check; contact:info@archive.org'})
    except requests.exceptions.TooManyRedirects:
        info['error'] = 'TooManyRedirects'
        info['terminal_status_code'] = info['status_code'] = -1
        return info
    except requests.exceptions.SSLError:
        info['error'] = 'SSLError'
        info['terminal_status_code'] = info['status_code'] = -1
        return info
    except requests.exceptions.ReadTimeout:
        info['error'] = 'ReadTimeout'
        info['terminal_status_code'] = info['status_code'] = -1
        return info
    except requests.exceptions.ConnectionError:
        info['error'] = 'ConnectionError'
        info['terminal_status_code'] = info['status_code'] = -1
        return info
    except requests.exceptions.ChunkedEncodingError:
        info['error'] = 'ChunkedEncodingError'
        info['terminal_status_code'] = info['status_code'] = -1
        return info

    if resp.history:
        info['status_code'] = resp.history[0].status_code
    else:
        info['status_code'] = resp.status_code

    info['terminal_status_code'] = resp.status_code
    info['terminal_url'] = resp.url
    content_type = resp.headers.get('Content-Type')
    if content_type:
        info['terminal_content_type'] = content_type.split(';')[0]
    info['issnl_in_body'] = bool(issnl in resp.text)
    info['gwb_url_success_dt'] = check_gwb(url, match_type='exact')
    info['gwb_terminal_url_success_dt'] = check_gwb(info['terminal_url'], match_type='exact')
    info['blocked'] = sniff_blocked(resp)
    info['software_platform'] = sniff_platform(resp)
    #info['gwb_host_success_dt'] = check_gwb(url, match_type='host')
    return info

def run(tsvfile):
    for line in tsvfile:
        records = line.split('\t')
        issnl = records[0]
        url = records[1].strip()
        print(json.dumps(check_url(issnl, url)))

if __name__=="__main__":
    if len(sys.argv) != 2:
        f = sys.stdin
    else:
        f = open(sys.argv[1], 'r')
    run(f)
