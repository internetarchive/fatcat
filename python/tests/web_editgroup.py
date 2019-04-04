
import json
import tempfile
import pytest
from fatcat_client.rest import ApiException
from fixtures import *

def test_web_editgroup(app):

    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaae')
    assert rv.status_code == 200

    rv = app.get('/editgroup/ccccccccccccccccccccccccca')
    assert rv.status_code == 404

    rv = app.get('/editor/aaaaaaaaaaaabkvkaaaaaaaaae')
    assert rv.status_code == 200

    rv = app.get('/editor/aaaaaaaaaaaabkvkaaaaaaaaae/editgroups')
    assert rv.status_code == 200
