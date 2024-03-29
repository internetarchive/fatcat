import pytest
from fatcat_openapi_client import *
from fixtures import *


def test_container(api):
    eg = quick_eg(api)

    # all the fields!
    c1 = ContainerEntity(
        name="some container name",
        container_type="journal",
        publisher="some container publisher",
        publication_status="active",
        issnl="1234-567X",
        issne="1230-0000",
        issnp="1234-0001",
        wikidata_qid="Q954248",
        extra=dict(a=1, b=2),
        edit_extra=dict(test_key="containers rule"),
    )

    c1edit = api.create_container(eg.editgroup_id, c1)
    assert c1edit.extra == c1.edit_extra
    api.accept_editgroup(eg.editgroup_id)
    c2 = api.get_container(c1edit.ident)

    # check that fields match
    assert c1.name == c2.name
    assert c1.container_type == c2.container_type
    assert c1.publisher == c2.publisher
    assert c1.issnl == c2.issnl
    assert c1.wikidata_qid == c2.wikidata_qid
    assert c1.extra == c2.extra

    # get revision
    c2_rev = api.get_container_revision(c1edit.revision)
    assert c1edit.revision == c2_rev.revision
    assert c2.revision == c2_rev.revision
    assert c2.name == c2_rev.name

    # lookups
    tmp = api.lookup_container(issnl=c1.issnl)
    assert tmp.revision == c2.revision
    tmp = api.lookup_container(issne=c1.issne)
    assert tmp.revision == c2.revision
    tmp = api.lookup_container(issnp=c1.issnp)
    assert tmp.revision == c2.revision
    tmp = api.lookup_container(issn=c1.issnp)
    assert tmp.revision == c2.revision
    with pytest.raises(fatcat_openapi_client.rest.ApiException):
        api.lookup_container(issne=c1.issnp)
    with pytest.raises(fatcat_openapi_client.rest.ApiException):
        api.lookup_container(issnp=c1.issne)

    # get redirects (none)
    assert api.get_container_redirects(c2.ident) == []

    # delete
    eg = quick_eg(api)
    api.delete_container(eg.editgroup_id, c2.ident)
    api.accept_editgroup(eg.editgroup_id)
    c2 = api.get_container(c2.ident)
    assert c2.state == "deleted"


def test_container_bad_idents(api):

    # all the fields!
    c1 = ContainerEntity(
        name="some container name",
        container_type="journal",
        publisher="some container publisher",
        wikidata_qid="Q954248",
        extra=dict(a=1, b=2),
        edit_extra=dict(test_key="containers rule"),
    )

    with pytest.raises(ValueError):
        c1.issnl = "1234-123 "

    with pytest.raises(ValueError):
        c1.issne = "asdf-hhhh"


def test_container_examples(api):

    c1 = api.get_container("aaaaaaaaaaaaaeiraaaaaaaaam")
    assert c1.name == "PLOS Medicine"
    assert c1.issnl == "1549-1277"
    assert c1.issne == "1549-1676"
    assert c1.issnp == "1549-1277"
    assert c1.publication_status == "active"

    c2 = api.lookup_container(issnl=c1.issnl)
    assert c1.ident == c2.ident

    c3 = api.lookup_container(issnp=c1.issnp)
    assert c1.ident == c3.ident

    c4 = api.lookup_container(issn=c1.issnp)
    assert c1.ident == c4.ident
