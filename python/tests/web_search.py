
import json
import pytest
import responses
from fatcat_client.rest import ApiException
from fixtures import *

@responses.activate
def test_release_search(app):

    with open('tests/files/elastic_release_search.json') as f:
        elastic_resp=json.loads(f.read())

    responses.add(responses.GET, 'http://localhost:9200/fatcat_release/_search',
        json=elastic_resp, status=200)

    rv = app.get('/release/search?q=blood')
    assert rv.status_code == 200
    assert b"Showing top " in rv.data

@responses.activate
def test_container_search(app):

    with open('tests/files/elastic_container_search.json') as f:
        elastic_resp=json.loads(f.read())

    responses.add(responses.GET, 'http://localhost:9200/fatcat_container/_search',
        json=elastic_resp, status=200)

    rv = app.get('/container/search?q=blood')
    assert rv.status_code == 200
    assert b"Showing top " in rv.data
    assert b"European Instructional Course Lectures" in rv.data
    assert b"British Editorial Society of Bone and Joint Surger" in rv.data

elastic_resp1 = {
    'timed_out': False,
    'aggregations': {
        'release_ref_count': {'value': 8031459}},
    'hits': {'total': 80578584, 'hits': [], 'max_score': 0.0},
    '_shards': {'successful': 5, 'total': 5, 'skipped': 0, 'failed': 0},
    'took': 0
}
elastic_resp2 = {
    'timed_out': False,
    'aggregations': {
        'paper_like': {'buckets': {
            'is_oa': {'doc_count': 8031459},
            'in_kbart': {'doc_count': 51594200},
            'in_web': {'doc_count': 10925092},
            'in_web_not_kbart': {'doc_count': 5160359}}}},
    'hits': {'total': 80578584, 'hits': [], 'max_score': 0.0},
    '_shards': {'successful': 5, 'total': 5, 'skipped': 0, 'failed': 0},
    'took': 0
}
elastic_resp3 = {
    'timed_out': False,
    'hits': {'total': 80578584, 'hits': [], 'max_score': 0.0},
    '_shards': {'successful': 5, 'total': 5, 'skipped': 0, 'failed': 0},
    'took': 0
}

@responses.activate
def test_stats(app):

    responses.add(responses.GET,
        'http://localhost:9200/fatcat_release/_search?request_cache=true',
        json=elastic_resp1.copy(), status=200)
    responses.add(responses.GET,
        'http://localhost:9200/fatcat_release/_search?request_cache=true',
        json=elastic_resp2.copy(), status=200)
    responses.add(responses.GET,
        'http://localhost:9200/fatcat_container/_search?request_cache=true',
        json=elastic_resp3.copy(), status=200)
    rv = app.get('/stats')
    assert rv.status_code == 200
    # TODO: probe these reponses better

@responses.activate
def test_stats_json(app):

    responses.add(responses.GET,
        'http://localhost:9200/fatcat_release/_search?request_cache=true',
        json=elastic_resp1.copy(), status=200)
    responses.add(responses.GET,
        'http://localhost:9200/fatcat_release/_search?request_cache=true',
        json=elastic_resp2.copy(), status=200)
    responses.add(responses.GET,
        'http://localhost:9200/fatcat_container/_search?request_cache=true',
        json=elastic_resp3.copy(), status=200)
    rv = app.get('/stats.json')
    assert rv.status_code == 200

@responses.activate
def test_container_stats(app):

    elastic_resp = {
        'timed_out': False,
        'aggregations': {
            'container_stats': {'buckets': {
              'is_preserved': {'doc_count': 461939},
              'in_kbart': {'doc_count': 461939},
              'in_web': {'doc_count': 2797}}}},
        'hits': {'total': 461939, 'hits': [], 'max_score': 0.0},
        '_shards': {'successful': 5, 'total': 5, 'skipped': 0, 'failed': 0},
        'took': 50
    }

    responses.add(responses.GET,
        'http://localhost:9200/fatcat_release/_search?request_cache=true',
        json=elastic_resp, status=200)
    rv = app.get('/container/issnl/1234-5678/stats.json')
    assert rv.status_code == 200
    # TODO: probe this reponse better

# TODO: container stats
# TODO: container ISSN-L query
# TODO: release DOI query
# TODO: release fulltext (filter) query
