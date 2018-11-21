
import requests
from flask import abort
from fatcat_web import app

"""
Helpers for doing elasticsearch queries (used in the web interface; not part of
the formal API)
"""

def do_search(q, limit=50, fulltext_only=True):

    #print("Search hit: " + q)
    if limit > 100:
        # Sanity check
        limit = 100

    if fulltext_only:
        q += " file_in_ia:true"

    search_request = {
        "query": {
            "query_string": {
            "query": q,
            "analyzer": "textIcuSearch",
            "default_operator": "AND",
            "analyze_wildcard": True,
            "lenient": True,
            "fields": ["title^5", "contrib_names^2", "container_title"]
            },
        },
        "size": int(limit),
    }

    #print(search_request)
    resp = requests.get("%s/%s/_search" %
            (app.config['ELASTICSEARCH_BACKEND'], app.config['ELASTICSEARCH_INDEX']),
        json=search_request)

    if resp.status_code != 200:
        print("elasticsearch non-200 status code: " + str(resp.status_code))
        print(resp.content)
        abort(resp.status_code)

    content = resp.json()
    #print(content)
    results = [h['_source'] for h in content['hits']['hits']]
    for h in results:
        # Ensure 'contrib_names' is a list, not a single string
        if type(h['contrib_names']) is not list:
            h['contrib_names'] = [h['contrib_names'], ]
        # Handle surrogate strings that elasticsearch returns sometimes,
        # probably due to mangled data processing in some pipeline.
        # "Crimes against Unicode"; production workaround
        for key in h:
            if type(h[key]) is str:
                h[key] = h[key].encode('utf8', 'ignore').decode('utf8')
        h['contrib_names'] = [name.encode('utf8', 'ignore').decode('utf8') for name in h['contrib_names']]

    found = content['hits']['total']
    return {"query": { "q": q },
            "count_returned": len(results),
            "count_found": found,
            "results": results }
