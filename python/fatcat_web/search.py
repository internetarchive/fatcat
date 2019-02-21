
import requests
from flask import abort, flash
from fatcat_web import app

"""
Helpers for doing elasticsearch queries (used in the web interface; not part of
the formal API)

TODO: ELASTICSEARCH_*_INDEX should probably be factored out and just hard-coded
"""


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
    if len(q.split()) == 1 and len(q) == 9 and isdigit(q[0:4]) and q[4] == '-':
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

