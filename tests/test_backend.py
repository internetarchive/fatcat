
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
    for key in ('id', 'rev', 'previous', 'state', 'redirect_id', 'edit_id',
            'extra_json'):
        assert key in e
    for key in ('id', 'rev'):
        assert e[key] is not None

## API Tests ################################################################

class FatcatTestCase(unittest.TestCase):

    def setUp(self):
        fatcat.app.config['DATABASE_URI'] = 'sqlite://:memory:'
        fatcat.app.testing = True
        self.app = fatcat.app.test_client()
        fatcat.db.create_all()

    def test_health(self):
        rv = self.app.get('/health')
        obj = json.loads(rv.data.decode('utf-8'))
        assert obj['ok']

    def test_works(self):

        # Invalid Id
        rv = self.app.get('/v0/work/_')
        assert rv.status_code == 404

        # Missing Id (TODO)
        #rv = self.app.get('/v0/work/rzga5b9cd7efgh04iljk')
        #assert rv.status == 404

        # Valid Id (TODO)
        #rv = self.app.get('/v0/work/r3zga5b9cd7ef8gh084714iljk')
        #assert rv.status_code == 200

        rv = self.app.get('/v0/work/random')
        obj = json.loads(rv.data.decode('utf-8'))
        check_entity_fields(obj)
        assert obj['title']
        assert obj['work_type'] == "journal-article"

    def test_populate(self):
        fatcat.sql.populate_db()

    def test_populate_complex(self):
        fatcat.sql.populate_complex_db()
