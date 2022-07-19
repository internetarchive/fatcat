#!/usr/bin/env python3
"""
Transform a transformed, fatcat-like dblp object (JSON) into zero or more
sandcrawler ingest requests.
"""

import argparse
import json
import sys

import urlcanon

DOMAIN_BLOCKLIST = [
    # we crawl some of these directly via extid; others are just catalogs
    "://arxiv.org/",
    "://europepmc.org/",
    #"://hdl.handle.net/",
    "ncbi.nlm.nih.gov/",
    "://doi.org/",
    "zenodo.org/",
    "figshare.com/",
    "://d-nb.info/",
    "://www.base-search.net/",
]


def canon(s):
    parsed = urlcanon.parse_url(s)
    return str(urlcanon.whatwg(parsed))


def transform(obj):
    """
    Transforms from a single object to zero or more ingest requests.
    Returns a list of dicts.
    """

    requests = []
    if not obj["ext_ids"].get("dblp"):
        return requests
    if not obj.get("_dblp_ee_urls"):
        return requests

    for url in obj["_dblp_ee_urls"]:
        skip = False
        for domain in DOMAIN_BLOCKLIST:
            if domain in url:
                skip = True
        if skip:
            continue
        try:
            base_url = canon(url)
        except UnicodeEncodeError:
            continue

        request = {
            "base_url": base_url,
            "ingest_type": "pdf",
            "link_source": "dblp",
            "link_source_id": obj["ext_ids"]["dblp"],
            "ingest_request_source": "dblp",
            "release_stage": obj.get("release_stage") or None,
            "ext_ids": {
                "dblp": obj["ext_ids"]["dblp"],
            },
            "edit_extra": {},
        }
        requests.append(request)

    return requests


def run(args):
    for l in args.json_file:
        if not l.strip():
            continue
        row = json.loads(l)

        requests = transform(row) or []
        for r in requests:
            print("{}".format(json.dumps(r, sort_keys=True)))


def main():
    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument(
        "json_file", help="dblp transformed JSON file to use", type=argparse.FileType("r")
    )
    subparsers = parser.add_subparsers()

    args = parser.parse_args()

    run(args)


if __name__ == "__main__":
    main()
