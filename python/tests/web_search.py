
import json
import pytest

from fatcat_web.search import get_elastic_container_random_releases, get_elastic_container_histogram
from fatcat_openapi_client.rest import ApiException
from fixtures import *


def test_release_search(app, mocker):

    with open('tests/files/elastic_release_search.json') as f:
        elastic_resp=json.loads(f.read())

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp)),
    ]

    rv = app.get('/release/search?q=blood')
    assert rv.status_code == 200
    assert b"Showing" in rv.data
    assert b"Quantum Studies of Acetylene Adsorption on Ice Surface" in rv.data

def test_container_search(app, mocker):

    with open('tests/files/elastic_container_search.json') as f:
        elastic_resp=json.loads(f.read())


    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp)),
    ]

    rv = app.get('/container/search?q=blood')
    assert rv.status_code == 200
    assert b"Showing" in rv.data
    assert b"European Instructional Course Lectures" in rv.data
    assert b"British Editorial Society of Bone and Joint Surger" in rv.data

def test_random_releases(app, mocker):

    with open('tests/files/elastic_release_search.json') as f:
        elastic_resp=json.loads(f.read())

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp)),
    ]

    resp = get_elastic_container_random_releases("123")
    assert len(resp) >= 1


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

def test_stats(app, mocker):

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp1)),
        (200, {}, json.dumps(elastic_resp2)),
        (200, {}, json.dumps(elastic_resp3)),
    ]

    rv = app.get('/stats')
    assert rv.status_code == 200
    assert b"80,578,584" in rv.data

def test_stats_json(app, mocker):

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp1)),
        (200, {}, json.dumps(elastic_resp2)),
        (200, {}, json.dumps(elastic_resp3)),
    ]

    rv = app.get('/stats.json')
    assert rv.status_code == 200
    assert rv.json['papers']['in_kbart'] == 51594200
    assert rv.json['release']['refs_total'] == 8031459

def test_container_stats(app, mocker):

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

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp)),
        (200, {}, json.dumps(elastic_resp)),
    ]
    rv = app.get('/container/issnl/1234-5678/stats.json')
    #print(rv.json)
    assert rv.status_code == 200

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaam/stats.json')
    assert rv.status_code == 200

def test_container_coverage(app, mocker):

    elastic_resp1 = {
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

    elastic_resp2 = {
        'took': 294,
        'timed_out': False,
        '_shards': {'total': 5, 'successful': 5, 'skipped': 0, 'failed': 0},
        'hits': {'total': 4327, 'max_score': 0.0, 'hits': []},
        'aggregations': {'year_in_ia': {
            'after_key': {'year': 2020.0, 'in_ia': True},
            'buckets': [
                {'key': {'year': 2004.0, 'in_ia': False}, 'doc_count': 4},
                {'key': {'year': 2004.0, 'in_ia': True}, 'doc_count': 68},
                {'key': {'year': 2005.0, 'in_ia': False}, 'doc_count': 26},
                {'key': {'year': 2005.0, 'in_ia': True}, 'doc_count': 428},
                {'key': {'year': 2006.0, 'in_ia': False}, 'doc_count': 14},
                {'key': {'year': 2006.0, 'in_ia': True}, 'doc_count': 487},
                {'key': {'year': 2007.0, 'in_ia': False}, 'doc_count': 13},
                {'key': {'year': 2007.0, 'in_ia': True}, 'doc_count': 345},
            ],
        }},
    }

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp1)),
    ]

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaam/coverage')
    assert rv.status_code == 200

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp2)),
    ]

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaam/ia_coverage_years.json')
    assert rv.status_code == 200

    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp2)),
    ]

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaam/ia_coverage_years.svg')
    assert rv.status_code == 200
