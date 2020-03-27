
"""
Helpers for doing elasticsearch queries (used in the web interface; not part of
the formal API)

TODO: ELASTICSEARCH_*_INDEX should probably be factored out and just hard-coded
"""

import datetime
import requests
from flask import abort, flash
from fatcat_web import app

def do_search(index, request, limit=30, offset=0, deep_page_limit=2000):

    # Sanity checks
    if limit > 100:
        limit = 100
    if offset < 0:
        offset = 0
    if offset > deep_page_limit:
        # Avoid deep paging problem.
        offset = deep_page_limit

    request["size"] = int(limit)
    request["from"] = int(offset)
    # print(request)
    resp = requests.get("%s/%s/_search" %
            (app.config['ELASTICSEARCH_BACKEND'], index),
        json=request)

    if resp.status_code == 400:
        print("elasticsearch 400: " + str(resp.content))
        flash("Search query failed to parse; you might need to use quotes.<p><code>{}</code>".format(resp.content))
        abort(resp.status_code)
    elif resp.status_code != 200:
        print("elasticsearch non-200 status code: " + str(resp.status_code))
        print(resp.content)
        abort(resp.status_code)

    content = resp.json()
    results = [h['_source'] for h in content['hits']['hits']]
    for h in results:
        # Handle surrogate strings that elasticsearch returns sometimes,
        # probably due to mangled data processing in some pipeline.
        # "Crimes against Unicode"; production workaround
        for key in h:
            if type(h[key]) is str:
                h[key] = h[key].encode('utf8', 'ignore').decode('utf8')

    return {"count_returned": len(results),
            "count_found": content['hits']['total'],
            "results": results,
            "offset": offset,
            "deep_page_limit": deep_page_limit}


def do_release_search(q, limit=30, fulltext_only=True, offset=0):

    #print("Search hit: " + q)
    if limit > 100:
        # Sanity check
        limit = 100

    # Convert raw DOIs to DOI queries
    if len(q.split()) == 1 and q.startswith("10.") and q.count("/") >= 1:
        q = 'doi:"{}"'.format(q)


    if fulltext_only:
        q += " in_web:true"

    search_request = {
        "query": {
            "query_string": {
                "query": q,
                "default_operator": "AND",
                "analyze_wildcard": True,
                "lenient": True,
                "fields": ["biblio"],
            },
        },
    }

    resp = do_search(app.config['ELASTICSEARCH_RELEASE_INDEX'], search_request, offset=offset)
    for h in resp['results']:
        # Ensure 'contrib_names' is a list, not a single string
        if type(h['contrib_names']) is not list:
            h['contrib_names'] = [h['contrib_names'], ]
        h['contrib_names'] = [name.encode('utf8', 'ignore').decode('utf8') for name in h['contrib_names']]
    resp["query"] = { "q": q }
    resp["limit"] = limit
    return resp


def do_container_search(q, limit=30, offset=0):

    # Convert raw ISSN-L to ISSN-L query
    if len(q.split()) == 1 and len(q) == 9 and q[0:4].isdigit() and q[4] == '-':
        q = 'issnl:"{}"'.format(q)

    search_request = {
        "query": {
            "query_string": {
                "query": q,
                "default_operator": "AND",
                "analyze_wildcard": True,
                "lenient": True,
                "fields": ["biblio"],
            },
        },
    }

    resp = do_search(app.config['ELASTICSEARCH_CONTAINER_INDEX'], search_request, limit=limit, offset=offset)
    resp["query"] = { "q": q }
    resp["limit"] = limit
    return resp

def get_elastic_entity_stats():
    """
    TODO: files, filesets, webcaptures (no schema yet)

    Returns dict:
        changelog: {latest: {index, datetime}}
        release: {total, refs_total}
        papers: {total, in_web, in_oa, in_kbart, in_web_not_kbart}
    """

    stats = {}

    # 2. releases
    #  - total count
    #  - total citation records
    #  - total (paper, chapter, proceeding)
    #  - " with fulltext on web
    #  - " open access
    #  - " not in KBART, in IA
    #
    # Can do the above with two queries:
    #  - all releases, aggregate count and sum(ref_count)
    #  - in-scope works, aggregate count by (fulltext, OA, kbart/ia)

    # 2a. release totals
    query = {
        "size": 0,
        "aggs": {
            "release_ref_count": { "sum": { "field": "ref_count" } }
        }
    }
    resp = requests.get(
        "{}/fatcat_release/_search".format(app.config['ELASTICSEARCH_BACKEND']),
        json=query,
        params=dict(request_cache="true"))
    # TODO: abort()
    resp.raise_for_status()
    resp = resp.json()
    stats['release'] = {
        "total": resp['hits']['total'],
        "refs_total": int(resp['aggregations']['release_ref_count']['value']),
    }

    # 2b. paper counts
    query = {
        "size": 0,
        "query": {
            "terms": { "release_type": [
                # "chapter", "thesis",
                "article-journal", "paper-conference",
            ] } },
        "aggs": { "paper_like": { "filters": { "filters": {
                "in_web": { "term": { "in_web": "true" } },
                "is_oa": { "term": { "is_oa": "true" } },
                "in_kbart": { "term": { "in_kbart": "true" } },
                "in_web_not_kbart": { "bool": { "filter": [
                        { "term": { "in_web": "true" } },
                        { "term": { "in_kbart": "false" } }
                ]}}
        }}}}
    }
    resp = requests.get(
        "{}/fatcat_release/_search".format(app.config['ELASTICSEARCH_BACKEND']),
        json=query,
        params=dict(request_cache="true"))
    # TODO: abort()
    resp.raise_for_status()
    resp = resp.json()
    buckets = resp['aggregations']['paper_like']['buckets']
    stats['papers'] = {
        'total': resp['hits']['total'],
        'in_web': buckets['in_web']['doc_count'],
        'is_oa': buckets['is_oa']['doc_count'],
        'in_kbart': buckets['in_kbart']['doc_count'],
        'in_web_not_kbart': buckets['in_web_not_kbart']['doc_count'],
    }

    # 3. containers
    #   => total count
    query = {
        "size": 0,
    }
    resp = requests.get(
        "{}/fatcat_container/_search".format(app.config['ELASTICSEARCH_BACKEND']),
        json=query,
        params=dict(request_cache="true"))
    # TODO: abort()
    resp.raise_for_status()
    resp = resp.json()
    stats['container'] = {
        "total": resp['hits']['total'],
    }

    return stats

def get_elastic_container_stats(ident, issnl=None):
    """
    Returns dict:
        ident
        issnl (optional)
        total
        in_web
        in_kbart
        preserved
    """

    query = {
        "size": 0,
        "query": {
            "term": { "container_id": ident }
        },
        "aggs": { "container_stats": { "filters": { "filters": {
                "in_web": { "term": { "in_web": "true" } },
                "in_kbart": { "term": { "in_kbart": "true" } },
                "is_preserved": { "term": { "is_preserved": "true" } },
        }}}}
    }
    resp = requests.get(
        "{}/fatcat_release/_search".format(app.config['ELASTICSEARCH_BACKEND']),
        json=query,
        params=dict(request_cache="true"))
    # TODO: abort()
    #print(resp.json())
    resp.raise_for_status()
    resp = resp.json()
    buckets = resp['aggregations']['container_stats']['buckets']
    stats = {
        'ident': ident,
        'issnl': issnl,
        'total': resp['hits']['total'],
        'in_web': buckets['in_web']['doc_count'],
        'in_kbart': buckets['in_kbart']['doc_count'],
        'is_preserved': buckets['is_preserved']['doc_count'],
    }

    return stats

def get_elastic_container_random_releases(ident, limit=5):
    """
    Returns a list of releases from the container.
    """

    assert limit > 0 and limit <= 100

    query = {
        "size": int(limit),
        "sort": [
            { "in_web": {"order": "desc"} },
            { "release_date": {"order": "desc"} },
        ],
        "query": {
            "bool": {
                "must": [
                    { "term": { "container_id": ident } },
                    { "range": { "release_year": { "lte": datetime.datetime.today().year } } },
                ],
            },
        },
    }
    resp = requests.get(
        "{}/fatcat_release/_search".format(app.config['ELASTICSEARCH_BACKEND']),
        json=query,
        params=dict(request_cache="true"))
    # TODO: abort()
    #print(resp.json())
    resp.raise_for_status()
    resp = resp.json()
    #print(resp)
    hits = [h['_source'] for h in resp['hits']['hits']]
    for h in hits:
        # Handle surrogate strings that elasticsearch returns sometimes,
        # probably due to mangled data processing in some pipeline.
        # "Crimes against Unicode"; production workaround
        for key in h:
            if type(h[key]) is str:
                h[key] = h[key].encode('utf8', 'ignore').decode('utf8')

    return hits

def get_elastic_container_histogram(ident):
    """
    Fetches a stacked histogram of 

    Filters to the past 500 years (at most), or about 1000 values.

    Returns a list of tuples:
        (year, in_ia, count)
    """

    query = {
        "aggs": {
            "year_in_ia": {
                "composite": {
                    "size": 1000,
                    "sources": [
                        {"year": {
                            "histogram": {
                                "field": "release_year",
                                "interval": 1,
                        }}},
                        {"in_ia": {
                            "terms": {
                                "field": "in_ia",
                        }}},
                    ],
                },
            },
        },
        "size": 0,
        "query": {
            "bool": {
                "must": [{
                    "range": {
                        "release_year": {
                            "gte": datetime.datetime.today().year - 499,
                            "lte": datetime.datetime.today().year,
                        }
                    }
                }],
                "filter": [{
                    "bool": {
                        "should": [{
                            "match": {
                                "container_id": ident
                            }
                        }],
                        "minimum_should_match": 1,
                    },
                }],
            }
        }
    }
    resp = requests.get(
        "{}/fatcat_release/_search".format(app.config['ELASTICSEARCH_BACKEND']),
        json=query,
        params=dict(request_cache="true"))
    resp.raise_for_status()
    # TODO: abort()
    resp = resp.json()
    #print(resp)
    vals = [(h['key']['year'], h['key']['in_ia'], h['doc_count'])
            for h in resp['aggregations']['year_in_ia']['buckets']]
    vals = sorted(vals)
    return vals
