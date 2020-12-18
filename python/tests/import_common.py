
import json
import datetime
from typing import Any

import pytest
import elasticsearch
import fatcat_openapi_client
from fatcat_openapi_client import ReleaseEntity, ReleaseExtIds
import fuzzycat.matching

from fatcat_tools.importers import EntityImporter
from fatcat_tools.transforms import entity_to_dict
from fixtures import *


@pytest.fixture(scope="function")
def entity_importer(api, mocker) -> Any:
    es_client = elasticsearch.Elasticsearch("mockbackend")
    mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    yield EntityImporter(api, es_client=es_client)

def test_fuzzy_match_none(entity_importer, mocker) -> None:
    """
    Simple ES-mocked test for "no search results" case
    """

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    es_raw.side_effect = [
        (200, {}, json.dumps(ES_RELEASE_EMPTY_RESP)),
        (200, {}, json.dumps(ES_RELEASE_EMPTY_RESP)),
    ]

    release = ReleaseEntity(
        title="some long title which should not match anything because it is for testing",
        ext_ids=ReleaseExtIds(),
    )

    resp = entity_importer.match_existing_release_fuzzy(release)
    assert resp == None

def test_fuzzy_match_different(entity_importer, mocker) -> None:
    """
    Simple fuzzycat-mocked test for "strong match" case
    """

    r1 = ReleaseEntity(
        title="example title: novel work",
        contribs=[ReleaseContrib(raw_name="robin hood")],
        ext_ids=ReleaseExtIds(doi="10.1234/abcdefg"),
    )
    r2 = ReleaseEntity(
        title="Example Title: Novel Work?",
        contribs=[ReleaseContrib(raw_name="robin hood")],
        ext_ids=ReleaseExtIds(),
    )
    r3 = ReleaseEntity(
        title="entirely different",
        contribs=[ReleaseContrib(raw_name="king tut")],
        ext_ids=ReleaseExtIds(),
    )

    match_raw = mocker.patch('fatcat_tools.importers.common.match_release_fuzzy')
    match_raw.side_effect = [[r3, r2, r3, r2]]
    resp = entity_importer.match_existing_release_fuzzy(r1)
    assert (resp[0], resp[2]) == ("STRONG", r2)

    match_raw.side_effect = [[r2, r2, r3, r1]]
    resp = entity_importer.match_existing_release_fuzzy(r1)
    assert (resp[0], resp[2]) == ("EXACT", r1)

    match_raw.side_effect = [[r3]]
    resp = entity_importer.match_existing_release_fuzzy(r1)
    assert resp == None

    match_raw.side_effect = [[]]
    resp = entity_importer.match_existing_release_fuzzy(r1)
    assert resp == None
