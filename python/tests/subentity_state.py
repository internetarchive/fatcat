
import json
import pytest
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
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
    r1 = ReleaseEntity(title="test article")
    r2 = ReleaseEntity(title="another test article")
    f1 = FileEntity(md5="c60f5b093ef5e6caad9d7b45268be409")
    f2 = FileEntity(md5="0000000000000000ad9d7b45268be409")

    # WIP container
    eg = quick_eg(api)
    j2 = api.get_container(api.create_container(j2, editgroup_id=eg.editgroup_id).ident)
    rv = app.get('/container/{}'.format(j2.ident))
    assert rv.status_code == 200

    # create inter-related entities
    eg = quick_eg(api)
    j1 = api.get_container(api.create_container(j1, editgroup_id=eg.editgroup_id).ident)
    c1 = api.get_creator(api.create_creator(c1, editgroup_id=eg.editgroup_id).ident)
    r1.container_id = j1.ident
    r1.contribs = [ReleaseContrib(creator_id=c1.ident)]
    r1 = api.get_release(api.create_release(r1, editgroup_id=eg.editgroup_id).ident)
    r2 = api.get_release(api.create_release(r2, editgroup_id=eg.editgroup_id).ident)
    f1.release_ids = [r1.ident]
    f1 = api.get_file(api.create_file(f1, editgroup_id=eg.editgroup_id).ident)
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
    api.delete_creator(c1.ident, editgroup_id=eg.editgroup_id)
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
    api.update_release(r1.ident, r1, editgroup_id=eg.editgroup_id)
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
    api.update_release(r2.ident, ReleaseEntity(redirect=r1.ident), editgroup_id=eg.editgroup_id)
    f2.release_ids = [r2.ident]
    f2 = api.get_file(api.create_file(f2, editgroup_id=eg.editgroup_id).ident)
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
    api.delete_release(r2.ident, editgroup_id=eg.editgroup_id)
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
    r1 = ReleaseEntity(title="test article")
    r2 = ReleaseEntity(title="another test article")
    f1 = FileEntity(md5="c60f5b093ef5e6caad9d7b45268be409")
    f2 = FileEntity(md5="0000000000000000ad9d7b45268be409")

    # create inter-related entities
    eg = quick_eg(api)
    j1 = api.get_container(api.create_container(j1, editgroup_id=eg.editgroup_id).ident)
    j2 = api.get_container(api.create_container(j2, editgroup_id=eg.editgroup_id).ident)
    c1 = api.get_creator(api.create_creator(c1, editgroup_id=eg.editgroup_id).ident)
    c2 = api.get_creator(api.create_creator(c2, editgroup_id=eg.editgroup_id).ident)
    r1.container_id = j1.ident
    r1.contribs = [ReleaseContrib(creator_id=c1.ident)]
    r1 = api.get_release(api.create_release(r1, editgroup_id=eg.editgroup_id).ident)
    r2 = api.get_release(api.create_release(r2, editgroup_id=eg.editgroup_id).ident)
    f1.release_ids = [r1.ident]
    f1 = api.get_file(api.create_file(f1, editgroup_id=eg.editgroup_id).ident)
    f2 = api.get_file(api.create_file(f2, editgroup_id=eg.editgroup_id).ident)
    api.accept_editgroup(eg.editgroup_id)

    # create redirects
    eg = quick_eg(api)
    api.update_container(j2.ident, ContainerEntity(redirect=j1.ident), editgroup_id=eg.editgroup_id)
    api.update_creator(c2.ident, CreatorEntity(redirect=c1.ident), editgroup_id=eg.editgroup_id)
    api.update_file(f2.ident, FileEntity(redirect=f1.ident), editgroup_id=eg.editgroup_id)
    api.update_release(r2.ident, ReleaseEntity(redirect=r1.ident), editgroup_id=eg.editgroup_id)
    api.update_work(r2.work_id, WorkEntity(redirect=r1.work_id), editgroup_id=eg.editgroup_id)
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
    api.delete_container(j1.ident, editgroup_id=eg.editgroup_id)
    api.delete_creator(c1.ident, editgroup_id=eg.editgroup_id)
    api.delete_file(f1.ident, editgroup_id=eg.editgroup_id)
    api.delete_release(r1.ident, editgroup_id=eg.editgroup_id)
    api.delete_work(r1.work_id, editgroup_id=eg.editgroup_id)
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

