
"""
Helpers for doing elasticsearch queries (used in the web interface; not part of
the formal API)

TODO: ELASTICSEARCH_*_INDEX should probably be factored out and just hard-coded
"""

import datetime
import requests
from flask import abort, flash
from fatcat_web import app

import elasticsearch
from elasticsearch_dsl import Search, Q

def generic_search_execute(search, limit=30, offset=0, deep_page_limit=2000):

    # Sanity checks
    if limit > 100:
        limit = 100
    if offset < 0:
        offset = 0
    if offset > deep_page_limit:
        # Avoid deep paging problem.
        offset = deep_page_limit

    search = search[int(offset):int(offset)+int(limit)]

    try:
        resp = search.execute()
    except elasticsearch.exceptions.RequestError as e:
        # this is a "user" error
        print("elasticsearch 400: " + str(e.info))
        flash("Search query failed to parse; you might need to use quotes.<p><code>{}: {}</code>".format(e.error, e.info['error']['root_cause'][0]['reason']))
        abort(e.status_code)
    except elasticsearch.exceptions.TransportError as e:
        # all other errors
        print("elasticsearch non-200 status code: {}".format(e.info))
        flash("Elasticsearch error: {}".format(e.error))
        abort(e.status_code)

    # just the dict()
    hits = [h._d_ for h in resp]
    for h in hits:
        # Handle surrogate strings that elasticsearch returns sometimes,
        # probably due to mangled data processing in some pipeline.
        # "Crimes against Unicode"; production workaround
        for key in h:
            if type(h[key]) is str:
                h[key] = h[key].encode('utf8', 'ignore').decode('utf8')

    return {"count_returned": len(hits),
            "count_found": int(resp.hits.total),
            "results": hits,
            "offset": offset,
            "limit": limit,
            "deep_page_limit": deep_page_limit}


def do_release_search(q, limit=30, fulltext_only=True, offset=0):

    # Convert raw DOIs to DOI queries
    if len(q.split()) == 1 and q.startswith("10.") and q.count("/") >= 1:
        q = 'doi:"{}"'.format(q)

    if fulltext_only:
        q += " in_web:true"

    search = Search(using=app.es_client, index=app.config['ELASTICSEARCH_RELEASE_INDEX']) \
        .query(
            'query_string',
            query=q,
            default_operator="AND",
            analyze_wildcard=True,
            lenient=True,
            fields=["biblio"],
        )

    resp = generic_search_execute(search, offset=offset)

    for h in resp['results']:
        print(h)
        # Ensure 'contrib_names' is a list, not a single string
        if type(h['contrib_names']) is not list:
            h['contrib_names'] = [h['contrib_names'], ]
        h['contrib_names'] = [name.encode('utf8', 'ignore').decode('utf8') for name in h['contrib_names']]
    resp["query"] = { "q": q }
    return resp


def do_container_search(q, limit=30, offset=0):

    # Convert raw ISSN-L to ISSN-L query
    if len(q.split()) == 1 and len(q) == 9 and q[0:4].isdigit() and q[4] == '-':
        q = 'issnl:"{}"'.format(q)

    search = Search(using=app.es_client, index=app.config['ELASTICSEARCH_RELEASE_INDEX']) \
        .query(
            'query_string',
            query=q,
            default_operator="AND",
            analyze_wildcard=True,
            lenient=True,
            fields=["biblio"],
        )

    resp = generic_search_execute(search, offset=offset)
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

    # release totals
    search = Search(
        using=app.es_client,
        index=app.config['ELASTICSEARCH_RELEASE_INDEX']) \
        .extra(request_cache=True)
    search.aggs.bucket(
        'release_ref_count',
        'sum',
        field='ref_count',
    )
    search = search[:0]

    # NOTE: not catching exceptions
    resp = search.execute()
    stats['release'] = {
        "total": int(resp.hits.total),
        "refs_total": int(resp.aggregations.release_ref_count.value),
    }

    # paper counts
    search = Search(
        using=app.es_client,
        index=app.config['ELASTICSEARCH_RELEASE_INDEX']) \
        .query(
            'terms',
            release_type=[
                "article-journal",
                "paper-conference",
                # "chapter",
                # "thesis",
            ],
        ) \
        .extra(request_cache=True)
    search.aggs.bucket(
        'paper_like',
        'filters',
        filters={
            "in_web": { "term": { "in_web": "true" } },
            "is_oa": { "term": { "is_oa": "true" } },
            "in_kbart": { "term": { "in_kbart": "true" } },
            "in_web_not_kbart": { "bool": { "filter": [
                { "term": { "in_web": "true" } },
                { "term": { "in_kbart": "false" } },
            ]}},
        }
    )
    search = search[:0]

    # NOTE: not catching exceptions
    resp = search.execute()
    buckets = resp.aggregations.paper_like.buckets
    stats['papers'] = {
        'total': resp.hits.total,
        'in_web': buckets.in_web.doc_count,
        'is_oa': buckets.is_oa.doc_count,
        'in_kbart': buckets.in_kbart.doc_count,
        'in_web_not_kbart': buckets.in_web_not_kbart.doc_count,
    }

    # container counts
    search = Search(
        using=app.es_client,
        index=app.config['ELASTICSEARCH_CONTAINER_INDEX']) \
        .extra(request_cache=True)
    search.aggs.bucket(
        'release_ref_count',
        'sum',
        field='ref_count',
    )
    search = search[:0]

    # NOTE: not catching exceptions
    resp = search.execute()
    stats['container'] = {
        "total": resp.hits.total,
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

    search = Search(using=app.es_client, index=app.conf.ELASTICSEARCH_RELEASE_INDEX) \
        .query('bool',
            must=[
                Q('term', container_id=ident),
                Q('range', release_year={ "lte": datetime.datetime.today().year }),
            ]
        ) \
        .sort('-in_web', '-release_date') \
        .extra(request_cache=True)

    search = search[:int(limit)]

    resp = search.execute()

    hits = [dict(h.source) for h in resp]

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
    Fetches a stacked histogram

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
