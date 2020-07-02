
from fatcat_openapi_client import *
from fixtures import *


def test_container(api):
    eg = quick_eg(api)

    # all the fields!
    c1 = ContainerEntity(
        name="some container name",
        container_type="journal",
        publisher="some container publisher",
        issnl="1234-567X",
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

    # get redirects (none)
    assert api.get_container_redirects(c2.ident) == []

    # delete
    eg = quick_eg(api)
    api.delete_container(eg.editgroup_id, c2.ident)
    api.accept_editgroup(eg.editgroup_id)
    c2 = api.get_container(c2.ident)
    assert c2.state == "deleted"

def test_container_examples(api):

    c1 = api.get_container('aaaaaaaaaaaaaeiraaaaaaaaam')
    assert c1.name == "PLOS Medicine"
    assert c1.issnl == "1549-1277"

    c2 = api.lookup_container(issnl=c1.issnl)
    assert c1.ident == c2.ident
