
import json
import pytest
import datetime
from copy import copy

from fatcat_openapi_client import *
from fatcat_openapi_client.rest import ApiException
from fixtures import *


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
