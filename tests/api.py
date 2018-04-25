
import json
import unittest
import tempfile
import pytest
import fatcat
import fatcat.sql
from fatcat.models import *
from fixtures import *


def test_health(app):
    rv = app.get('/health')
    obj = json.loads(rv.data.decode('utf-8'))
    assert obj['ok']

def test_api_work(app):
    fatcat.dummy.insert_example_works()

    # Invalid Id
    rv = app.get('/v0/work/_')
    assert rv.status_code == 404

    # Random
    rv = app.get('/v0/work/random')
    rv = app.get(rv.location)
    work = json.loads(rv.data.decode('utf-8'))
    check_entity_fields(work)
    print(work)
    assert work['title']
    assert work['work_type']

    # Valid Id (from random above)
    rv = app.get('/v0/work/{}'.format(work['id']))
    assert rv.status_code == 200

    # Missing Id
    rv = app.get('/v0/work/r3zga5b9cd7ef8gh084714iljk')
    assert rv.status_code == 404

def test_api_work_create(app):
    assert WorkIdent.query.count() == 0
    assert WorkRev.query.count() == 0
    assert WorkEdit.query.count() == 0
    rv = app.post('/v0/work',
        data=json.dumps(dict(title="dummy", work_type="thing", extra=dict(a=1, b="zing"))),
        headers={"content-type": "application/json"})
    print(rv)
    assert rv.status_code == 200
    assert WorkIdent.query.count() == 1
    assert WorkRev.query.count() == 1
    assert WorkEdit.query.count() == 1
    # not alive yet
    assert WorkIdent.query.filter(WorkIdent.is_live==True).count() == 0

def test_api_rich_create(app):

    # TODO: create user?

    rv = app.post('/v0/editgroup',
        data=json.dumps(dict(
            extra=dict(q=1, u="zing"))),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    editgroup_id = obj['id']

    for cls in (WorkIdent, WorkRev, WorkEdit,
                ContainerIdent, ContainerRev, ContainerEdit,
                CreatorIdent, CreatorRev, CreatorEdit,
                ReleaseIdent, ReleaseRev, ReleaseEdit,
                FileIdent, FileRev, FileEdit,
                ChangelogEntry):
        assert cls.query.count() == 0

    rv = app.post('/v0/container',
        data=json.dumps(dict(
            name="schmournal",
            publisher="society of authors",
            issn="2222-3333",
            editgroup=editgroup_id,
            extra=dict(a=2, i="zing"))),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    container_id = obj['id']

    rv = app.post('/v0/creator',
        data=json.dumps(dict(
            name="anon y. mouse",
            orcid="0000-0002-1825-0097",
            editgroup=editgroup_id,
            extra=dict(w=1, q="zing"))),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    creator_id = obj['id']

    rv = app.post('/v0/work',
        data=json.dumps(dict(
            title="dummy work",
            work_type="book",
            editgroup=editgroup_id,
            extra=dict(a=3, b="zing"))),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    work_id = obj['id']

    # this stub work will be referenced
    rv = app.post('/v0/release',
        data=json.dumps(dict(
            title="derivative work",
            work_type="journal-article",
            work=work_id,
            creators=[creator_id],
            doi="10.1234/58",
            editgroup=editgroup_id,
            refs=[
                dict(stub="some other journal article"),
            ],
            extra=dict(f=7, b="zing"))),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    stub_release_id = obj['id']

    rv = app.post('/v0/release',
        data=json.dumps(dict(
            title="dummy work",
            work_type="book",
            work=work_id,
            container=container_id,
            creators=[creator_id],
            doi="10.1234/5678",
            editgroup=editgroup_id,
            refs=[
                dict(stub="some book", target=stub_release_id),
            ],
            extra=dict(f=7, b="loopy"))),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    release_id = obj['id']

    rv = app.post('/v0/file',
        data=json.dumps(dict(
            sha1="deadbeefdeadbeef",
            size=1234,
            releases=[release_id],
            editgroup=editgroup_id,
            extra=dict(f=4, b="zing"))),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    file_id = obj['id']

    for cls in (WorkIdent, WorkRev, WorkEdit,
                ContainerIdent, ContainerRev, ContainerEdit,
                CreatorIdent, CreatorRev, CreatorEdit,
                FileIdent, FileRev, FileEdit):
        assert cls.query.count() == 1
    for cls in (ReleaseIdent, ReleaseRev, ReleaseEdit):
        assert cls.query.count() == 2

    for cls in (WorkIdent,
                ContainerIdent,
                CreatorIdent,
                ReleaseIdent,
                FileIdent):
        assert cls.query.filter(cls.is_live==True).count() == 0

    assert ChangelogEntry.query.count() == 0
    rv = app.post('/v0/editgroup/{}/accept'.format(editgroup_id),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    assert ChangelogEntry.query.count() == 1

    for cls in (WorkIdent, WorkRev, WorkEdit,
                ContainerIdent, ContainerRev, ContainerEdit,
                CreatorIdent, CreatorRev, CreatorEdit,
                FileIdent, FileRev, FileEdit):
        assert cls.query.count() == 1
    for cls in (ReleaseIdent, ReleaseRev, ReleaseEdit):
        assert cls.query.count() == 2

    for cls in (WorkIdent,
                ContainerIdent,
                CreatorIdent,
                FileIdent):
        assert cls.query.filter(cls.is_live==True).count() == 1
    assert ReleaseIdent.query.filter(ReleaseIdent.is_live==True).count() == 2

    # Test that foreign key relations worked
    release_rv = json.loads(app.get('/v0/release/{}'.format(release_id)).data.decode('utf-8'))
    print(release_rv)
    assert release_rv['creators'][0]['creator'] == creator_id
    assert release_rv['container']['id'] == container_id
    assert release_rv['work']['id'] == work_id
    assert release_rv['refs'][0]['target'] == stub_release_id

    file_rv = json.loads(app.get('/v0/file/{}'.format(file_id)).data.decode('utf-8'))
    print(file_rv)
    assert file_rv['releases'][0]['release'] == release_id

    # test that editor's active edit group is now invalid
    editor = Editor.query.first()
    assert editor.active_editgroup is None

def test_api_release_lookup(rich_app):
    app = rich_app

    rv = app.get('/v0/release/1',
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))

    rv = app.get('/v0/release/lookup',
        data=json.dumps(dict(doi="10.1234/5678")),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    assert obj['doi'] == "10.1234/5678"
    assert obj.get('id') != None

    rv = app.get('/v0/release/lookup',
        data=json.dumps(dict(doi="10.1234/5678_noexit")),
        headers={"content-type": "application/json"})
    assert rv.status_code == 404

    rv = app.get('/v0/release/lookup',
        data=json.dumps(dict(doi="not_even_valid_doi")),
        headers={"content-type": "application/json"})
    assert rv.status_code == 400

def test_api_creator_lookup(rich_app):
    app = rich_app

    rv = app.get('/v0/creator/1',
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))

    rv = app.get('/v0/creator/lookup',
        data=json.dumps(dict(orcid="0000-0002-1825-0097")),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    assert obj['orcid'] == "0000-0002-1825-0097"
    assert obj.get('id') != None

    rv = app.get('/v0/creator/lookup',
        data=json.dumps(dict(orcid="0000-0002-1825-0098")),
        headers={"content-type": "application/json"})
    assert rv.status_code == 404

    rv = app.get('/v0/creator/lookup',
        data=json.dumps(dict(orcid="not_even_valid_orcid")),
        headers={"content-type": "application/json"})
    assert rv.status_code == 400


def test_api_container_lookup(rich_app):
    app = rich_app

    rv = app.get('/v0/container/1',
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))

    rv = app.get('/v0/container/lookup',
        data=json.dumps(dict(issn="2222-3333")),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    assert obj['issn'] == "2222-3333"
    assert obj.get('id') != None

    rv = app.get('/v0/container/lookup',
        data=json.dumps(dict(issn="2222-3334")),
        headers={"content-type": "application/json"})
    assert rv.status_code == 404

    rv = app.get('/v0/container/lookup',
        data=json.dumps(dict(issn="not_even_valid_issn")),
        headers={"content-type": "application/json"})
    assert rv.status_code == 400

def test_api_editor_get(rich_app):
    app = rich_app

    rv = app.get('/v0/editor/admin',
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    print(obj)
    assert obj['username'] == "admin"
    assert obj['id'] == 1

def test_api_editor_changelog(rich_app):
    app = rich_app

    rv = app.get('/v0/editor/admin/changelog',
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    print(obj)
    assert len(obj) == 1
