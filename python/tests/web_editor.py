
import json
import pytest
from fatcat_openapi_client.rest import ApiException
from fixtures import *


def test_change_username(app_admin):

    # these tests aren't supposed to mutate database
    rv = app_admin.post('/auth/change_username', data={'username': 'admin-tmp'},
        follow_redirects=True)
    assert rv.status_code == 200
    rv = app_admin.get('/auth/account')
    assert b'admin-tmp' in rv.data

    rv = app_admin.post('/auth/change_username', data={'username': 'claire'},
        follow_redirects=True)
    assert rv.status_code == 400
    rv = app_admin.get('/auth/account')
    assert b'admin-tmp' in rv.data

    rv = app_admin.post('/auth/change_username', data={'username': 'admin'},
        follow_redirects=True)
    assert rv.status_code == 200
    rv = app_admin.get('/auth/account')
    assert b'admin-tmp' not in rv.data

