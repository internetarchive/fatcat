
from fatcat_openapi_client import *
from fixtures import *


def test_creators(api):
    eg = quick_eg(api)

    # all the fields!
    c1 = CreatorEntity(
        display_name="Emma Smith",
        given_name="emma",
        surname="smith",
        orcid="0000-0002-1825-0097",
        wikidata_qid="Q9542248",
        extra=dict(a=1, b=5),
        edit_extra=dict(test_key="creators rule"),
    )

    c1edit = api.create_creator(eg.editgroup_id, c1)
    assert c1edit.extra == c1.edit_extra
    api.accept_editgroup(eg.editgroup_id)
    c2 = api.get_creator(c1edit.ident)

    # check that fields match
    assert c1.display_name == c2.display_name
    assert c1.given_name == c2.given_name
    assert c1.surname == c2.surname
    assert c1.orcid == c2.orcid
    assert c1.wikidata_qid == c2.wikidata_qid
    assert c1.extra == c2.extra

    # get revision
    c2_rev = api.get_creator_revision(c1edit.revision)
    assert c1edit.revision == c2_rev.revision
    assert c2.revision == c2_rev.revision
    assert c2.display_name == c2_rev.display_name

    # get redirects (none)
    assert api.get_creator_redirects(c2.ident) == []

    # also try a batch insert
    c3_eg = api.create_creator_auto_batch(CreatorAutoBatch(Editgroup(), [c1]))
    c3edit = c3_eg.edits.creators[0]
    assert c3edit.extra == c1.edit_extra
    c3 = api.get_creator(c3edit.ident)

    assert c1.display_name == c3.display_name
    assert c1.extra == c3.extra

    # delete
    eg = quick_eg(api)
    api.delete_creator(eg.editgroup_id, c2.ident)
    api.delete_creator(eg.editgroup_id, c3.ident)
    api.accept_editgroup(eg.editgroup_id)
    c2 = api.get_creator(c2.ident)
    assert c2.state == "deleted"

# TODO: test expansion of new creator/release pair (release get)?

def test_creators_examples(api):
    # ident: aaaaaaaaaaaaaircaaaaaaaaam

    c1 = api.lookup_creator(orcid='0000-0003-3118-6859')
    assert c1.ident == "aaaaaaaaaaaaaircaaaaaaaaam"
