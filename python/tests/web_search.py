import json

import pytest
from fatcat_openapi_client.rest import ApiException
from fixtures import *

from fatcat_web.search import get_elastic_container_random_releases


def test_generic_search(app):

    rv = app.get("/search?q=blood")
    assert rv.status_code == 302
    assert "/release/search" in rv.location

    # file sha1sum
    rv = app.get("/search?q=0262d5351e8e7a0af27af8ceaf7b4e581da085f2")
    assert rv.status_code == 302
    assert "/file/lookup" in rv.location

    # PMCID
    rv = app.get("/search?q=PMC12345")
    assert rv.status_code == 302
    assert "/release/lookup" in rv.location

    # ISSN
    rv = app.get("/search?q=1234-5678")
    assert rv.status_code == 302
    assert "/container/lookup" in rv.location


def test_release_search(app, mocker):

    rv = app.get("/release/search")
    assert rv.status_code == 200

    with open("tests/files/elastic_release_search.json") as f:
        elastic_resp = json.loads(f.read())

    es_raw = mocker.patch("elasticsearch.connection.Urllib3HttpConnection.perform_request")
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp)),
    ]

    rv = app.get("/release/search?q=blood")
    assert rv.status_code == 200
    assert b"Showing" in rv.data
    assert b"Quantum Studies of Acetylene Adsorption on Ice Surface" in rv.data


def test_container_search(app, mocker):

    rv = app.get("/container/search")
    assert rv.status_code == 200

    with open("tests/files/elastic_container_search.json") as f:
        elastic_resp = json.loads(f.read())

    es_raw = mocker.patch("elasticsearch.connection.Urllib3HttpConnection.perform_request")
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp)),
    ]

    rv = app.get("/container/search?q=blood")
    assert rv.status_code == 200
    assert b"Showing" in rv.data
    assert b"European Instructional Course Lectures" in rv.data
    assert b"British Editorial Society of Bone and Joint Surger" in rv.data


def test_random_releases(app, mocker):

    with open("tests/files/elastic_release_search.json") as f:
        elastic_resp = json.loads(f.read())

    es_raw = mocker.patch("elasticsearch.connection.Urllib3HttpConnection.perform_request")
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp)),
    ]

    resp = get_elastic_container_random_releases("123")
    assert len(resp) >= 1


elastic_resp1 = {
    "timed_out": False,
    "aggregations": {"release_ref_count": {"value": 8031459}},
    "hits": {"total": 80578584, "hits": [], "max_score": 0.0},
    "_shards": {"successful": 5, "total": 5, "skipped": 0, "failed": 0},
    "took": 0,
}
elastic_resp2 = {
    "timed_out": False,
    "aggregations": {
        "paper_like": {
            "buckets": {
                "is_oa": {"doc_count": 8031459},
                "in_kbart": {"doc_count": 51594200},
                "in_web": {"doc_count": 10925092},
                "in_web_not_kbart": {"doc_count": 5160359},
            }
        }
    },
    "hits": {"total": 80578584, "hits": [], "max_score": 0.0},
    "_shards": {"successful": 5, "total": 5, "skipped": 0, "failed": 0},
    "took": 0,
}
elastic_resp3 = {
    "timed_out": False,
    "hits": {"total": 80578584, "hits": [], "max_score": 0.0},
    "_shards": {"successful": 5, "total": 5, "skipped": 0, "failed": 0},
    "took": 0,
}


def test_stats(app, mocker):

    es_raw = mocker.patch("elasticsearch.connection.Urllib3HttpConnection.perform_request")
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp1)),
        (200, {}, json.dumps(elastic_resp2)),
        (200, {}, json.dumps(elastic_resp3)),
    ]

    rv = app.get("/stats")
    assert rv.status_code == 200
    assert b"80,578,584" in rv.data


def test_stats_json(app, mocker):

    es_raw = mocker.patch("elasticsearch.connection.Urllib3HttpConnection.perform_request")
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp1)),
        (200, {}, json.dumps(elastic_resp2)),
        (200, {}, json.dumps(elastic_resp3)),
    ]

    rv = app.get("/stats.json")
    assert rv.status_code == 200
    assert rv.json["papers"]["in_kbart"] == 51594200
    assert rv.json["release"]["refs_total"] == 8031459


def test_container_stats(app, mocker):

    elastic_resp = {
        "timed_out": False,
        "aggregations": {
            "container_stats": {
                "buckets": {
                    "is_preserved": {"doc_count": 461939},
                    "in_kbart": {"doc_count": 461939},
                    "in_web": {"doc_count": 2797},
                }
            },
            "preservation": {
                "doc_count_error_upper_bound": 0,
                "sum_other_doc_count": 0,
                "buckets": [
                    {"key": "bright", "doc_count": 4143},
                    {"key": "none", "doc_count": 101},
                    {"key": "dark", "doc_count": 79},
                    {"key": "shadows_only", "doc_count": 5},
                ],
            },
            "release_type": {
                "doc_count_error_upper_bound": 0,
                "sum_other_doc_count": 0,
                "buckets": [
                    {"key": "article-journal", "doc_count": 4324},
                    {"key": "article", "doc_count": 2},
                    {"key": "_unknown", "doc_count": 1},
                    {"key": "editorial", "doc_count": 1},
                ],
            },
        },
        "hits": {"total": 461939, "hits": [], "max_score": 0.0},
        "_shards": {"successful": 5, "total": 5, "skipped": 0, "failed": 0},
        "took": 50,
    }

    es_raw = mocker.patch("elasticsearch.connection.Urllib3HttpConnection.perform_request")
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp)),
        (200, {}, json.dumps(elastic_resp)),
    ]
    rv = app.get("/container/issnl/1234-5678/stats.json")
    assert rv.status_code == 200
    stats = rv.json
    assert isinstance(stats["total"], int)
    assert isinstance(stats["release_type"], dict)
    assert isinstance(stats["preservation"]["total"], int)
    assert isinstance(stats["preservation"]["bright"], int)
    assert isinstance(stats["preservation"]["dark"], int)
    assert isinstance(stats["preservation"]["none"], int)

    rv = app.get("/container/aaaaaaaaaaaaaeiraaaaaaaaam/stats.json")
    assert rv.status_code == 200
    stats = rv.json
    assert isinstance(stats["total"], int)
    assert stats["ident"] == "aaaaaaaaaaaaaeiraaaaaaaaam"
