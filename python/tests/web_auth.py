import responses
from fixtures import *


@responses.activate
def test_ia_xauth_fail(full_app):

    # failed login
    with full_app.test_client() as app:

        rv = app.get("/auth/ia/login")
        assert rv.status_code == 200

        responses.add(
            responses.POST,
            full_app.config["IA_XAUTH_URI"] + "?op=authenticate",
            status=401,
            json=dict(success=False),
        )
        rv = app.post(
            "/auth/ia/login",
            follow_redirects=True,
            data=dict(email="abcd@example.com", password="god"),
        )
        assert rv.status_code == 401

        rv = app.get("/auth/account", follow_redirects=False)
        assert rv.status_code == 302


@responses.activate
def test_ia_xauth(full_app):

    # successful login
    with full_app.test_client() as app:

        rv = app.get("/auth/token_login")
        assert rv.status_code == 200

        responses.add(
            responses.POST,
            full_app.config["IA_XAUTH_URI"] + "?op=authenticate",
            status=200,
            json={"success": True},
        )
        responses.add(
            responses.POST,
            full_app.config["IA_XAUTH_URI"] + "?op=info",
            status=200,
            json={
                "success": True,
                "values": {"screenname": "user123", "itemname": "user_item123"},
            },
        )
        rv = app.post(
            "/auth/ia/login",
            follow_redirects=True,
            data=dict(email="abcd@example.com", password="god"),
        )
        assert rv.status_code == 200

        rv = app.get("/auth/account", follow_redirects=False)
        assert rv.status_code == 200


def test_basic_auth_views(app):

    rv = app.get("/auth/login")
    assert rv.status_code == 200

    rv = app.get("/auth/logout")
    assert rv.status_code == 200


def test_auth_token(app_admin):

    rv = app_admin.get("/auth/account", follow_redirects=False)
    assert rv.status_code == 200

    rv = app_admin.post("/auth/create_token", follow_redirects=False)
    assert rv.status_code == 200
