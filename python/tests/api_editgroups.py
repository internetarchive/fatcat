
import json
import pytest
import datetime
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
from fixtures import *


def test_editgroup_submit(api):
    # 1. check that edit group can be submitted/unsubmitted, and shows up in reviewable appropriately
    # 2. accepted edits don't show up as reviewable and can't be submitted

    c1 = CreatorEntity(display_name="test updates")
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup_id=eg.editgroup_id).ident)

    eg2 = api.get_editgroup(eg.editgroup_id)
    assert not eg2.submitted
    assert not eg2.changelog_index

    reviewable = api.get_editgroups_reviewable(limit=100)
    assert eg.editgroup_id not in [v.editgroup_id for v in reviewable]
    wip = api.get_editor_editgroups(eg.editor_id, limit=100)
    assert eg.editgroup_id in [v.editgroup_id for v in wip]

    api.update_editgroup(eg.editgroup_id, eg2, submit=True)
    eg3 = api.get_editgroup(eg.editgroup_id)
    assert eg3.submitted
    reviewable = api.get_editgroups_reviewable(limit=100)
    assert eg.editgroup_id in [v.editgroup_id for v in reviewable]

    api.update_editgroup(eg.editgroup_id, eg2, submit=False)
    eg3 = api.get_editgroup(eg.editgroup_id)
    assert not eg3.submitted
    reviewable = api.get_editgroups_reviewable(limit=100)
    assert eg.editgroup_id not in [v.editgroup_id for v in reviewable]

    # put back in reviewable
    api.update_editgroup(eg.editgroup_id, eg2, submit=True)
    reviewable = api.get_editgroups_reviewable(limit=100)
    assert eg.editgroup_id in [v.editgroup_id for v in reviewable]

    # shouldn't be reviewable if accepted
    api.accept_editgroup(eg.editgroup_id)
    reviewable = api.get_editgroups_reviewable(limit=100)
    assert eg.editgroup_id not in [v.editgroup_id for v in reviewable]
    eg3 = api.get_editgroup(eg.editgroup_id)
    #print(eg3)
    assert eg3.submitted
    assert eg3.changelog_index

    with pytest.raises(fatcat_client.rest.ApiException):
        api.update_editgroup(eg.editgroup_id, eg3, submit=True)
    with pytest.raises(fatcat_client.rest.ApiException):
        eg3.description = "something"
        api.update_editgroup(eg.editgroup_id, eg3)


def test_editgroup_ordering(api):

    eg1 = quick_eg(api)
    eg2 = quick_eg(api)
    api.update_editgroup(
        eg1.editgroup_id,
        Editgroup(editgroup_id=eg1.editgroup_id, description="FAIL"),
        submit=True)
    api.update_editgroup(
        eg2.editgroup_id,
        Editgroup(editgroup_id=eg2.editgroup_id, description="FAIL"),
        submit=True)

    r1 = api.get_editgroups_reviewable()
    #print(r1)
    assert not r1[0].description
    assert not r1[1].description
    assert r1[0].submitted >= r1[1].submitted

    # should be no editgroups "in the future" (since now + 1sec)
    r1 = api.get_editgroups_reviewable(since=(datetime.datetime.utcnow() + datetime.timedelta(seconds=1)).isoformat()+"Z")
    assert not r1

    r1 = api.get_editgroups_reviewable(since=(datetime.datetime.utcnow() - datetime.timedelta(seconds=5)).isoformat()+"Z")
    assert r1[0].submitted <= r1[1].submitted


def test_editgroup_autoaccept(api):
    # autoaccept changes: editgroups required when, in what combination

    eg = quick_eg(api)
    c1 = CreatorEntity(display_name="test autoaccept")
    c2 = CreatorEntity(display_name="test another autoaccept")

    with pytest.raises(fatcat_client.rest.ApiException):
        edits = api.create_creator_batch([c1, c2])

    with pytest.raises(fatcat_client.rest.ApiException):
        edits = api.create_creator_batch([c1, c2], editgroup_id=eg.editgroup_id, autoaccept=True)

    edits1 = api.create_creator_batch([c1, c2], editgroup_id=eg.editgroup_id)
    edits2 = api.create_creator_batch([c1, c2], autoaccept=True)

    assert edits1[0].editgroup_id == eg.editgroup_id
    assert edits1[0].editgroup_id != edits2[1].editgroup_id
    eg1 = api.get_editgroup(edits1[0].editgroup_id)
    eg2 = api.get_editgroup(edits2[0].editgroup_id)

    assert not eg1.changelog_index
    assert eg2.changelog_index
    #print(edits1)
    #print(eg1.edits.creators)
    assert eg1.edits.creators[0].ident == edits1[0].ident
    assert eg2.edits.creators[0].ident == edits2[0].ident

