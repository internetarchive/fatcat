
import json
import pytest
import fatcat
import fatcat.sql
import tempfile
from fatcat.models import *
from fixtures import *


def test_static_routes(rich_app):
    app = rich_app

    for route in ('/health', '/robots.txt', '/', '/about'):
        rv = app.get(route)
        assert rv.status_code == 200

    assert app.get("/static/bogus/route").status_code == 404


def test_all_views(rich_app):
    app = rich_app

    for route in ('work', 'release', 'creator', 'container', 'file'):
        print(route)
        rv = app.get('/{}/1'.format(route))
        assert rv.status_code == 200

        rv = app.get('/v0/work/random')
        rv = app.get(rv.location)
        assert rv.status_code == 200

    rv = app.get('/editgroup/1')
    assert rv.status_code == 200

    rv = app.get('/editor/admin')
    assert rv.status_code == 200

    rv = app.get('/editor/admin/changelog')
    assert rv.status_code == 200
