
import json
import pytest

from fatcat_openapi_client.rest import ApiException
from fixtures import *


def test_container_coverage(app, mocker):

    elastic_resp1 = {
        'timed_out': False,
        'aggregations': {
            'container_stats': {'buckets': {
              'is_preserved': {'doc_count': 461939},
              'in_kbart': {'doc_count': 461939},
              'in_web': {'doc_count': 2797},
            }},
            'preservation': {
              'buckets': [
                {'key': 'bright', 'doc_count': 444},
                {'key': 'dark', 'doc_count': 111},
              ],
              'sum_other_doc_count': 0,
            },
            'release_type': {
              'buckets': [
                {'key': 'article-journal', 'doc_count': 456},
                {'key': 'book', 'doc_count': 123},
              ],
              'sum_other_doc_count': 0,
            },
        },
        'hits': {'total': 461939, 'hits': [], 'max_score': 0.0},
        '_shards': {'successful': 5, 'total': 5, 'skipped': 0, 'failed': 0},
        'took': 50
    }

    elastic_resp2 = {
        'took': 294,
        'timed_out': False,
        '_shards': {'total': 5, 'successful': 5, 'skipped': 0, 'failed': 0},
        'hits': {'total': 4327, 'max_score': 0.0, 'hits': []},
        'aggregations': {
            'year_in_ia': {
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
            },
        },
    }

    elastic_resp3 = {
        'took': 294,
        'timed_out': False,
        '_shards': {'total': 5, 'successful': 5, 'skipped': 0, 'failed': 0},
        'hits': {'total': 4327, 'max_score': 0.0, 'hits': []},
        'aggregations': {
            'type_preservation': {
              'buckets': [
                {'key': {'release_type': 'article-journal', 'preservation': 'bright'}, 'doc_count': 444},
                {'key': {'release_type': 'book', 'preservation': 'dark'}, 'doc_count': 111},
              ],
              'sum_other_doc_count': 0,
            },
        },
    }

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        # status
        (200, {}, json.dumps(elastic_resp1)),
        # type preservation histogram
        (200, {}, json.dumps(elastic_resp3)),
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

def test_coverage_empty_years(app, mocker):

    elastic_resp = {
        'took': 294,
        'timed_out': False,
        '_shards': {'total': 5, 'successful': 5, 'skipped': 0, 'failed': 0},
        'hits': {'total': 4327, 'max_score': 0.0, 'hits': []},
        'aggregations': {'year_in_ia': {
            'after_key': {'year': 2020.0, 'in_ia': True},
            'buckets': [
            ],
        }},
    }

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp)),
    ]

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaam/ia_coverage_years.svg')
    assert rv.status_code == 200
