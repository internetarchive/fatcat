
import os
import json
import pytest
import fatcat
import fatcat.sql
import unittest
import tempfile

# TODO: http://alextechrants.blogspot.com/2013/08/unit-testing-sqlalchemy-apps.html

## Helpers ##################################################################

def check_entity_fields(e):
    for key in ('id', 'rev', 'is_live', 'redirect_id'):
        assert key in e
    for key in ('id',):
        assert e[key] is not None

## API Tests ################################################################

class FatcatTestCase(unittest.TestCase):

    def setUp(self):
        fatcat.app.config['SQLALCHEMY_DATABASE_URI'] = 'sqlite://'
        fatcat.app.testing = True
        self.app = fatcat.app.test_client()
        fatcat.db.session.remove()
        fatcat.db.drop_all()
        fatcat.db.create_all()
        fatcat.sql.populate_db()

    def test_health(self):
        rv = self.app.get('/health')
        obj = json.loads(rv.data.decode('utf-8'))
        assert obj['ok']

    def test_works(self):
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

    def test_populate(self):
        fatcat.sql.populate_db()

    def test_example_works(self):
        fatcat.dummy.insert_example_works()

    def test_random_works(self):
        fatcat.dummy.insert_random_works()

    def test_load_crossref(self):
        with open('./tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
            raw = [json.loads(l) for l in f.readlines() if len(l) > 3]
        for obj in raw:
            fatcat.sql.add_crossref(obj)

    def test_hydrate_work(self):
        fatcat.dummy.insert_random_works()
        fatcat.sql.hydrate_work(1)

    def test_hydrate_release(self):
        fatcat.dummy.insert_random_works()
        fatcat.sql.hydrate_release(1)
