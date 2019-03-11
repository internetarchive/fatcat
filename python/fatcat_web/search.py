
"""
Helpers for doing elasticsearch queries (used in the web interface; not part of
the formal API)

TODO: ELASTICSEARCH_*_INDEX should probably be factored out and just hard-coded
"""

import requests
from flask import abort, flash
from fatcat_web import app


def do_search(index, request, limit=30):

    if limit > 100:
        # Sanity check
        limit = 100

    request["size"] = int(limit)
    #print(request)
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
            "results": results }


def do_release_search(q, limit=30, fulltext_only=True):

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
                "fields": ["title^5", "contrib_names^2", "container_title"],
            },
        },
    }

    resp = do_search(app.config['ELASTICSEARCH_RELEASE_INDEX'], search_request)
    for h in resp['results']:
        # Ensure 'contrib_names' is a list, not a single string
        if type(h['contrib_names']) is not list:
            h['contrib_names'] = [h['contrib_names'], ]
        h['contrib_names'] = [name.encode('utf8', 'ignore').decode('utf8') for name in h['contrib_names']]
    resp["query"] = { "q": q }
    return resp


def do_container_search(q, limit=30):

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
                "fields": ["name^5", "publisher"],
            },
        },
    }

    resp = do_search(app.config['ELASTICSEARCH_CONTAINER_INDEX'], search_request, limit=limit)
    resp["query"] = { "q": q }
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

def get_elastic_container_stats(issnl):
    """
    TODO: container_id, not issnl

    Returns dict:
        total
        in_web
        preserved
    """

    query = {
        "size": 0,
        "query": {
            "term": { "container_issnl": issnl }
        },
        "aggs": { "container_stats": { "filters": { "filters": {
                "in_web": { "term": { "in_web": "true" } },
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
        'issnl': issnl,
        'total': resp['hits']['total'],
        'in_web': buckets['in_web']['doc_count'],
        'is_preserved': buckets['is_preserved']['doc_count'],
    }

    return stats
