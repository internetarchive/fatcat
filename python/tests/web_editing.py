
import json
import pytest
from fatcat_client.rest import ApiException
from fixtures import *


def test_web_release_create_merge(app_admin, api):

    eg = quick_eg(api)

    rv = app_admin.get('/editgroup/{}'.format(eg.editgroup_id))
    assert rv.status_code == 200
    assert b'Release Edits (0)' in rv.data

    # bogus/bad submit
    rv = app_admin.post('/release/create',
        data={
            'editgroup_id': eg.editgroup_id,
            'release_type': 'badmojo',
            'release_status': 'published',
            'title': 'something bogus',
        },
        follow_redirects=True)
    assert rv.status_code == 400
    # Does not insert bad choices into drop-down
    #assert b'badmojo' in rv.data
    assert b'Not a valid choice' in rv.data

    # ok/valid submit
    rv = app_admin.post('/release/create',
        data={
            'editgroup_id': eg.editgroup_id,
            'release_type': 'article-journal',
            'release_status': 'published',
            'title': 'something bogus',
        },
        follow_redirects=True)
    assert rv.status_code == 200

    rv = app_admin.get('/editgroup/{}'.format(eg.editgroup_id))
    assert rv.status_code == 200
    assert b'Release Edits (1)' in rv.data

    rv = app_admin.post('/editgroup/{}/submit'.format(eg.editgroup_id),
        follow_redirects=True,
    )
    assert rv.status_code == 200
    rv = app_admin.get('/editgroup/{}'.format(eg.editgroup_id))
    assert rv.status_code == 200
    assert b'Submitted' in rv.data
    assert b'None!' in rv.data

    rv = app_admin.post('/editgroup/{}/annotation'.format(eg.editgroup_id),
        data={
            'comment_markdown': "This is an **example** of markdown in a test annotation",
        },
        follow_redirects=True,
    )
    assert rv.status_code == 200
    rv = app_admin.get('/editgroup/{}'.format(eg.editgroup_id))
    assert rv.status_code == 200
    assert b'<p>This is an <strong>example</strong> of markdown in a test annotation' in rv.data

    rv = app_admin.post('/editgroup/{}/accept'.format(eg.editgroup_id),
        follow_redirects=True,
    )
    assert rv.status_code == 200
    rv = app_admin.get('/editgroup/{}'.format(eg.editgroup_id))
    assert rv.status_code == 200
    assert b'Merged' in rv.data


def test_web_container_create(app_admin, api):

    eg = quick_eg(api)

    # bogus/bad submit
    rv = app_admin.post('/container/create',
        data={
            'editgroup_id': eg.editgroup_id,
        },
        follow_redirects=True)
    assert rv.status_code == 400

    # ok/valid submit
    rv = app_admin.post('/container/create',
        data={
            'editgroup_id': eg.editgroup_id,
            'name': 'blah blah journal blah',
        },
        follow_redirects=True)
    assert rv.status_code == 200
