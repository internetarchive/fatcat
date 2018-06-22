
import requests
from flask import abort
from fatcat import app


def do_search(q, limit=20):

    print("Search hit: " + q)
    if limit > 100:
        # Sanity check
        limit = 100

    search_request = {
        "query": {
            "query_string": {
                "query": q,
                "analyzer": "textIcuSearch",
                "default_operator": "AND",
                "analyze_wildcard": True,
                "lenient": True,
                "auto_generate_phrase_queries": True,
                "default_field": "_all",
            },
        },
        "size": int(limit),
    }

    resp = requests.get("%s/%s/_search" %
            (app.config['ELASTIC_BACKEND'], app.config['ELASTIC_INDEX']),
        json=search_request)

    if resp.status_code != 200:
        print("elasticsearch non-200 status code: " + str(resp.status_code))
        print(resp.content)
        abort(resp.status_code)

    content = resp.json()
    results = [h['_source'] for h in content['hits']['hits']]
    for h in results:
        # Ensure 'authors' is a list, not a single string
        if type(h['authors']) is not list:
            h['authors'] = [h['authors'], ]

    found = content['hits']['total']
    return {"query": { "q": q },
            "count_returned": len(results),
            "count_found": found,
            "results": results }
