
import json
import pytest
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
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
    )

    c1edit = api.create_creator(c1, editgroup_id=eg.editgroup_id)
    api.accept_editgroup(eg.editgroup_id)
    c2 = api.get_creator(c1edit.ident)

    # check that fields match
    assert c1.display_name == c2.display_name
    assert c1.given_name == c2.given_name
    assert c1.surname == c2.surname
    assert c1.orcid == c2.orcid
    assert c1.wikidata_qid == c2.wikidata_qid
    assert c1.extra == c2.extra

    # expansion
    # TODO: via release
    # lookup
    # TODO: via issnl; but need to generate random identifiers

def test_creators_examples(api):
    # TODO: aaaaaaaaaaaaaircaaaaaaaaam

    api.lookup_creator(orcid='0000-0003-3118-6859')
