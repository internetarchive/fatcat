
import json
import tempfile
import pytest
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
        assert rv.status_code == 400

        rv = app.get('/{}/f1f046a3-45c9-ffff-ffff-ffffffffffff'.format(route))
        assert rv.status_code == 400

        rv = app.get('/{}/ccccccccccccccccccccccccca'.format(route))
        assert rv.status_code == 404

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai')
    assert rv.status_code == 200

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai/history')
    assert rv.status_code == 200

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai/edit')
    assert rv.status_code == 200

    rv = app.get('/container/create')
    assert rv.status_code == 200

    rv = app.get('/container/lookup')
    assert rv.status_code == 400

    rv = app.get('/container/lookup?issnl=9999-9999')
    assert rv.status_code == 404

    rv = app.get('/container/lookup?issnl=1234-5678')
    assert rv.status_code == 302

    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai')
    assert rv.status_code == 200

    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai/history')
    assert rv.status_code == 200

    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai/edit')
    assert rv.status_code == 200

    rv = app.get('/creator/lookup?orcid=0000-0003-2088-7465')
    assert rv.status_code == 302

    rv = app.get('/file/aaaaaaaaaaaaamztaaaaaaaaai')
    assert rv.status_code == 200

    rv = app.get('/file/lookup?sha1=7d97e98f8af710c7e7fe703abc8f639e0ee507c4')
    assert rv.status_code == 302

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai')
    assert rv.status_code == 200

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/history')
    assert rv.status_code == 200

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/edit')
    assert rv.status_code == 200

    rv = app.get('/release/create')
    assert rv.status_code == 200

    rv = app.get('/release/lookup?doi=10.123/abc')
    assert rv.status_code == 302

    rv = app.get('/release/lookup?doi=10.123%2Fabc')
    assert rv.status_code == 302

    rv = app.get('/release/search')
    assert rv.status_code == 200

    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai')
    assert rv.status_code == 200

    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai/history')
    assert rv.status_code == 200

    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai/edit')
    assert rv.status_code == 200

    rv = app.get('/work/create')
    assert rv.status_code == 404

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
