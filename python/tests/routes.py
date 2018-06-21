
import json
import tempfile
import pytest
import fatcat
from fatcat_client.rest import ApiException
from fixtures import *


def test_static_routes(app):
    for route in ('/health', '/robots.txt', '/', '/about'):
        rv = app.get(route)
        assert rv.status_code == 200

    assert app.get("/static/bogus/route").status_code == 404


def test_all_views(app):
    for route in ('work', 'release', 'creator', 'container', 'file'):
        print(route)
        rv = app.get('/{}/9999999999'.format(route))
        assert rv.status_code == 404

        rv = app.get('/{}/f1f046a3-45c9-ffff-ffff-ffffffffffff'.format(route))
        assert rv.status_code == 404

    rv = app.get('/container/00000000-0000-0000-1111-000000000002'.format(route))
    assert rv.status_code == 200

    rv = app.get('/container/create')
    assert rv.status_code == 200

    rv = app.get('/creator/00000000-0000-0000-2222-000000000002'.format(route))
    assert rv.status_code == 200

    rv = app.get('/file/00000000-0000-0000-3333-000000000002'.format(route))
    assert rv.status_code == 200

    rv = app.get('/release/00000000-0000-0000-4444-000000000002'.format(route))
    assert rv.status_code == 200

    rv = app.get('/work/00000000-0000-0000-5555-000000000002'.format(route))
    assert rv.status_code == 200

    rv = app.get('/work/create')
    assert rv.status_code == 200

    #rv = app.get('/release/00000000-0000-0000-4444-000000000002/changelog')
    #assert rv.status_code == 200

    rv = app.get('/editgroup/1')
    assert rv.status_code == 200

    rv = app.get('/editgroup/99999999')
    print(rv)
    print(rv.data)
    assert rv.status_code == 404

    #rv = app.get('/editgroup/current')
    #assert rv.status_code == 302

    rv = app.get('/editor/admin')
    assert rv.status_code == 200

    rv = app.get('/editor/admin/changelog')
    assert rv.status_code == 200
