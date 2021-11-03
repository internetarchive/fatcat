
from fatcat_openapi_client import *
from fixtures import *


def test_citation_indexing(api):
    # indexing is consistent and reacts to change

    eg = quick_eg(api)
    r1 = ReleaseEntity(title="the target", ext_ids=ReleaseExtIds())
    r1.refs = [
        ReleaseRef(key="first", title="the first title"),
        ReleaseRef(key="second", title="the second title"),
        ReleaseRef(key="third", title="a third title"),
    ]
    r1 = api.get_release(api.create_release(eg.editgroup_id, r1).ident)
    api.accept_editgroup(eg.editgroup_id)

    assert r1.refs[0].index == 0
    assert r1.refs[0].key == "first"
    assert r1.refs[1].index == 1
    assert r1.refs[1].key == "second"
    assert r1.refs[2].index == 2
    assert r1.refs[2].key == "third"

    r1.refs.pop(1)
    eg = quick_eg(api)
    api.update_release(eg.editgroup_id, r1.ident, r1)
    api.accept_editgroup(eg.editgroup_id)
    r1 = api.get_release(r1.ident)

    assert r1.refs[0].index == 0
    assert r1.refs[0].key == "first"
    assert r1.refs[1].index == 1
    assert r1.refs[1].key == "third"

def test_citation_targets(api):
    # invariant to linking citations
    # also, updates work

    eg = quick_eg(api)
    r1 = ReleaseEntity(title="the target", ext_ids=ReleaseExtIds())
    r1 = api.get_release(api.create_release(eg.editgroup_id, r1).ident)
    r2 = ReleaseEntity(title="the citer", ext_ids=ReleaseExtIds())
    r2.refs = [
        ReleaseRef(key="first", title="something else"),
        ReleaseRef(key="second", title="the target title"),
    ]
    r2 = api.get_release(api.create_release(eg.editgroup_id, r2).ident)
    api.accept_editgroup(eg.editgroup_id)

    eg = quick_eg(api)
    r2.refs[1].target_release_id = r1.ident
    api.update_release(eg.editgroup_id, r2.ident, r2)
    api.accept_editgroup(eg.editgroup_id)
    r2 = api.get_release(r2.ident)
    assert r2.refs[0].key == "first"
    assert r2.refs[1].key == "second"
    assert r2.refs[0].index == 0 # TODO: one-indexing?
    assert r2.refs[1].index == 1
    assert r2.refs[0].target_release_id is None
    assert r2.refs[1].target_release_id == r1.ident
    assert len(r2.refs) == 2

def test_citation_empty_array(api):
    # distinction between empty array (no citations) and no array (hidden)

    r1 = ReleaseEntity(title="citation null", ext_ids=ReleaseExtIds())
    r2 = ReleaseEntity(title="citation empty array", ext_ids=ReleaseExtIds())
    r1.refs = None
    r2.refs = []

    eg = quick_eg(api)
    r1 = api.get_release(api.create_release(eg.editgroup_id, r1).ident)
    r2 = api.get_release(api.create_release(eg.editgroup_id, r2).ident)
    api.accept_editgroup(eg.editgroup_id)

    print(r1.refs)
    print(r2.refs)
    assert r1.refs == []
    assert r1.refs == r2.refs

    r1b = api.get_release(r1.ident, hide="refs")
    assert r1b.refs is None

def test_citation_encoding(api):
    # escape-only changes (eg, \u1234 whatever for ASCII)

    r1 = ReleaseEntity(title="citation encoding", ext_ids=ReleaseExtIds())
    title = "title-unicode \\u0050 \\\" "
    container = "container-unicode ☃︎ ä ö ü スティー"
    extra = extra={'a': 1, 'b': 2, 'ö': 3}
    locator = "p123"
    r1.refs = [
        ReleaseRef(key="1", year=1923, title=title, container_name=container,
            extra=extra, locator=locator),
        ReleaseRef(key="2"),
    ]

    eg = quick_eg(api)
    r1 = api.get_release(api.create_release(eg.editgroup_id, r1).ident)
    api.accept_editgroup(eg.editgroup_id)

    assert title == r1.refs[0].title
    assert container == r1.refs[0].container_name
    assert extra == r1.refs[0].extra
    assert locator == r1.refs[0].locator
