
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
