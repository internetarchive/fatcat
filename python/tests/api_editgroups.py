
import datetime

import pytest
from fatcat_openapi_client import *
from fixtures import *


def test_editgroup_submit(api):
    # 1. check that edit group can be submitted/unsubmitted, and shows up in reviewable appropriately
    # 2. accepted edits don't show up as reviewable and can't be submitted

    c1 = CreatorEntity(display_name="test updates")
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(eg.editgroup_id, c1).ident)

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

    with pytest.raises(fatcat_openapi_client.rest.ApiException):
        api.update_editgroup(eg.editgroup_id, eg3, submit=True)
    with pytest.raises(fatcat_openapi_client.rest.ApiException):
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


def test_editgroup_auto_batch(api):

    c1 = CreatorEntity(display_name="test auto_batch")
    c2 = CreatorEntity(display_name="test another auto_batch")

    eg1 = api.create_creator_auto_batch(CreatorAutoBatch(
        editgroup=Editgroup(),
        entity_list=[c1, c2]))

    assert eg1.changelog_index
    assert len(eg1.edits.creators) == 2


def test_batch_params(api):

    c1 = CreatorEntity(display_name="test auto_batch")
    c2 = CreatorEntity(display_name="test another auto_batch")

    desc = "test description"
    extra = dict(a=75, q="thing")
    eg1 = api.create_creator_auto_batch(CreatorAutoBatch(
        editgroup=Editgroup(
            description=desc,
            extra=extra),
        entity_list=[c1, c2]))

    assert eg1.description == desc
    assert eg1.extra == extra
