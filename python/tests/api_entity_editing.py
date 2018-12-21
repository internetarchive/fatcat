
import json
import pytest
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
from fixtures import *


def test_multiple_edits_same_group(api):

    c1 = CreatorEntity(display_name="test updates")

    # create
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup=eg.id).ident)
    api.accept_editgroup(eg.id)

    # try multiple edits in the same group
    eg = quick_eg(api)
    c2 = CreatorEntity(display_name="left")
    c3 = CreatorEntity(display_name="right")
    edit = api.update_creator(c1.ident, c2, editgroup=eg.id)
    # should fail with existing
    with pytest.raises(fatcat_client.rest.ApiException):
        api.update_creator(c1.ident, c3, editgroup=eg.id)
    # ... but succeed after deleting
    api.delete_creator_edit(edit.edit_id)
    api.update_creator(c1.ident, c3, editgroup=eg.id)
    api.accept_editgroup(eg.id)
    res = api.get_creator(c1.ident)
    assert res.display_name == "right"
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 1

    # cleanup
    eg = quick_eg(api)
    api.delete_creator(c1.ident)
    api.accept_editgroup(eg.id)


def test_edit_after_accept(api):

    c1 = CreatorEntity(display_name="test updates")

    # create
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup=eg.id).ident)
    api.accept_editgroup(eg.id)

    # should be unable to create an edit on an old editgroup
    c2 = CreatorEntity(display_name="left")
    try:
        api.create_creator(c2, editgroup=eg.id)
        assert False
    except fatcat_client.rest.ApiException as e:
        assert 400 <= e.status < 500
        # TODO: need better message
        #assert "accepted" in e.body

    # cleanup
    eg = quick_eg(api)
    api.delete_creator(c1.ident)
    api.accept_editgroup(eg.id)


def test_edit_deletion(api):

    c1 = CreatorEntity(display_name="test edit updates")

    # create
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup=eg.id).ident)
    api.accept_editgroup(eg.id)

    # try multiple edits in the same group
    c2 = CreatorEntity(display_name="update one")
    eg = quick_eg(api)
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 0
    edit = api.update_creator(c1.ident, c2, editgroup=eg.id)
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 1
    api.delete_creator_edit(edit.edit_id)
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 0

    api.accept_editgroup(eg.id)
    res = api.get_creator(c1.ident)
    assert res.display_name == "test edit updates"
    eg = api.get_editgroup(eg.id)
    assert len(eg.edits.creators) == 0

    # cleanup
    eg = quick_eg(api)
    api.delete_creator(c1.ident)
    api.accept_editgroup(eg.id)


def test_empty_editgroup(api):
    eg = quick_eg(api)
    api.accept_editgroup(eg.id)


def test_delete_accepted_edit(api):

    c1 = CreatorEntity(display_name="test edit updates")

    # create
    eg = quick_eg(api)
    edit = api.create_creator(c1, editgroup=eg.id)
    api.accept_editgroup(eg.id)

    # try to delete
    try:
        api.delete_creator_edit(edit.edit_id)
        assert False
    except fatcat_client.rest.ApiException as e:
        assert 400 <= e.status < 500
        assert "accepted" in e.body


def test_wip_revision(api):

    c1 = CreatorEntity(display_name="test edit nugget")

    # fetch revision before accepting
    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(c1, editgroup=eg.id).ident)
    rev = api.get_creator_revision(c1.revision)
    assert "nugget" in rev.display_name
    assert rev.state is None
    assert rev.ident is None
    assert rev.revision == c1.revision

    # fetch revision after accepting
    api.accept_editgroup(eg.id)
    rev = api.get_creator_revision(c1.revision)
    assert "nugget" in rev.display_name
    assert rev.state is None
    assert rev.ident is None
    assert rev.revision == c1.revision

