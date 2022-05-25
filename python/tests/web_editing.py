from fixtures import *


def test_web_release_create_accept(app_admin, api):

    eg = quick_eg(api)

    rv = app_admin.get(f"/editgroup/{eg.editgroup_id}")
    assert rv.status_code == 200
    assert b"Release Edits (0)" in rv.data

    # bogus/bad submit
    rv = app_admin.post(
        "/release/create",
        data={
            "editgroup_id": eg.editgroup_id,
            "release_type": "badmojo",
            "release_stage": "published",
            "title": "something bogus",
        },
        follow_redirects=True,
    )
    assert rv.status_code == 400
    # Does not insert bad choices into drop-down
    # assert b'badmojo' in rv.data
    assert b"Not a valid choice" in rv.data

    # bad wikidata QID
    rv = app_admin.post(
        "/release/create",
        data={
            "editgroup_id": eg.editgroup_id,
            "release_type": "article-journal",
            "release_stage": "published",
            "title": "something bogus",
            "wikidata_qid": "884",
        },
        follow_redirects=True,
    )
    assert rv.status_code == 400

    # ok/valid submit
    rv = app_admin.post(
        "/release/create",
        data={
            "editgroup_id": eg.editgroup_id,
            "release_type": "article-journal",
            "release_stage": "published",
            "title": "something bogus",
            "doi": "10.1234/999999",
        },
        follow_redirects=True,
    )
    assert rv.status_code == 200
    assert b"10.1234/999999" in rv.data

    rv = app_admin.get(f"/editgroup/{eg.editgroup_id}")
    assert rv.status_code == 200
    assert b"Release Edits (1)" in rv.data

    rv = app_admin.post(
        f"/editgroup/{eg.editgroup_id}/submit",
        follow_redirects=True,
    )
    assert rv.status_code == 200
    rv = app_admin.get(f"/editgroup/{eg.editgroup_id}")
    assert rv.status_code == 200
    assert b"Submitted" in rv.data
    assert b"None!" in rv.data

    rv = app_admin.post(
        f"/editgroup/{eg.editgroup_id}/unsubmit",
        follow_redirects=True,
    )
    assert rv.status_code == 200
    rv = app_admin.get(f"/editgroup/{eg.editgroup_id}")
    assert rv.status_code == 200

    rv = app_admin.post(
        f"/editgroup/{eg.editgroup_id}/annotation",
        data={
            "comment_markdown": "This is an **example** of markdown in a test annotation",
        },
        follow_redirects=True,
    )
    assert rv.status_code == 200
    rv = app_admin.get(f"/editgroup/{eg.editgroup_id}")
    assert rv.status_code == 200
    assert b"<p>This is an <strong>example</strong> of markdown in a test annotation" in rv.data

    rv = app_admin.post(
        f"/editgroup/{eg.editgroup_id}/accept",
        follow_redirects=True,
    )
    assert rv.status_code == 200
    rv = app_admin.get(f"/editgroup/{eg.editgroup_id}")
    assert rv.status_code == 200
    assert b"Merged" in rv.data


def test_web_container_create(app_admin, api):

    eg = quick_eg(api)

    # bogus/bad submit
    rv = app_admin.post(
        "/container/create",
        data={
            "editgroup_id": eg.editgroup_id,
        },
        follow_redirects=True,
    )
    assert rv.status_code == 400

    # ok/valid submit
    rv = app_admin.post(
        "/container/create",
        data={
            "editgroup_id": eg.editgroup_id,
            "name": "blah blah journal blah",
        },
        follow_redirects=True,
    )
    assert rv.status_code == 200


def test_web_file_create(app_admin, api):

    eg = quick_eg(api)

    # bogus/bad submit
    rv = app_admin.post(
        "/file/create",
        data={
            "editgroup_id": eg.editgroup_id,
        },
        follow_redirects=True,
    )
    assert rv.status_code == 400

    # ok/valid submit
    rv = app_admin.post(
        "/file/create",
        data={
            "editgroup_id": eg.editgroup_id,
            "size": "12345",
            "sha1": "45be56a396c4d03faaa41e055170c23534dec736",
        },
        follow_redirects=True,
    )
    assert rv.status_code == 200

    # upper-case SHA-1
    rv = app_admin.post(
        "/file/create",
        data={
            "editgroup_id": eg.editgroup_id,
            "size": "12345",
            "sha1": "45BE56A396C4D03FAAA41E055170C23534DEC736",
        },
        follow_redirects=True,
    )
    assert rv.status_code == 200


def test_web_file_toml_create(app_admin, api):

    eg = quick_eg(api)

    # bogus/bad submit
    rv = app_admin.post(
        "/file/create/toml",
        data={
            "editgroup_id": eg.editgroup_id,
        },
        follow_redirects=True,
    )
    assert rv.status_code == 400

    # ok/valid submit
    rv = app_admin.post(
        "/file/create/toml",
        data={
            "editgroup_id": eg.editgroup_id,
            "toml": """
size = 12345
sha1 = "45be56a396c4d03faaa41e055170c23534dec736"
            """,
        },
        follow_redirects=True,
    )
    assert rv.status_code == 200

    # upper-case SHA-1
    rv = app_admin.post(
        "/file/create/toml",
        data={
            "editgroup_id": eg.editgroup_id,
            "toml": """
size = 12345
sha1 = "45BE56A396C4D03FAAA41E055170C23534DEC736"
            """,
        },
        follow_redirects=True,
    )
    assert rv.status_code == 400


def test_web_file_delete(app_admin, api):

    eg = quick_eg(api)

    rv = app_admin.get("/file/aaaaaaaaaaaaamztaaaaaaaaam/delete")
    assert rv.status_code == 200

    rv = app_admin.post(
        "/file/aaaaaaaaaaaaamztaaaaaaaaam/delete",
        data={
            "editgroup_id": eg.editgroup_id,
        },
        follow_redirects=True,
    )
    assert rv.status_code == 200
    # NOTE: did not *accept* the deletion edit


DUMMY_DEMO_ENTITIES = {
    "container": "aaaaaaaaaaaaaeiraaaaaaaaam",
    "creator": "aaaaaaaaaaaaaircaaaaaaaaaq",
    "file": "aaaaaaaaaaaaamztaaaaaaaaam",
    "fileset": "aaaaaaaaaaaaaztgaaaaaaaaai",
    "webcapture": "aaaaaaaaaaaaa53xaaaaaaaaai",
    "release": "aaaaaaaaaaaaarceaaaaaaaaai",
    "work": "aaaaaaaaaaaaavkvaaaaaaaaai",
}


def test_web_edit_get(app_admin):

    # these are all existing entities
    for entity_type in ["release", "file", "container"]:
        rv = app_admin.get(f"/{entity_type}/{DUMMY_DEMO_ENTITIES[entity_type]}/edit")
        assert rv.status_code == 200
        if entity_type == "release":
            assert b"A bigger example" in rv.data
        elif entity_type == "file":
            assert (
                b"ffc1005680cb620eec4c913437dfabbf311b535cfe16cbaeb2faec1f92afc362" in rv.data
            )
        elif entity_type == "container":
            assert b"1549-1277" in rv.data

        rv = app_admin.get(f"/{entity_type}/{DUMMY_DEMO_ENTITIES[entity_type]}/edit/toml")
        assert rv.status_code == 200
        if entity_type == "release":
            assert b"A bigger example" in rv.data
        elif entity_type == "file":
            assert (
                b"ffc1005680cb620eec4c913437dfabbf311b535cfe16cbaeb2faec1f92afc362" in rv.data
            )
        elif entity_type == "container":
            assert b"1549-1277" in rv.data

    # TOML-only endpoints
    for entity_type in ["creator", "fileset", "webcapture", "work"]:
        rv = app_admin.get(f"/{entity_type}/{DUMMY_DEMO_ENTITIES[entity_type]}/edit")
        assert rv.status_code == 302

        rv = app_admin.get(f"/{entity_type}/{DUMMY_DEMO_ENTITIES[entity_type]}/edit/toml")
        assert rv.status_code == 200


def test_web_create_get(app_admin):

    for entity_type in ["release", "file", "container"]:
        rv = app_admin.get(f"/{entity_type}/create")
        assert rv.status_code == 200

        rv = app_admin.get(f"/{entity_type}/create/toml")
        assert rv.status_code == 200

    # these are TOML only
    for entity_type in ["creator", "fileset", "webcapture", "work"]:
        rv = app_admin.get(f"/{entity_type}/create")
        assert rv.status_code == 302

        rv = app_admin.get(f"/{entity_type}/create/toml")
        assert rv.status_code == 200


def test_web_edit_delete(app_admin):

    for entity_type in DUMMY_DEMO_ENTITIES.keys():
        rv = app_admin.get(f"/{entity_type}/{DUMMY_DEMO_ENTITIES[entity_type]}/delete")
        assert rv.status_code == 200
