
import json
import pytest
from copy import copy

from fatcat_openapi_client import *
from fatcat_openapi_client.rest import ApiException
from fixtures import *

"""
This file has API and webface tests for when an entity references another
entity which is deleted/redirected/wip. For example, a release points to a
container (via container_id) which is deleted.

Current set of such references:

    => release -> collection
    => release -> creator
    => release -> file
    => file -> release
    => release -> work
    => work -> release
"""

def test_relation_states(api, app):

    j1 = ContainerEntity(name="test journal")
    j2 = ContainerEntity(name="another test journal")
    c1 = CreatorEntity(display_name="test person")
    r1 = ReleaseEntity(title="test article", ext_ids=ReleaseExtIds())
    r2 = ReleaseEntity(title="another test article", ext_ids=ReleaseExtIds())
    f1 = FileEntity(md5="c60f5b093ef5e6caad9d7b45268be409")
    f2 = FileEntity(md5="0000000000000000ad9d7b45268be409")

    # WIP container
    eg = quick_eg(api)
    j2 = api.get_container(api.create_container(eg.editgroup_id, j2).ident)
    rv = app.get('/container/{}'.format(j2.ident))
    assert rv.status_code == 200

    # create inter-related entities
    eg = quick_eg(api)
    j1 = api.get_container(api.create_container(eg.editgroup_id, j1).ident)
    c1 = api.get_creator(api.create_creator(eg.editgroup_id, c1).ident)
    r1.container_id = j1.ident
    r1.contribs = [ReleaseContrib(creator_id=c1.ident)]
    r1 = api.get_release(api.create_release(eg.editgroup_id, r1).ident)
    r2 = api.get_release(api.create_release(eg.editgroup_id, r2).ident)
    f1.release_ids = [r1.ident]
    f1 = api.get_file(api.create_file(eg.editgroup_id, f1).ident)
    api.accept_editgroup(eg.editgroup_id)

    r1 = api.get_release(r1.ident, expand="container,creators,files")
    assert r1.container.name == "test journal"
    assert r1.files[0].md5 == "c60f5b093ef5e6caad9d7b45268be409"
    assert r1.contribs[0].creator_id == c1.ident
    assert r1.contribs[0].creator.display_name == "test person"
    assert r1.contribs[0].creator.state == "active"
    rv = app.get('/release/{}'.format(r1.ident))
    assert rv.status_code == 200

    # delete creator
    eg = quick_eg(api)
    api.delete_creator(eg.editgroup_id, c1.ident)
    api.accept_editgroup(eg.editgroup_id)
    rv = app.get('/creator/{}'.format(c1.ident))
    assert rv.status_code == 200 # TODO: HTTP status "Gone"?

    c1_deleted = api.get_creator(c1.ident)
    assert c1_deleted.state == "deleted"
    assert c1_deleted.display_name is None

    r1 = api.get_release(r1.ident, expand="container,creators,files")
    assert r1.container.name == "test journal"
    assert r1.files[0].md5 == "c60f5b093ef5e6caad9d7b45268be409"
    assert r1.contribs[0].creator_id == c1.ident
    assert r1.contribs[0].creator.display_name is None
    assert r1.contribs[0].creator.state == "deleted"
    rv = app.get('/release/{}'.format(r1.ident))
    assert rv.status_code == 200

    # wip container
    eg = quick_eg(api)
    r1.container_id = j2.ident
    api.update_release(eg.editgroup_id, r1.ident, r1)
    api.accept_editgroup(eg.editgroup_id)

    r1 = api.get_release(r1.ident, expand="container,creators,files")
    assert r1.container_id == j2.ident
    assert r1.container.name == "another test journal"
    assert r1.container.state == "wip"
    assert r1.files[0].md5 == "c60f5b093ef5e6caad9d7b45268be409"
    assert r1.contribs[0].creator_id == c1.ident
    assert r1.contribs[0].creator.display_name is None
    assert r1.contribs[0].creator.state == "deleted"
    rv = app.get('/release/{}'.format(r1.ident))
    assert rv.status_code == 200

    # redirect release
    r2 = api.get_release(r2.ident, expand="container,creators,files")
    assert r2.files == []
    eg = quick_eg(api)
    api.update_release(eg.editgroup_id, r2.ident, ReleaseEntity(redirect=r1.ident, ext_ids=ReleaseExtIds()))
    f2.release_ids = [r2.ident]
    f2 = api.get_file(api.create_file(eg.editgroup_id, f2).ident)
    api.accept_editgroup(eg.editgroup_id)
    r2 = api.get_release(r2.ident, expand="container,creators,files")
    assert r2.container_id == j2.ident
    assert r2.container.name == "another test journal"
    assert r2.container.state == "wip"
    # fetching for *target*; tricky!
    assert r2.files[0].md5 == "c60f5b093ef5e6caad9d7b45268be409"
    assert r2.contribs[0].creator_id == c1.ident
    assert r2.contribs[0].creator.display_name is None
    assert r2.contribs[0].creator.state == "deleted"
    rv = app.get('/release/{}'.format(r2.ident))
    assert rv.status_code == 302
    rv = app.get('/file/{}'.format(f2.ident))
    assert rv.status_code == 200

    # delete release
    eg = quick_eg(api)
    api.delete_release(eg.editgroup_id, r2.ident)
    api.accept_editgroup(eg.editgroup_id)
    r2 = api.get_release(r2.ident, expand="container,creators,files")
    assert r2.container_id is None
    assert r2.container is None
    assert r2.files is None
    assert r2.contribs is None
    rv = app.get('/release/{}'.format(r2.ident))
    assert rv.status_code == 200 # TODO: HTTP Gone?
    rv = app.get('/file/{}'.format(f2.ident))
    print(rv.data)
    assert rv.status_code == 200


def test_app_entity_states(api, app):

    j1 = ContainerEntity(name="test journal")
    j2 = ContainerEntity(name="another test journal")
    c1 = CreatorEntity(display_name="test person")
    c2 = CreatorEntity(display_name="another test person")
    r1 = ReleaseEntity(title="test article", ext_ids=ReleaseExtIds())
    r2 = ReleaseEntity(title="another test article", ext_ids=ReleaseExtIds())
    f1 = FileEntity(md5="c60f5b093ef5e6caad9d7b45268be409")
    f2 = FileEntity(md5="0000000000000000ad9d7b45268be409")

    # create inter-related entities
    eg = quick_eg(api)
    j1 = api.get_container(api.create_container(eg.editgroup_id, j1).ident)
    j2 = api.get_container(api.create_container(eg.editgroup_id, j2).ident)
    c1 = api.get_creator(api.create_creator(eg.editgroup_id, c1).ident)
    c2 = api.get_creator(api.create_creator(eg.editgroup_id, c2).ident)
    r1.container_id = j1.ident
    r1.contribs = [ReleaseContrib(creator_id=c1.ident)]
    r1 = api.get_release(api.create_release(eg.editgroup_id, r1).ident)
    r2 = api.get_release(api.create_release(eg.editgroup_id, r2).ident)
    f1.release_ids = [r1.ident]
    f1 = api.get_file(api.create_file(eg.editgroup_id, f1).ident)
    f2 = api.get_file(api.create_file(eg.editgroup_id, f2).ident)
    api.accept_editgroup(eg.editgroup_id)

    # create redirects
    eg = quick_eg(api)
    api.update_container(eg.editgroup_id, j2.ident, ContainerEntity(redirect=j1.ident))
    api.update_creator(eg.editgroup_id, c2.ident, CreatorEntity(redirect=c1.ident))
    api.update_file(eg.editgroup_id, f2.ident, FileEntity(redirect=f1.ident))
    api.update_release(eg.editgroup_id, r2.ident, ReleaseEntity(redirect=r1.ident, ext_ids=ReleaseExtIds()))
    api.update_work(eg.editgroup_id, r2.work_id, WorkEntity(redirect=r1.work_id))
    api.accept_editgroup(eg.editgroup_id)

    # all entities
    rv = app.get('/container/{}'.format(j1.ident))
    assert rv.status_code == 200
    rv = app.get('/container/{}'.format(j2.ident))
    assert rv.status_code == 302
    rv = app.get('/creator/{}'.format(c1.ident))
    assert rv.status_code == 200
    rv = app.get('/creator/{}'.format(c2.ident))
    assert rv.status_code == 302
    rv = app.get('/file/{}'.format(f1.ident))
    assert rv.status_code == 200
    rv = app.get('/file/{}'.format(f2.ident))
    assert rv.status_code == 302
    rv = app.get('/release/{}'.format(r1.ident))
    assert rv.status_code == 200
    rv = app.get('/release/{}'.format(r2.ident))
    assert rv.status_code == 302
    rv = app.get('/work/{}'.format(r1.work_id))
    assert rv.status_code == 200
    rv = app.get('/work/{}'.format(r2.work_id))
    assert rv.status_code == 302

    # delete targets
    eg = quick_eg(api)
    api.delete_container(eg.editgroup_id, j1.ident)
    api.delete_creator(eg.editgroup_id, c1.ident)
    api.delete_file(eg.editgroup_id, f1.ident)
    api.delete_release(eg.editgroup_id, r1.ident)
    api.delete_work(eg.editgroup_id, r1.work_id)
    api.accept_editgroup(eg.editgroup_id)

    # all entities
    rv = app.get('/container/{}'.format(j1.ident))
    assert rv.status_code == 200
    rv = app.get('/container/{}'.format(j2.ident))
    assert rv.status_code == 302
    rv = app.get('/creator/{}'.format(c1.ident))
    assert rv.status_code == 200
    rv = app.get('/creator/{}'.format(c2.ident))
    assert rv.status_code == 302
    rv = app.get('/file/{}'.format(f1.ident))
    assert rv.status_code == 200
    rv = app.get('/file/{}'.format(f2.ident))
    assert rv.status_code == 302
    rv = app.get('/release/{}'.format(r1.ident))
    assert rv.status_code == 200
    rv = app.get('/release/{}'.format(r2.ident))
    assert rv.status_code == 302
    rv = app.get('/work/{}'.format(r1.work_id))
    assert rv.status_code == 200
    rv = app.get('/work/{}'.format(r2.work_id))
    assert rv.status_code == 302

