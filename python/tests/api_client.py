
import pytest
import fatcat.api_client
from fixtures import *


def test_client_health(api_client):
    assert api_client.health() != None


def test_import_crossref(api_client):
    api_client.import_crossref_file('tests/files/crossref-works.2018-01-21.badsample.json')

    # TODO: use API to check that entities actually created...
