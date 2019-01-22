
import json
import pytest
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
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
    )

    c1edit = api.create_container(c1, editgroup_id=eg.editgroup_id)
    api.accept_editgroup(eg.editgroup_id)
    c2 = api.get_container(c1edit.ident)

    # check that fields match
    assert c1.name == c2.name
    assert c1.container_type == c2.container_type
    assert c1.publisher == c2.publisher
    assert c1.issnl == c2.issnl
    assert c1.wikidata_qid == c2.wikidata_qid
    assert c1.extra == c2.extra

    # expansion
    # TODO: via release
    # lookup
    # TODO: via issnl; but need to generate random identifiers

def test_container_examples(api):

    api.lookup_container(issnl='1549-1277')

    c1 = api.get_container('aaaaaaaaaaaaaeiraaaaaaaaam')
    assert c1.name == "PLOS Medicine"
    assert c1.issnl == "1549-1277"

