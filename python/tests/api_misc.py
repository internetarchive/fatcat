
import json
import pytest
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
from fixtures import *


def test_lookups(api):

    api.lookup_creator(orcid='0000-0003-3118-6859')
    api.lookup_container(issnl='1549-1277')
    api.lookup_file(sha256='ffc1005680cb620eec4c913437dfabbf311b535cfe16cbaeb2faec1f92afc362')
    api.lookup_release(pmid='54321')
    api.lookup_release(isbn13='978-3-16-148410-0')

def test_lookup_hide_extend(api):

    r = api.lookup_release(doi='10.1371/journal.pmed.0020124')
    assert len(r.refs) >= 2
    assert r.files is None
    assert r.container is None
    assert len(r.container_id) > 10
    assert r.abstracts == []

    r = api.lookup_release(doi='10.1371/journal.pmed.0020124', expand='files', hide='refs,abstracts')
    assert r.refs is None
    assert len(r.files[0].sha1) == 40
    assert r.container is None
    assert r.abstracts is None

    r = api.lookup_release(doi='10.1371/journal.pmed.0020124', expand='container,abstracts')
    assert len(r.refs) >= 2
    assert r.files is None
    assert r.container.issnl
    assert r.abstracts == []
