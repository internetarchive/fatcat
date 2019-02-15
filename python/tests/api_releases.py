
import json
import pytest
import datetime
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
from fixtures import *


def test_release(api):

    eg = quick_eg(api)

    # all the fields!
    r1 = ReleaseEntity(
        title="some title",
        original_title="оригинальное название",
        release_type="post-weblog",
        release_status="submitted",
        release_date=datetime.datetime.utcnow().date(),
        release_year=2015,
        doi="10.5555/12345678",
        pmid="12345",
        pmcid="PMC4321",
        wikidata_qid="Q1234",
        isbn13="978-3-16-148410-0",
        core_id="187348",
        arxiv_id="aslkdjfh",
        jstor_id="8328424",
        volume="84",
        issue="XII",
        pages="4-99",
        publisher="some publisher",
        language="en",
        license_slug="CC-0",
        extra=dict(a=1, b=2),
        contribs=[],
        refs=[],
        abstracts=[
            ReleaseEntityAbstracts(
                content="this is some abstract",
                mimetype="text/plain",
                lang="en"),
            ReleaseEntityAbstracts(
                content="this is some other abstract",
                mimetype="text/plain",
                lang="de"),
        ],
    )

    r1edit = api.create_release(r1, editgroup_id=eg.editgroup_id)
    api.accept_editgroup(eg.editgroup_id)
    r2 = api.get_release(r1edit.ident)

    # check that fields match
    assert r1.title == r2.title
    assert r1.original_title == r2.original_title
    assert r1.release_type == r2.release_type
    assert r1.release_date == r2.release_date
    assert r1.release_year == r2.release_year
    assert r1.doi == r2.doi
    assert r1.pmid == r2.pmid
    assert r1.pmcid == r2.pmcid
    assert r1.wikidata_qid == r2.wikidata_qid
    assert r1.isbn13 == r2.isbn13
    assert r1.core_id == r2.core_id
    assert r1.arxiv_id == r2.arxiv_id
    assert r1.jstor_id == r2.jstor_id
    assert r1.volume == r2.volume
    assert r1.issue == r2.issue
    assert r1.pages == r2.pages
    assert r1.publisher == r2.publisher
    assert r1.language == r2.language
    assert r1.license_slug == r2.license_slug
    assert r1.extra == r2.extra

    for i in range(len(r1.abstracts)):
        r1.abstracts[i].content == r2.abstracts[i].content
        r1.abstracts[i].mimetype == r2.abstracts[i].mimetype
        r1.abstracts[i].lang == r2.abstracts[i].lang
    for i in range(len(r1.contribs)):
        r1.contribs[i] == r2.contribs[i]
    for i in range(len(r1.refs)):
        r1.refs[i] == r2.refs[i]

    # expansion
    # TODO: via work
    # lookup
    # TODO: via all; but need to generate random identifiers

def test_release_examples(api):

    api.lookup_release(pmid='54321')
    api.lookup_release(isbn13='978-3-16-148410-0')

    r1 = api.get_release('aaaaaaaaaaaaarceaaaaaaaaai')
    assert r1.title == "bigger example"
    assert len(r1.refs) == 5
    assert r1.contribs[0].role == "editor"
    assert r1.abstracts[0].mimetype == "application/xml+jats"

def test_empty_fields(api):

    eg = quick_eg(api)

    r1 = ReleaseEntity(title="something", contribs=[ReleaseContrib(raw_name="somebody")])
    r1edit = api.create_release(r1, editgroup_id=eg.editgroup_id)

    with pytest.raises(fatcat_client.rest.ApiException):
        r2 = ReleaseEntity(title="")
        api.create_release(r2, editgroup_id=eg.editgroup_id)
    with pytest.raises(fatcat_client.rest.ApiException):
        r2 = ReleaseEntity(title="something", contribs=[ReleaseContrib(raw_name="")])
        api.create_release(r2, editgroup_id=eg.editgroup_id)

def test_controlled_vocab(api):

    eg = quick_eg(api)

    r1 = ReleaseEntity(title="something", release_type="journal-thingie")
    with pytest.raises(fatcat_client.rest.ApiException):
        api.create_release(r1, editgroup_id=eg.editgroup_id)
    r1.release_type = "article"
    api.create_release(r1, editgroup_id=eg.editgroup_id)

    r2 = ReleaseEntity(title="something elase", release_status="pre-print")
    with pytest.raises(fatcat_client.rest.ApiException):
        api.create_release(r2, editgroup_id=eg.editgroup_id)
    r2.release_status = "published"
    api.create_release(r2, editgroup_id=eg.editgroup_id)

