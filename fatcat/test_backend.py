
import os
import json
import backend
import unittest
import tempfile
from nose.tools import *

# TODO: replace all these "assert" with unit test version (which displays left
# and right on failure)

# TODO: http://alextechrants.blogspot.com/2013/08/unit-testing-sqlalchemy-apps.html

## Helpers ##################################################################

def check_entity_fields(e):
    for key in ('id', 'rev', 'previous', 'state', 'redirect_id', 'edit_id',
            'extra_json'):
        assert_in(key, e)
    for key in ('id', 'rev'):
        assert_is_not_none(e[key])

## API Tests ################################################################

class BackendTestCase(unittest.TestCase):

    def setUp(self):
        backend.app.config['DATABASE_URI'] = 'sqlite://:memory:'
        backend.app.testing = True
        self.app = backend.app.test_client()

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
        #assert rv.status is 404

        # Valid Id
        rv = self.app.get('/v0/work/r3zga5b9cd7ef8gh084714iljk')
        assert rv.status_code == 200
        obj = json.loads(rv.data.decode('utf-8'))
        check_entity_fields(obj)
        assert obj['title']
        assert_equal(obj['work_type'], "journal-article")
