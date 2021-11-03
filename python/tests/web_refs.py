
import json

import pytest
from fatcat_openapi_client.rest import ApiException
from fixtures import *

from fatcat_web.search import get_elastic_container_random_releases


def test_basic_refs(app, mocker):

    with open('tests/files/elastic_refs_in_release.json') as f:
        elastic_resp_in = json.loads(f.read())
    with open('tests/files/elastic_refs_out_release.json') as f:
        elastic_resp_out = json.loads(f.read())
    with open('tests/files/elastic_empty.json') as f:
        elastic_resp_empty = json.loads(f.read())

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(elastic_resp_in)),
        (200, {}, json.dumps(elastic_resp_in)),
        (200, {}, json.dumps(elastic_resp_empty)),
        (200, {}, json.dumps(elastic_resp_out)),
        (200, {}, json.dumps(elastic_resp_out)),
        (200, {}, json.dumps(elastic_resp_empty)),
    ]

    # render refs-in
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/refs-in')
    assert rv.status_code == 200
    assert b"Why Most Published Research Findings Are False" in rv.data

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/refs-in.json')
    assert rv.status_code == 200

    # empty (in)
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/refs-in')
    assert rv.status_code == 200
    assert b"No References Found" in rv.data

    # render refs-out
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/refs-out')
    assert rv.status_code == 200
    assert b"Why Most Published Research Findings Are False" in rv.data

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/refs-out.json')
    assert rv.status_code == 200

    # empty (out)
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/refs-out')
    assert rv.status_code == 200
    assert b"No References Found" in rv.data
