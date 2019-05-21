
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
        subtitle="son of some title",
        original_title="оригинальное название",
        release_type="post-weblog",
        release_stage="submitted",
        release_date=datetime.datetime.utcnow().date(),
        release_year=2015,
        withdrawn_status="withdrawn",
        withdrawn_year=2017,
        withdrawn_date="2017-04-10",
        ext_ids=ReleaseExtIds(
            doi="10.5555/12345678",
            pmid="12345",
            pmcid="PMC4321",
            wikidata_qid="Q1234",
            isbn13="978-3-16-148410-0",
            core="187348",
            arxiv="aslkdjfh",
            jstor="8328424",
            mag="9439328",
            ark="ark:/12025/654xz321",
        ),
        volume="84",
        number="RFC1234",
        version="v4",
        issue="XII",
        pages="4-99",
        publisher="some publisher",
        language="en",
        license_slug="CC-0",
        contribs=[
            ReleaseContrib(
                given_name="Paul",
                surname="Otlet"),
            ReleaseContrib(
                raw_name="Cindy Sherman",
                given_name="Cindy",
                surname="Sherman"),
            ReleaseContrib(
                raw_name="Andy Warhol"),
        ],
        refs=[],
        abstracts=[
            ReleaseAbstract(
                content="this is some abstract",
                mimetype="text/plain",
                lang="en"),
            ReleaseAbstract(
                content="this is some other abstract",
                mimetype="text/plain",
                lang="de"),
        ],
        extra=dict(a=1, b=2),
        edit_extra=dict(test_key="releases rule"),
    )
    r1.bogus = "asdf"

    r1edit = api.create_release(eg.editgroup_id, r1)
    assert r1edit.extra == r1.edit_extra
    api.accept_editgroup(eg.editgroup_id)
    r2 = api.get_release(r1edit.ident)

    # get revision
    r2_rev = api.get_release_revision(r1edit.revision)
    assert r1edit.revision == r2_rev.revision
    assert r2.revision == r2_rev.revision
    assert r2.title == r2_rev.title

    # check that fields match
    assert r1.title == r2.title
    assert r1.subtitle == r2.subtitle
    assert r1.original_title == r2.original_title
    assert r1.release_type == r2.release_type
    assert r1.release_stage == r2.release_stage
    assert r1.release_date == r2.release_date
    assert r1.release_year == r2.release_year
    assert r1.withdrawn_status == r2.withdrawn_status
    assert str(r1.withdrawn_date) == str(r2.withdrawn_date)
    assert r1.withdrawn_year == r2.withdrawn_year
    assert r1.ext_ids.doi == r2.ext_ids.doi
    assert r1.ext_ids.pmid == r2.ext_ids.pmid
    assert r1.ext_ids.pmcid == r2.ext_ids.pmcid
    assert r1.ext_ids.wikidata_qid == r2.ext_ids.wikidata_qid
    assert r1.ext_ids.isbn13 == r2.ext_ids.isbn13
    assert r1.ext_ids.core == r2.ext_ids.core
    assert r1.ext_ids.arxiv == r2.ext_ids.arxiv
    assert r1.ext_ids.jstor == r2.ext_ids.jstor
    assert r1.ext_ids.ark == r2.ext_ids.ark
    assert r1.ext_ids.mag == r2.ext_ids.mag
    assert r1.number == r2.number
    assert r1.version == r2.version
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

    # get redirects (none)
    assert api.get_release_redirects(r2.ident) == []
    
    # delete
    eg = quick_eg(api)
    api.delete_release(eg.editgroup_id, r2.ident)
    api.accept_editgroup(eg.editgroup_id)
    r2 = api.get_release(r2.ident)
    assert r2.state == "deleted"

def test_release_examples(api):

    api.lookup_release(pmid='54321')
    api.lookup_release(doi='10.123/abc')
    api.lookup_release(isbn13='978-3-16-148410-0')
    api.lookup_release(arxiv='1905.03769v1')
    api.lookup_release(jstor='1819117828')
    api.lookup_release(ark='ark:/13030/m53r5pzm')
    api.lookup_release(mag='992489213')

    # failed lookup exception type
    try:
        api.lookup_release(pmid='5432100')
    except fatcat_client.rest.ApiException as ae:
        assert ae.status == 404
        assert "DatabaseRowNotFound" in ae.body

    # failed lookup formatting
    try:
        api.lookup_release(doi='blah')
    except fatcat_client.rest.ApiException as ae:
        assert ae.status == 400
        assert "MalformedExternalId" in ae.body

    r1 = api.get_release('aaaaaaaaaaaaarceaaaaaaaaai')
    assert r1.title.startswith("A bigger example")
    assert len(r1.refs) == 5
    assert r1.contribs[14].role == "editor"
    assert r1.abstracts[0].mimetype == "application/xml+jats"

    api.get_release_files(r1.ident)
    api.get_release_filesets(r1.ident)
    api.get_release_webcaptures(r1.ident)

def test_empty_fields(api):

    eg = quick_eg(api)

    r1 = ReleaseEntity(
        title="something",
        contribs=[ReleaseContrib(raw_name="somebody")],
        ext_ids=ReleaseExtIds())
    r1edit = api.create_release(eg.editgroup_id, r1)

    with pytest.raises(fatcat_client.rest.ApiException):
        r2 = ReleaseEntity(title="", ext_ids=ReleaseExtIds())
        api.create_release(eg.editgroup_id, r2)
    with pytest.raises(fatcat_client.rest.ApiException):
        r2 = ReleaseEntity(title="something", contribs=[ReleaseContrib(raw_name="")], ext_ids=ReleaseExtIds())
        api.create_release(eg.editgroup_id, r2)

def test_controlled_vocab(api):

    eg = quick_eg(api)

    r1 = ReleaseEntity(title="something", release_type="journal-thingie", ext_ids=ReleaseExtIds())
    with pytest.raises(fatcat_client.rest.ApiException):
        api.create_release(eg.editgroup_id, r1)
    r1.release_type = "article"
    api.create_release(eg.editgroup_id, r1)

    r2 = ReleaseEntity(title="something else", release_stage="pre-print", ext_ids=ReleaseExtIds())
    with pytest.raises(fatcat_client.rest.ApiException):
        api.create_release(eg.editgroup_id, r2)
    r2.release_stage = "published"
    api.create_release(eg.editgroup_id, r2)

    r3 = ReleaseEntity(title="something else", withdrawn_status="boondogle", ext_ids=ReleaseExtIds())
    with pytest.raises(fatcat_client.rest.ApiException):
        api.create_release(eg.editgroup_id, r3)
    r3.withdrawn_status = "spam"
    api.create_release(eg.editgroup_id, r3)

