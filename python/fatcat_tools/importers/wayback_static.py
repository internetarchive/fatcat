#!/usr/bin/env python3

"""
Helpers to create Web Capture entities from extracted wayback content.

Works as a stand-alone script (for debugging) or as library routines.
"""

import argparse
import datetime
import hashlib
import json
import subprocess
import sys

import requests
from bs4 import BeautifulSoup
from fatcat_openapi_client import (
    ApiClient,
    Editgroup,
    WebcaptureCdxLine,
    WebcaptureEntity,
    WebcaptureUrl,
)

from .common import b32_hex

CDX_API_BASE = "https://web.archive.org/cdx/search/cdx"
GWB_URL_BASE = "https://web.archive.org/web"
REQ_SESSION = requests.Session()


def parse_wbm_url(url):
    """Takes a wayback machine URL, and returns a tuple:

        (timestamp, datetime, original_url)
    """
    chunks = url.split('/')
    assert len(chunks) >= 6
    assert chunks[2] == 'web.archive.org'
    assert chunks[3] == 'web'
    return (chunks[4],
            parse_wbm_timestamp(chunks[4]),
            '/'.join(chunks[5:]))

def test_parse_wbm_url():
    u = "http://web.archive.org/web/20010712114837/http://www.dlib.org/dlib/june01/reich/06reich.html"
    assert parse_wbm_url(u) == (
        "20010712114837",
        datetime.datetime(2001, 7, 12, 11, 48, 37),
        "http://www.dlib.org/dlib/june01/reich/06reich.html")

def parse_wbm_timestamp(timestamp):
    """
    Takes a complete WBM timestamp string (like "20020327115625") and returns a
    python datetime object (UTC)
    """
    # strip any "im_" or "id_" suffix
    if timestamp.endswith('_'):
        timestamp = timestamp[:-3]
    # inflexible; require the full second-precision timestamp
    assert len(timestamp) == 14
    return datetime.datetime(
        year=int(timestamp[0:4]),
        month=int(timestamp[4:6]),
        day=int(timestamp[6:8]),
        hour=int(timestamp[8:10]),
        minute=int(timestamp[10:12]),
        second=int(timestamp[12:14]))

def test_parse_wbm_timestamp():
    assert parse_wbm_timestamp("20010712114837") == \
        datetime.datetime(2001, 7, 12, 11, 48, 37)

def fetch_wbm(url):
    resp = REQ_SESSION.get(url)
    resp.raise_for_status()
    assert resp.content
    return resp.content

def lookup_cdx(embed_url, verify_hashes=True, cdx_output=None):
    sys.stderr.write(embed_url + "\n")
    assert embed_url.startswith('/web/')
    embed_url = embed_url.split('/')
    timestamp = embed_url[2]
    if timestamp.endswith('_'):
        timestamp = timestamp[:-3]
    url = '/'.join(embed_url[3:])
    #print((timestamp, url))
    resp = REQ_SESSION.get(CDX_API_BASE, params=dict(
        url=url,
        closest=timestamp,
        sort="closest",
        resolveRevisits="true",
        matchType="exact",
        limit=1,
    ))
    resp.raise_for_status()
    #print(resp.url)
    if resp.content:
        hit = resp.content.decode('utf-8').split('\n')[0]
        if cdx_output:
            cdx_output.write(hit + "\n")
        cdx = hit.split(' ')
        cdx = [x if (x and x != '-') else None for x in cdx]
        webcapture_cdx = WebcaptureCdxLine(
            surt=cdx[0],
            timestamp=parse_wbm_timestamp(cdx[1]).isoformat() + "Z",
            url=cdx[2],
            mimetype=cdx[3],
            status_code=(cdx[4] and int(cdx[4])) or None,
            sha1=b32_hex(cdx[5]),
            sha256=None,
        )
        if verify_hashes:
            resp = REQ_SESSION.get(GWB_URL_BASE + "/{}id_/{}".format(
                cdx[1], # raw timestamp
                webcapture_cdx.url))
            resp.raise_for_status()
            assert webcapture_cdx.sha1 == hashlib.sha1(resp.content).digest().hex()
            webcapture_cdx.sha256 = hashlib.sha256(resp.content).digest().hex()
            webcapture_cdx.size = len(resp.content)
        return webcapture_cdx
    else:
        return None

def wayback_url_to_relative(url):
    """
    Wayback URLs can be relative or absolute in rewritten documents. This
    function converts any form of rewritten URL to a relative (to
    web.archive.org) one, or returns None if it isn't a rewritten URL at all.
    """
    if url.startswith('https://web.archive.org/'):
        url = url[23:]
    elif url.startswith('http://web.archive.org/'):
        url = url[22:]

    if url.startswith('/web/'):
        return url
    else:
        return None

def extract_embeds(soup):

    embeds = set()

    # <link href="">
    for tag in soup.find_all('link', href=True):
        if tag['rel'] not in ('stylesheet',):
            continue
        url = wayback_url_to_relative(tag['href'])
        if url:
            embeds.add(url)
    # <img src="">
    for tag in soup.find_all('img', src=True):
        url = wayback_url_to_relative(tag['src'])
        if url:
            embeds.add(url)

    # <script src="">
    for tag in soup.find_all('script', src=True):
        url = wayback_url_to_relative(tag['src'])
        if url:
            embeds.add(url)

    return list(embeds)

def static_wayback_webcapture(wayback_url, cdx_output=None):
    """
    Given a complete wayback machine capture URL, like:

        http://web.archive.org/web/20010712114837/http://www.dlib.org/dlib/june01/reich/06reich.html

    Will return a new ("bare") fatcat webcapture entity python object, with all
    the CDX entries filled in.
    """

    wbm_html = fetch_wbm(wayback_url)
    raw_timestamp, timestamp, original_url = parse_wbm_url(wayback_url)
    #with open(rewritten_path, 'r') as fp:
    #    soup = BeautifulSoup(fp, "lxml")
    soup = BeautifulSoup(wbm_html, "lxml")
    embeds = extract_embeds(soup)
    cdx_obj = lookup_cdx("/web/{}/{}".format(raw_timestamp, original_url),
        cdx_output=cdx_output)
    cdx_list = [cdx_obj]
    for url in embeds:
        cdx_obj = lookup_cdx(url, cdx_output=cdx_output)
        cdx_list.append(cdx_obj)
    archive_urls = [WebcaptureUrl(
        rel="wayback",
        url="https://web.archive.org/web/",
    )]
    wc = WebcaptureEntity(
        cdx=cdx_list,
        timestamp=timestamp.isoformat() + "Z",
        original_url=original_url,
        archive_urls=archive_urls,
        release_ids=None)
    return wc

def auto_wayback_static(api, release_id, wayback_url, editgroup_id=None):
    """
    Returns a tuple: (editgroup_id, edit). If failed, both are None
    """

    raw_timestamp, timestamp, original_url = parse_wbm_url(wayback_url)
    git_rev = subprocess.check_output(
        ["git", "describe", "--always"]).strip().decode('utf-8')

    release = api.get_release(release_id, expand="webcaptures")

    # check for existing webcapture with same parameters
    for wc in release.webcaptures:
        if wc.original_url == original_url and wc.timestamp.date() == timestamp.date():
            # skipping: already existed
            print("release {} already had webcapture {} {}".format(
                release_id, raw_timestamp, original_url))
            return (None, None)

    wc = static_wayback_webcapture(wayback_url)
    assert len(wc.cdx) >= 1
    wc.release_ids = [release_id]
    if not editgroup_id:
        eg = api.create_editgroup(Editgroup(
            description="One-off import of static web content from wayback machine",
            extra=dict(
                git_rev=git_rev,
                agent="fatcat_tools.auto_wayback_static")))
        editgroup_id = eg.editgroup_id
    edit = api.create_webcapture(eg.editgroup_id, wc)
    return (editgroup_id, edit)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--verbose',
        action='store_true',
        help="verbose output")
    parser.add_argument('wayback_url',
        type=str,
        help="URL of wayback capture to extract from")
    parser.add_argument('--json-output',
        type=argparse.FileType('w'), default=sys.stdout,
        help="where to write out webcapture entity (as JSON)")
    parser.add_argument('--cdx-output',
        type=argparse.FileType('w'), default=None,
        help="(optional) file to write out CDX stub")

    args = parser.parse_args()

    # entity-to-JSON code; duplicate of entity_to_dict()
    api_client = ApiClient()
    wc = static_wayback_webcapture(args.wayback_url, cdx_output=args.cdx_output)
    wc_dict = api_client.sanitize_for_serialization(wc)
    print(json.dumps(wc_dict))

if __name__ == '__main__':
    main()
