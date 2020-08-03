
import json
from fixtures import *

from fatcat_web.forms import ReleaseEntityForm, FileEntityForm, ContainerEntityForm

DUMMY_DEMO_ENTITIES = {
    'container': ('aaaaaaaaaaaaaeiraaaaaaaaai', '00000000-0000-0000-1111-fff000000002'),
    # note inconsistency here (q not i)
    'creator': ('aaaaaaaaaaaaaircaaaaaaaaaq', '00000000-0000-0000-2222-fff000000002'),
    'file': ('aaaaaaaaaaaaamztaaaaaaaaai', '00000000-0000-0000-3333-fff000000002'),
    'fileset': ('aaaaaaaaaaaaaztgaaaaaaaaai', '00000000-0000-0000-6666-fff000000002'),
    'webcapture': ('aaaaaaaaaaaaa53xaaaaaaaaai', '00000000-0000-0000-7777-fff000000002'),
    'release': ('aaaaaaaaaaaaarceaaaaaaaaai', '00000000-0000-0000-4444-fff000000002'),
    'work': ('aaaaaaaaaaaaavkvaaaaaaaaai', '00000000-0000-0000-5555-fff000000002'),
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


def test_entity_basics(app, mocker):

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    # these are basic ES stats for the container view pages
    es_raw.side_effect = [
        (200, {}, json.dumps(ES_CONTAINER_STATS_RESP)),
        (200, {}, json.dumps(ES_CONTAINER_RANDOM_RESP)),
    ]

    for entity_type, (ident, revision) in DUMMY_DEMO_ENTITIES.items():
        # good requests
        rv = app.get('/{}/{}'.format(entity_type, ident))
        assert rv.status_code == 200
        rv = app.get('/{}_{}'.format(entity_type, ident))
        assert rv.status_code == 302
        rv = app.get('/{}/{}/history'.format(entity_type, ident))
        assert rv.status_code == 200
        rv = app.get('/{}/{}/metadata'.format(entity_type, ident))
        assert rv.status_code == 200
        rv = app.get('/{}/rev/{}'.format(entity_type, revision))
        assert rv.status_code == 200
        rv = app.get('/{}/rev/{}_something'.format(entity_type, revision))
        assert rv.status_code == 400
        rv = app.get('/{}/rev/{}/metadata'.format(entity_type, revision))
        assert rv.status_code == 200
        print('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/{}/{}'.format(entity_type, ident))
        rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/{}/{}'.format(entity_type, ident))
        assert rv.status_code == 200
        rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/{}/{}/metadata'.format(entity_type, ident))
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

def test_web_deleted_release(app, api):
    # specific regression test for view of a deleted release

    # create release
    eg = quick_eg(api)
    r1 = ReleaseEntity(
        title="some title",
        ext_ids=ReleaseExtIds(),
    )
    r1edit = api.create_release(eg.editgroup_id, r1)
    api.accept_editgroup(eg.editgroup_id)

    # delete
    eg = quick_eg(api)
    api.delete_release(eg.editgroup_id, r1edit.ident)
    api.accept_editgroup(eg.editgroup_id)
    r2 = api.get_release(r1edit.ident)
    assert r2.state == "deleted"

    rv = app.get('/release/{}'.format(r2.ident))
    assert rv.status_code == 200
    rv = app.get('/release/{}/metadata'.format(r2.ident))
    assert rv.status_code == 200
    rv = app.get('/release/{}/history'.format(r2.ident))
    assert rv.status_code == 200


def test_lookups(app):

    rv = app.get('/container/lookup')
    assert rv.status_code == 200
    rv = app.get('/container/lookup?issnl=9999-9999')
    assert rv.status_code == 404
    rv = app.get('/container/lookup?issnl=1234-5678')
    assert rv.status_code == 302

    rv = app.get('/creator/lookup')
    assert rv.status_code == 200
    rv = app.get('/creator/lookup?orcid=0000-0003-2088-7465')
    assert rv.status_code == 302
    rv = app.get('/creator/lookup?orcid=0000-0003-2088-0000')
    assert rv.status_code == 404

    rv = app.get('/file/lookup')
    assert rv.status_code == 200
    rv = app.get('/file/lookup?sha1=7d97e98f8af710c7e7fe703abc8f639e0ee507c4')
    assert rv.status_code == 302
    rv = app.get('/file/lookup?sha1=7d97e98f8af710c7e7f00000000000000ee507c4')
    assert rv.status_code == 404

    rv = app.get('/fileset/lookup')
    assert rv.status_code == 404

    rv = app.get('/webcapture/lookup')
    assert rv.status_code == 404

    rv = app.get('/release/lookup')
    assert rv.status_code == 200
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


def test_web_container(app, mocker):

    es_raw = mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    # these are basic ES stats for the container view pages
    es_raw.side_effect = [
        (200, {}, json.dumps(ES_CONTAINER_STATS_RESP)),
        (200, {}, json.dumps(ES_CONTAINER_RANDOM_RESP)),
    ]

    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai/metadata')
    assert rv.status_code == 200
    rv = app.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai/edit')
    assert rv.status_code == 302
    rv = app.get('/container/create')
    assert rv.status_code == 302
    rv = app.get('/container/rev/00000000-0000-0000-1111-fff000000002')
    assert rv.status_code == 200
    rv = app.get('/container/rev/00000000-0000-0000-1111-fff000000002/metadata')
    assert rv.status_code == 200
    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/container/aaaaaaaaaaaaaeiraaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/container/aaaaaaaaaaaaaeiraaaaaaaaai/metadata')
    assert rv.status_code == 200
    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/container/aaaaaaaaaaaaaeiraaaaaaaaai/edit')
    assert rv.status_code == 302


def test_web_container_login(full_app, app_admin):

    rv = app_admin.get('/container/aaaaaaaaaaaaaeiraaaaaaaaai/edit')
    assert rv.status_code == 200
    assert b'Journal of Trivial Results' in rv.data
    assert b'1234-5678' in rv.data
    rv = app_admin.get('/container/create')
    assert rv.status_code == 200

    # creation (via form)
    with full_app.test_request_context():
        form = ContainerEntityForm()
        form.issnl.data = "invalid-issn"
        rv = app_admin.post('/container/create', data=form.data, follow_redirects=True)
        assert rv.status_code == 400
        assert b'invalid-issn' in rv.data

    with full_app.test_request_context():
        form = ContainerEntityForm()
        # these fields are required
        form.name.data = "Journal of Experiments"
        rv = app_admin.post('/container/create', data=form.data, follow_redirects=True)
        assert rv.status_code == 200
        assert b"Journal of Experiments" in rv.data

    # editing (via form)
    with full_app.test_request_context():
        form = ContainerEntityForm()
        form.issnl.data = "invalid-issn"
        rv = app_admin.post('/container/aaaaaaaaaaaaaeiraaaaaaaaai/edit',
            data=form.data, follow_redirects=True)
        assert rv.status_code == 400
        assert b'invalid-issn' in rv.data


def test_web_creator(app):
    # not logged in

    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/creator/aaaaaaaaaaaaaircaaaaaaaaai/edit')
    assert rv.status_code == 302
    rv = app.get('/creator/create')
    assert rv.status_code == 302


def test_web_file(app):
    # not logged in

    rv = app.get('/file/aaaaaaaaaaaaamztaaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/file/aaaaaaaaaaaaamztaaaaaaaaai/edit')
    assert rv.status_code == 302
    rv = app.get('/file/create')
    assert rv.status_code == 302


def test_web_file_login(full_app, app_admin):

    rv = app_admin.get('/file/aaaaaaaaaaaaamztaaaaaaaaai/edit')
    assert rv.status_code == 200
    assert b'7d97e98f8af710c7e7fe703abc8f639e0ee507c4' in rv.data
    assert b'archive.org/robots.txt' in rv.data
    rv = app_admin.get('/file/create')
    assert rv.status_code == 200

    # creation (via form)
    with full_app.test_request_context():
        form = FileEntityForm()
        form.sha1.data = "invalidstring"
        rv = app_admin.post('/file/create', data=form.data, follow_redirects=True)
        assert rv.status_code == 400
        assert b'invalidstring' in rv.data

    with full_app.test_request_context():
        form = FileEntityForm()
        # these fields are required
        form.size.data = 1234
        form.sha1.data = "202f899638fcaa97128b968a43a8f45e00b69a25"
        rv = app_admin.post('/file/create', data=form.data, follow_redirects=True)
        assert rv.status_code == 200

    # editing (via form)
    with full_app.test_request_context():
        form = FileEntityForm()
        form.md5.data = "invalidstring"
        rv = app_admin.post('/file/aaaaaaaaaaaaamztaaaaaaaaai/edit',
            data=form.data, follow_redirects=True)
        assert rv.status_code == 400
        assert b'invalidstring' in rv.data

def test_web_fileset(app):
    # not logged in

    rv = app.get('/fileset/aaaaaaaaaaaaaztgaaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/fileset/aaaaaaaaaaaaaztgaaaaaaaaai/edit')
    assert rv.status_code == 302
    rv = app.get('/fileset/create')
    assert rv.status_code == 302


def test_web_webcatpure(app):
    # not logged in

    rv = app.get('/webcapture/aaaaaaaaaaaaa53xaaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/webcapture/aaaaaaaaaaaaa53xaaaaaaaaai/edit')
    assert rv.status_code == 302
    rv = app.get('/webcapture/create')
    assert rv.status_code == 302


def test_web_release(app):
    # not logged in

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/contribs')
    assert rv.status_code == 200
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/references')
    assert rv.status_code == 200
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/metadata')
    assert rv.status_code == 200
    rv = app.get('/release/rev/00000000-0000-0000-4444-fff000000002/contribs')
    assert rv.status_code == 200
    rv = app.get('/release/rev/00000000-0000-0000-4444-fff000000002/references')
    assert rv.status_code == 200
    rv = app.get('/release/rev/00000000-0000-0000-4444-fff000000002/metadata')
    assert rv.status_code == 200
    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/release/aaaaaaaaaaaaarceaaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/release/aaaaaaaaaaaaarceaaaaaaaaai/contribs')
    assert rv.status_code == 200
    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/release/aaaaaaaaaaaaarceaaaaaaaaai/references')
    assert rv.status_code == 200
    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaaq/release/aaaaaaaaaaaaarceaaaaaaaaai/metadata')
    assert rv.status_code == 200

    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/edit')
    assert rv.status_code == 302
    rv = app.get('/release/create')
    assert rv.status_code == 302


def test_web_release_login(full_app, app_admin):

    rv = app_admin.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/edit')
    assert rv.status_code == 200
    assert b'robin hood' in rv.data
    assert b'PMC555' in rv.data
    rv = app_admin.get('/release/create')
    assert rv.status_code == 200

    # creation (via form)
    with full_app.test_request_context():
        form = ReleaseEntityForm()
        form.title.data = "My Research: Missing Some Stuff"
        rv = app_admin.post('/release/create', data=form.data, follow_redirects=True)
        assert rv.status_code == 400
        assert b'My Research: Missing Some Stuff' in rv.data
        assert not b'already' in rv.data

    with full_app.test_request_context():
        form = ReleaseEntityForm()
        # these fields are required
        form.title.data = "Creating Releases: A Review"
        form.release_type.data = "article-journal"
        form.release_stage.data = "draft"
        rv = app_admin.post('/release/create', data=form.data, follow_redirects=True)
        assert rv.status_code == 200

    with full_app.test_request_context():
        form = ReleaseEntityForm()
        # these fields are required
        form.title.data = "Creating Releases: A Review"
        form.release_type.data = "article-journal"
        form.release_stage.data = "draft"
        # already merged editgroup
        form.editgroup_id.data = "aaaaaaaaaaaabo53aaaaaaaaae"
        rv = app_admin.post('/release/create', data=form.data, follow_redirects=True)
        assert rv.status_code == 400
        # XXX: this should return the page with error annotated, not generic
        # 400 page
        #assert b"already accepted" in rv.data

    # editing
    with full_app.test_request_context():
        form = ReleaseEntityForm()
        form.title.data = "My Research: Missing Some Stuff"
        form.release_type.data = "bogus-release-type"
        rv = app_admin.post('/release/create', data=form.data, follow_redirects=True)
        assert rv.status_code == 400
        assert b'My Research: Missing Some Stuff' in rv.data


def test_web_search(app):

    rv = app.get('/release/search')
    assert rv.status_code == 200


def test_web_work(app):

    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai')
    assert rv.status_code == 200
    rv = app.get('/work/aaaaaaaaaaaaavkvaaaaaaaaai/edit')
    assert rv.status_code == 302
    rv = app.get('/work/create')
    assert rv.status_code == 302
