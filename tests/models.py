
import json
import pytest
import unittest
import tempfile
import fatcat
import fatcat.sql
from fatcat.models import *
from fixtures import *


def test_example_works(app):
    fatcat.dummy.insert_example_works()

def test_random_works(app):
    fatcat.dummy.insert_random_works()

def test_load_crossref(app):
    with open('./tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        raw = [json.loads(l) for l in f.readlines() if len(l) > 3]
    for obj in raw:
        fatcat.sql.add_crossref_via_model(obj)

def test_schema_release_rev(app):
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

def test_schema_creator_rev(app):
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

def test_schema_container_rev(app):
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

def test_schema_file_rev(app):
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
