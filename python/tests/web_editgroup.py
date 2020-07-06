
from fixtures import *


def test_editgroup_basics(app):

    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaae')
    assert rv.status_code == 200
    rv = app.get('/editgroup/ccccccccccccccccccccccccca')
    assert rv.status_code == 404

    rv = app.get('/editor/aaaaaaaaaaaabkvkaaaaaaaaae')
    assert rv.status_code == 200
    rv = app.get('/editor/ccccccccccccccccccccccccca')
    assert rv.status_code == 404
    rv = app.get('/editor/aaaaaaaaaaaabkvkaaaaaaaaae/editgroups')
    assert rv.status_code == 200
    rv = app.get('/editor/ccccccccccccccccccccccccca/editgroups')
    assert rv.status_code == 404

    rv = app.get('/changelog')
    assert rv.status_code == 200
    rv = app.get('/changelog/1')
    assert rv.status_code == 200
    rv = app.get('/changelog/1.3')
    assert rv.status_code == 404
    rv = app.get('/changelog/9999999')
    assert rv.status_code == 404

    rv = app.get('/reviewable')
    assert rv.status_code == 200

def test_editgroup_annotations(app):

    rv = app.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaa4')
    assert rv.status_code == 200
    assert b'updated with changes, please re-review' in rv.data
    assert b'concerns about this edit...' in rv.data
    assert b'Admin' in rv.data
    assert b'demo-user' in rv.data
    assert b'claire' in rv.data
    assert b'Signup' in rv.data
    assert b'Add Comment' not in rv.data

    rv = app.get('/editor/aaaaaaaaaaaabkvkaaaaaaaaaq/annotations')
    assert rv.status_code == 200
    assert b'updated with changes, please re-review' not in rv.data
    assert b'concerns about this edit...' in rv.data
    assert b'Admin' not in rv.data
    assert b'claire' in rv.data
    assert b'aaaaaaaaaaaabo53aaaaaaaaa4' in rv.data

def test_editgroup_annotations_login(app_admin):

    # if logged in, should see form
    rv = app_admin.get('/editgroup/aaaaaaaaaaaabo53aaaaaaaaa4')
    assert rv.status_code == 200
    assert b'Signup' not in rv.data
    assert b'Add Comment' in rv.data
