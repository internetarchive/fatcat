
import pytest
import fatcat
import fatcat.sql
from fatcat.models import *


@pytest.fixture
def app():
    fatcat.app.config['SQLALCHEMY_DATABASE_URI'] = 'sqlite://'
    fatcat.app.testing = True
    fatcat.app.debug = True
    fatcat.db.session.remove()
    fatcat.db.drop_all()
    fatcat.db.create_all()
    fatcat.sql.populate_db()
    return fatcat.app.test_client()

@pytest.fixture
def rich_app(app):

    rv = app.post('/v0/editgroup',
        data=json.dumps(dict(
            extra=dict(q=1, u="zing"))),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    obj = json.loads(rv.data.decode('utf-8'))
    editgroup_id = obj['id']

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

    rv = app.post('/v0/editgroup/{}/accept'.format(editgroup_id),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200

    return app

def test_rich_app_fixture(rich_app):
    app = rich_app

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

    # test that editor's active edit group is now invalid
    editor = Editor.query.first()
    assert editor.active_edit_group == None


## Helpers ##################################################################

def check_entity_fields(e):
    for key in ('rev', 'is_live', 'redirect_id'):
        assert key in e
    for key in ('id',):
        assert e[key] is not None

def check_release(e):
    for key in ('work', 'release_type'):
        assert key in e
    for key in ('title'):
        assert e[key] is not None
    for key in ('refs', 'creators'):
        assert type(e[key]) == list

def check_creator(e):
    for key in ('name',):
        assert e[key] is not None

def check_container(e):
    for key in ('name',):
        assert e[key] is not None

def check_file(e):
    for key in ('size', 'sha1'):
        assert e[key] is not None
