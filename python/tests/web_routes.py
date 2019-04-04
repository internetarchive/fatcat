
import json
import tempfile
import pytest
from fatcat_client.rest import ApiException
from fixtures import *


def test_static_routes(app):
    for route in ('/health.json', '/robots.txt', '/', '/about'):
        rv = app.get(route)
        assert rv.status_code == 200

    assert app.get("/static/bogus/route").status_code == 404


def test_all_views(app):

    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaae')
    assert rv.status_code == 200

    rv = app.get('/editgroup/ccccccccccccccccccccccccca')
    print(rv)
    print(rv.data)
    assert rv.status_code == 404

    #rv = app.get('/editgroup/current')
    #assert rv.status_code == 302

    rv = app.get('/editor/aaaaaaaaaaaabkvkaaaaaaaaae')
    assert rv.status_code == 200

    rv = app.get('/editor/aaaaaaaaaaaabkvkaaaaaaaaae/editgroups')
    assert rv.status_code == 200
