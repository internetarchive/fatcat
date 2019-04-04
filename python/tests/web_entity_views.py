
import json
import pytest
from fatcat_client.rest import ApiException
from fixtures import *

DUMMY_DEMO_ENTITIES = {
    'container': 'aaaaaaaaaaaaaeiraaaaaaaaai',
    'creator': 'aaaaaaaaaaaaaircaaaaaaaaai',
    'file': 'aaaaaaaaaaaaamztaaaaaaaaai',
    'fileset': 'aaaaaaaaaaaaaztgaaaaaaaaai',
    'webcapture': 'aaaaaaaaaaaaa53xaaaaaaaaai',
    'release': 'aaaaaaaaaaaaarceaaaaaaaaai',
    'work': 'aaaaaaaaaaaaavkvaaaaaaaaai',
}

REALISTIC_DEMO_ENTITIES = {
    'container': 'aaaaaaaaaaaaaeiraaaaaaaaam',
    'creator': 'aaaaaaaaaaaaaircaaaaaaaaam',
    'file': 'aaaaaaaaaaaaamztaaaaaaaaam',
    'fileset': 'aaaaaaaaaaaaaztgaaaaaaaaam',
    'webcapture': 'aaaaaaaaaaaaa53xaaaaaaaaam',
    'release': 'aaaaaaaaaaaaarceaaaaaaaaam',
    'work': 'aaaaaaaaaaaaavkvaaaaaaaaam',
}


def test_entity_basics(app):

    for entity_type, ident in DUMMY_DEMO_ENTITIES.items():
        # good requests
        rv = app.get('/{}/{}'.format(entity_type, ident))
        assert rv.status_code == 200
        rv = app.get('/{}/{}/history'.format(entity_type, ident))
        assert rv.status_code == 200

        # bad requests
        rv = app.get('/{}/9999999999'.format(entity_type))
        assert rv.status_code == 400
        rv = app.get('/{}/9999999999/history'.format(entity_type))
        assert rv.status_code == 400
        rv = app.get('/{}/f1f046a3-45c9-ffff-ffff-ffffffffffff'.format(entity_type))
        assert rv.status_code == 400
        rv = app.get('/{}/ccccccccccccccccccccccccca'.format(entity_type))
        assert rv.status_code == 404

        # TODO: redirects and deleted entities


def test_container(app_admin):
    app = app_admin

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai')
    assert rv.status_code == 200

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai/edit')
    assert rv.status_code == 200

    rv = app.get('/container/create')
    assert rv.status_code == 200


def test_lookups(app):

    rv = app.get('/container/lookup')
    assert rv.status_code == 400
    rv = app.get('/container/lookup?issnl=9999-9999')
    assert rv.status_code == 404
    rv = app.get('/container/lookup?issnl=1234-5678')
    assert rv.status_code == 302

    rv = app.get('/creator/lookup')
    assert rv.status_code == 400
    rv = app.get('/creator/lookup?orcid=0000-0003-2088-7465')
    assert rv.status_code == 302
    rv = app.get('/creator/lookup?orcid=0000-0003-2088-0000')
    assert rv.status_code == 404

    rv = app.get('/file/lookup')
    assert rv.status_code == 400
    rv = app.get('/file/lookup?sha1=7d97e98f8af710c7e7fe703abc8f639e0ee507c4')
    assert rv.status_code == 302
    rv = app.get('/file/lookup?sha1=7d97e98f8af710c7e7f00000000000000ee507c4')
    assert rv.status_code == 404

    rv = app.get('/fileset/lookup')
    assert rv.status_code == 404

    rv = app.get('/webcapture/lookup')
    assert rv.status_code == 404

    rv = app.get('/release/lookup')
    assert rv.status_code == 400
    rv = app.get('/release/lookup?doi=10.123/abc')
    assert rv.status_code == 302
    rv = app.get('/release/lookup?doi=10.123%2Fabc')
    assert rv.status_code == 302
    rv = app.get('/release/lookup?doi=abcde')
    assert rv.status_code == 400
    rv = app.get('/release/lookup?doi=10.1234/uuu')
    assert rv.status_code == 404

    rv = app.get('/work/lookup')
    assert rv.status_code == 404


def test_web_creator(app):
    # not logged in

    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai/edit')
    assert rv.status_code == 302


def test_web_file(app):
    # not logged in

    rv = app.get('/file/aaaaaaaaaaaaamztaaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/file/aaaaaaaaaaaaamztaaaaaaaaai/edit')
    assert rv.status_code == 302
    rv = app.get('/file/aaaaaaaaaaaaamztaaaaaaaaai/history')
    assert rv.status_code == 200

def test_web_release(app):
    # not logged in

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai')
    assert rv.status_code == 200

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/edit')
    assert rv.status_code == 302
    rv = app.get('/release/create')
    assert rv.status_code == 302


def test_web_release_login(app_admin):

    rv = app_admin.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/edit')
    assert rv.status_code == 200
    rv = app_admin.get('/release/create')
    assert rv.status_code == 200


def test_web_search(app):

    rv = app.get('/release/search')
    assert rv.status_code == 200


def test_web_work(app):

    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai')
    assert rv.status_code == 200

    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai/edit')
    assert rv.status_code == 404

    rv = app.get('/work/create')
    assert rv.status_code == 404

