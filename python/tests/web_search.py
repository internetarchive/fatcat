
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

# TODO: entity stats
# TODO: container stats
# TODO: container ISSN-L query
# TODO: release DOI query
# TODO: release fulltext query
