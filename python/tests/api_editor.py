
import pytest

import fatcat_openapi_client

from fixtures import api


def test_editor_update(api):

    editor_id = api.editor_id
    orig = api.get_editor(editor_id)
    newer = api.get_editor(editor_id)
    newer.username = "temp-bogus-username"
    api.update_editor(editor_id, newer)
    check = api.get_editor(editor_id)
    assert check.username != orig.username
    assert check.editor_id == orig.editor_id
    api.update_editor(editor_id, orig)
    check = api.get_editor(editor_id)
    assert check == orig

def test_editor_get(api):

    editor_id = api.editor_id
    api.get_editor(editor_id)

def test_editor_lookup(api):

    editor_id = api.editor_id
    e1 = api.get_editor(editor_id)

    e2 = api.lookup_editor(username=e1.username)
    assert e1.editor_id == e2.editor_id

    with pytest.raises(fatcat_openapi_client.rest.ApiException):
        api.lookup_editor(username="")

    with pytest.raises(fatcat_openapi_client.rest.ApiException):
        api.lookup_editor(username="bogus-username-notfound")
