
import json
import pytest
import unittest
import tempfile
import fatcat
import fatcat.sql
from fatcat.models import *
from fixtures import *


def test_merge_works(app):

    # two works, each with releases
    rv = app.post('/v0/work',
        data=json.dumps(dict()),
        headers={"content-type": "application/json"})
    workA_id = json.loads(rv.data.decode('utf-8'))['id']

    rv = app.post('/v0/work',
        data=json.dumps(dict()),
        headers={"content-type": "application/json"})
    workB_id = json.loads(rv.data.decode('utf-8'))['id']

    rv = app.post('/v0/release',
        data=json.dumps(dict(
            title="some release",
            work_type="journal-article",
            work=workA_id,
            doi="10.1234/A1")),
        headers={"content-type": "application/json"})
    releaseA1 = json.loads(rv.data.decode('utf-8'))['id']

    rv = app.post('/v0/release',
        data=json.dumps(dict(
            title="some release",
            work_type="journal-article",
            work=workB_id,
            doi="10.1234/B1")),
        headers={"content-type": "application/json"})
    releaseB1 = json.loads(rv.data.decode('utf-8'))['id']

    rv = app.post('/v0/release',
        data=json.dumps(dict(
            title="some release",
            work_type="journal-article",
            work=workB_id,
            doi="10.1234/A1")),
        headers={"content-type": "application/json"})
    releaseB2 = json.loads(rv.data.decode('utf-8'))['id']

    # XXX: what if workB primary was set?

    editgroup_id = 1
    rv = app.post('/v0/editgroup/{}/accept'.format(editgroup_id),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200
    assert ChangelogEntry.query.count() == 1
    assert WorkIdent.query.filter(WorkIdent.is_live==True).count() == 2
    assert ReleaseIdent.query.filter(ReleaseIdent.is_live==True).count() == 3

    # merge works
    fatcat.sql.merge_works(workA_id, workB_id)
    editgroup_id = 2
    rv = app.post('/v0/editgroup/{}/accept'.format(editgroup_id),
        headers={"content-type": "application/json"})
    assert rv.status_code == 200

    # check results
    assert ChangelogEntry.query.count() == 2
    assert WorkIdent.query.filter(WorkIdent.is_live==True).count() == 2
    assert ReleaseIdent.query.filter(ReleaseIdent.is_live==True).count() == 3

    workA_json = json.loads(app.get('/v0/work/{}'.format(workA_id)).data.decode('utf-8'))
    workB_json = json.loads(app.get('/v0/work/{}'.format(workB_id)).data.decode('utf-8'))
    assert workA_json['rev'] == workB_json['rev']
    print(workA_json)
    print(workB_json)
    assert workA_json['redirect_id'] == None
    assert workB_json['redirect_id'] == workA_json['id']
