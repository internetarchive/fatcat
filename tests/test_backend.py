
import os
import json
import pytest
import fatcat
import fatcat.sql
from fatcat.models import *
import unittest
import tempfile

# TODO: http://alextechrants.blogspot.com/2013/08/unit-testing-sqlalchemy-apps.html

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

class FatcatTestCase(unittest.TestCase):

    def setUp(self):
        fatcat.app.config['SQLALCHEMY_DATABASE_URI'] = 'sqlite://'
        fatcat.app.testing = True
        fatcat.app.debug = True
        self.app = fatcat.app.test_client()
        fatcat.db.session.remove()
        fatcat.db.drop_all()
        fatcat.db.create_all()
        fatcat.sql.populate_db()


## Model Tests ###############################################################

class ModelTestCase(FatcatTestCase):

    def test_example_works(self):
        fatcat.dummy.insert_example_works()

    def test_random_works(self):
        fatcat.dummy.insert_random_works()

    def test_load_crossref(self):
        with open('./tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
            raw = [json.loads(l) for l in f.readlines() if len(l) > 3]
        for obj in raw:
            fatcat.sql.add_crossref_via_model(obj)

    def test_schema_release_rev(self):
        assert ReleaseRev.query.count() == 0
        e = {
            "title": "Bogus title",
            "release_type": "book",
            "creators": [],
            "refs": [],
        }
        model = release_rev_schema.load(e)
        fatcat.db.session.add(model.data)
        fatcat.db.session.commit()
        assert ReleaseRev.query.count() == 1
        model_after = ReleaseRev.query.first()
        serial = release_rev_schema.dump(model_after).data
        #check_release(serial)
        for k in e.keys():
            assert e[k] == serial[k]

    def test_schema_creator_rev(self):
        assert ReleaseRev.query.count() == 0
        e = {
            "name": "Robin (Batman)",
        }
        model = creator_rev_schema.load(e)
        fatcat.db.session.add(model.data)
        fatcat.db.session.commit()
        assert CreatorRev.query.count() == 1
        model_after = CreatorRev.query.first()
        serial = creator_rev_schema.dump(model_after).data
        check_creator(serial)
        for k in e.keys():
            assert e[k] == serial[k]

    def test_schema_container_rev(self):
        assert ReleaseRev.query.count() == 0
        e = {
            "name": "Papers Monthly",
        }
        model = container_rev_schema.load(e)
        fatcat.db.session.add(model.data)
        fatcat.db.session.commit()
        assert ContainerRev.query.count() == 1
        model_after = ContainerRev.query.first()
        serial = container_rev_schema.dump(model_after).data
        check_container(serial)
        for k in e.keys():
            assert e[k] == serial[k]

    def test_schema_file_rev(self):
        assert ReleaseRev.query.count() == 0
        e = {
            "sha1": "asdf",
            "size": 6,
        }
        model = file_rev_schema.load(e)
        print(model)
        fatcat.db.session.add(model.data)
        fatcat.db.session.commit()
        assert FileRev.query.count() == 1
        model_after = FileRev.query.first()
        serial = file_rev_schema.dump(model_after).data
        check_file(serial)
        for k in e.keys():
            assert e[k] == serial[k]

## API Tests #################################################################

class APITestCase(FatcatTestCase):

    def test_health(self):
        rv = self.app.get('/health')
        obj = json.loads(rv.data.decode('utf-8'))
        assert obj['ok']

    def test_api_work(self):
        fatcat.dummy.insert_example_works()

        # Invalid Id
        rv = self.app.get('/v0/work/_')
        assert rv.status_code == 404

        # Random
        rv = self.app.get('/v0/work/random')
        rv = self.app.get(rv.location)
        work = json.loads(rv.data.decode('utf-8'))
        check_entity_fields(work)
        print(work)
        assert work['title']
        assert work['work_type']

        # Valid Id (from random above)
        rv = self.app.get('/v0/work/{}'.format(work['id']))
        assert rv.status_code == 200

        # Missing Id
        rv = self.app.get('/v0/work/r3zga5b9cd7ef8gh084714iljk')
        assert rv.status_code == 404

    def test_api_work_create(self):
        assert WorkIdent.query.count() == 0
        assert WorkRev.query.count() == 0
        assert WorkEdit.query.count() == 0
        assert ExtraJson.query.count() == 0
        rv = self.app.post('/v0/work',
            data=json.dumps(dict(title="dummy", work_type="thing", extra=dict(a=1, b="zing"))),
            headers={"content-type": "application/json"})
        print(rv)
        assert rv.status_code == 200
        assert WorkIdent.query.count() == 1
        assert WorkRev.query.count() == 1
        assert WorkEdit.query.count() == 1
        assert ExtraJson.query.count() == 1
        # not alive yet
        assert WorkIdent.query.filter(WorkIdent.is_live==True).count() == 0

    def test_api_rich_create(self):

        # TODO: create user?

        rv = self.app.post('/v0/editgroup',
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

        rv = self.app.post('/v0/container',
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

        rv = self.app.post('/v0/creator',
            data=json.dumps(dict(
                name="anon y. mouse",
                orcid="0000-0002-1825-0097",
                editgroup=editgroup_id,
                extra=dict(w=1, q="zing"))),
            headers={"content-type": "application/json"})
        assert rv.status_code == 200
        obj = json.loads(rv.data.decode('utf-8'))
        creator_id = obj['id']

        rv = self.app.post('/v0/work',
            data=json.dumps(dict(
                title="dummy work",
                work_type="book",
                editgroup=editgroup_id,
                extra=dict(a=3, b="zing"))),
            headers={"content-type": "application/json"})
        assert rv.status_code == 200
        obj = json.loads(rv.data.decode('utf-8'))
        work_id = obj['id']

        rv = self.app.post('/v0/release',
            data=json.dumps(dict(
                title="dummy work",
                work_type="book",
                work=work_id,
                container=container_id,
                creators=[creator_id],
                doi="10.1234/5678",
                editgroup=editgroup_id,
                refs=[
                    dict(stub="some other journal article"),
                ],
                extra=dict(f=7, b="zing"))),
            headers={"content-type": "application/json"})
        assert rv.status_code == 200
        obj = json.loads(rv.data.decode('utf-8'))
        release_id = obj['id']

        rv = self.app.post('/v0/file',
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
                    ReleaseIdent, ReleaseRev, ReleaseEdit,
                    FileIdent, FileRev, FileEdit):
            assert cls.query.count() == 1

        for cls in (WorkIdent,
                    ContainerIdent,
                    CreatorIdent,
                    ReleaseIdent,
                    FileIdent):
            assert cls.query.filter(cls.is_live==True).count() == 0

        rv = self.app.post('/v0/editgroup/{}/accept'.format(editgroup_id),
            headers={"content-type": "application/json"})
        assert rv.status_code == 200
        assert ChangelogEntry.query.count() == 1

        for cls in (WorkIdent, WorkRev, WorkEdit,
                    ContainerIdent, ContainerRev, ContainerEdit,
                    CreatorIdent, CreatorRev, CreatorEdit,
                    ReleaseIdent, ReleaseRev, ReleaseEdit,
                    FileIdent, FileRev, FileEdit):
            assert cls.query.count() == 1

        for cls in (WorkIdent,
                    ContainerIdent,
                    CreatorIdent,
                    ReleaseIdent,
                    FileIdent):
            assert cls.query.filter(cls.is_live==True).count() == 1

        # Test that foreign key relations worked
        release_rv = json.loads(self.app.get('/v0/release/{}'.format(release_id)).data.decode('utf-8'))
        print(release_rv)
        assert(release_rv['creators'][0]['creator'] == creator_id)
        assert(release_rv['container']['id'] == container_id)
        assert(release_rv['work']['id'] == work_id)

        file_rv = json.loads(self.app.get('/v0/file/{}'.format(file_id)).data.decode('utf-8'))
        print(file_rv)
        assert(file_rv['releases'][0]['release'] == release_id)
