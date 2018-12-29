
import json
import tempfile
import pytest
from fatcat_client.rest import ApiException
from fixtures import *


def test_static_routes(app):
    for route in ('/health', '/robots.txt', '/', '/about'):
        rv = app.get(route)
        rv.raise_for_status()

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
    rv.raise_for_status()

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai/history')
    rv.raise_for_status()

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai/edit')
    rv.raise_for_status()

    rv = app.get('/container/create')
    rv.raise_for_status()

    rv = app.get('/container/lookup')
    assert rv.status_code == 400

    rv = app.get('/container/lookup?issnl=9999-9999')
    assert rv.status_code == 404

    rv = app.get('/container/lookup?issnl=1234-5678')
    assert rv.status_code == 302

    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai')
    rv.raise_for_status()

    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai/history')
    rv.raise_for_status()

    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai/edit')
    rv.raise_for_status()

    rv = app.get('/creator/lookup?orcid=0000-0003-2088-7465')
    assert rv.status_code == 302

    rv = app.get('/file/aaaaaaaaaaaaamztaaaaaaaaai')
    rv.raise_for_status()

    rv = app.get('/file/lookup?sha1=7d97e98f8af710c7e7fe703abc8f639e0ee507c4')
    assert rv.status_code == 302

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai')
    rv.raise_for_status()

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/history')
    rv.raise_for_status()

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/edit')
    rv.raise_for_status()

    rv = app.get('/release/create')
    rv.raise_for_status()

    rv = app.get('/release/lookup?doi=10.123/abc')
    assert rv.status_code == 302

    rv = app.get('/release/lookup?doi=10.123%2Fabc')
    assert rv.status_code == 302

    rv = app.get('/release/search')
    rv.raise_for_status()

    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai')
    rv.raise_for_status()

    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai/history')
    rv.raise_for_status()

    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai/edit')
    rv.raise_for_status()

    rv = app.get('/work/create')
    assert rv.status_code == 404

    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaae')
    rv.raise_for_status()

    rv = app.get('/editgroup/ccccccccccccccccccccccccca')
    print(rv)
    print(rv.data)
    assert rv.status_code == 404

    #rv = app.get('/editgroup/current')
    #assert rv.status_code == 302

    rv = app.get('/editor/aaaaaaaaaaaabkvkaaaaaaaaae')
    rv.raise_for_status()

    rv = app.get('/editor/aaaaaaaaaaaabkvkaaaaaaaaae/changelog')
    rv.raise_for_status()
