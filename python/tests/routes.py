
import json
import tempfile
import pytest
import fatcat
from fixtures import *


def test_static_routes(app):
    for route in ('/health', '/robots.txt', '/', '/about'):
        rv = app.get(route)
        assert rv.status_code == 200

    assert app.get("/static/bogus/route").status_code == 404


def test_all_views(app):
    for route in ('work', 'release', 'creator', 'container', 'file'):
        print(route)
        rv = app.get('/{}/1'.format(route))
        assert rv.status_code == 200

        rv = app.get('/{}/999999999999'.format(route))
        assert rv.status_code == 404

    rv = app.get('/work/random')
    rv = app.get(rv.location)
    assert rv.status_code == 200

    rv = app.get('/work/random')
    assert rv.status_code == 302

    rv = app.get('/work/create')
    assert rv.status_code == 200

    rv = app.get('/release/random')
    assert rv.status_code == 302

    rv = app.get('/release/1/changelog')
    assert rv.status_code == 200

    rv = app.get('/editgroup/1')
    assert rv.status_code == 200

    rv = app.get('/editgroup/99999999')
    assert rv.status_code == 404

    rv = app.get('/editgroup/current')
    assert rv.status_code == 302

    rv = app.get('/editor/admin')
    assert rv.status_code == 200

    rv = app.get('/editor/bizzaro')
    assert rv.status_code == 404

    rv = app.get('/editor/admin/changelog')
    assert rv.status_code == 200

    rv = app.get('/editor/bizarro/changelog')
    assert rv.status_code == 404
