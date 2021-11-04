#!/usr/bin/env python3

import sys
import json
import base64
from typing import Optional, List

import requests
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry  # pylint: disable=import-error

def requests_retry_session(
    retries: int = 10,
    backoff_factor: int = 3,
    status_forcelist: List[int] = [500, 502, 504],
    session: requests.Session = None,
) -> requests.Session:
    """
    From: https://www.peterbe.com/plog/best-practice-with-retries-with-requests
    """
    session = session or requests.Session()
    retry = Retry(
        total=retries,
        read=retries,
        connect=retries,
        backoff_factor=backoff_factor,
        status_forcelist=status_forcelist,
    )
    adapter = HTTPAdapter(max_retries=retry)
    session.mount("http://", adapter)
    session.mount("https://", adapter)
    return session

def b32_hex(s: str) -> str:
    """
    Converts a base32-encoded SHA-1 checksum into hex-encoded

    base32 checksums are used by, eg, heritrix and in wayback CDX files
    """
    s = s.strip().split()[0].lower()
    if s.startswith("sha1:"):
        s = s[5:]
    if len(s) != 32:
        if len(s) == 40:
            return s
        raise ValueError("not a base-32 encoded SHA-1 hash: {}".format(s))
    return base64.b16encode(base64.b32decode(s.upper())).lower().decode("utf-8")


SANDCRAWLER_POSTGREST_URL = "http://wbgrp-svc506.us.archive.org:3030"

def get_db_cdx(url: str, http_session) -> List[dict]:
    resp = http_session.get(SANDCRAWLER_POSTGREST_URL + "/cdx", params=dict(url="eq." + url))
    resp.raise_for_status()
    rows = resp.json()
    return rows or []

CDX_API_URL = "https://web.archive.org/cdx/search/cdx"

def get_api_cdx(url: str, partial_dt: str, http_session) -> Optional[dict]:

    params = {
        "url": url,
        "from": partial_dt,
        "to": partial_dt,
        "matchType": "exact",
        "output": "json",
        "limit": 20,
        # can't filter status because might be warc/revisit
        #"filter": "statuscode:200",
    }
    resp = http_session.get(CDX_API_URL, params=params)
    resp.raise_for_status()
    rows = resp.json()

    if not rows:
        return None
    #print(rows, file=sys.stderr)
    if len(rows) < 2:
        return None

    for raw in rows[1:]:
        record = dict(
            surt=raw[0],
            datetime=raw[1],
            url=raw[2],
            mimetype=raw[3],
            status_code=raw[4],
            sha1b32=raw[5],
            sha1hex=b32_hex(raw[5]),
        )
        if record['url'] != url:
            # TODO: could allow HTTP/HTTPS fuzzy match
            print("CDX API near match: URL", file=sys.stderr)
            continue
        if not record['datetime'].startswith(partial_dt):
            print(f"CDX API near match: datetime {partial_dt} {record['datetime']}", file=sys.stderr)
            continue
        if record['status_code'] == "200" or (record['status_code'] == '-' and record['mimetype'] == 'warc/revisit'):
            return record
        else:
            print(f"CDX API near match: status  {record['status_code']}", file=sys.stderr)
    return None

def process_file(fe, session) -> dict:
    short_urls = []
    self_urls = dict()
    full_urls = dict()
    status = "unknown"

    for pair in fe['urls']:
        u = pair['url']
        if not '://web.archive.org/web/' in u:
            continue
        seg = u.split('/')
        assert seg[2] == "web.archive.org"
        assert seg[3] == "web"
        assert seg[4].isdigit()
        original_url = "/".join(seg[5:])
        if len(seg[4]) == 12:
            short_urls.append(u)
        elif len(seg[4]) == 14:
            self_urls[original_url] = u
        else:
            print(f"other bogus ts: {seg[4]}", file=sys.stderr)
            return dict(file_entity=fe, full_urls=full_urls, status="fail-bogus-ts")

    if len(short_urls) == 0:
        return dict(file_entity=fe, full_urls=[], status="skip-no-shorts")

    for short in list(set(short_urls)):
        seg = short.split('/')
        ts = seg[4]
        assert len(ts) == 12 and ts.isdigit()
        original_url = '/'.join(seg[5:])

        if original_url in full_urls:
            continue

        if original_url in self_urls:
            full_urls[original_url] = self_urls[original_url]
            status = "success-self"
            continue

        cdx_row_list = get_db_cdx(original_url, http_session=session)
        for cdx_row in cdx_row_list:
            if cdx_row['sha1hex'] == fe['sha1'] and cdx_row['url'] == original_url and cdx_row['datetime'].startswith(ts):
                assert len(cdx_row['datetime']) == 14 and cdx_row['datetime'].isdigit()
                full_urls[original_url] = f"https://web.archive.org/web/{cdx_row['datetime']}/{original_url}"
                status = "success-db"
                break
            else:
                #print(f"cdx DB found, but no match", file=sys.stderr)
                pass
        cdx_row = None

        if original_url in full_urls:
            continue

        cdx_record = get_api_cdx(original_url, partial_dt=ts, http_session=session)
        if cdx_record:
            if cdx_record['sha1hex'] == fe['sha1'] and cdx_record['url'] == original_url and cdx_record['datetime'].startswith(ts):
                assert len(cdx_record['datetime']) == 14 and cdx_record['datetime'].isdigit()
                full_urls[original_url] = f"https://web.archive.org/web/{cdx_record['datetime']}/{original_url}"
                status = "success-api"
                break
            else:
                print(f"cdx API found, but no match", file=sys.stderr)
        else:
            print(f"no CDX API record found: {original_url}", file=sys.stderr)

        if original_url not in full_urls:
            return dict(file_entity=fe, full_urls=full_urls, status="fail-not-found")

    return dict(
        file_entity=fe,
        full_urls=full_urls,
        status=status,
    )

def main():
    session = requests_retry_session()
    session.headers.update({
        "User-Agent": "Mozilla/5.0 fatcat.CdxFixupBot",
    })
    for line in sys.stdin:
        if not line.strip():
            continue
        fe = json.loads(line)
        print(json.dumps(process_file(fe, session=session)))

if __name__=="__main__":
    main()
