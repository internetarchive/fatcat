
from fatcat_openapi_client import *
from fixtures import *


def test_annotations(api):

    eg = quick_eg(api)

    # ensure no annotations on this object
    a = api.get_editgroup_annotations(eg.editgroup_id)
    assert a == []

    # create an annotation!
    api.create_editgroup_annotation(
        eg.editgroup_id,
        EditgroupAnnotation(
            comment_markdown="some *annotation*",
            extra=dict(thing="thang")))

    # check that we can fetch it all sorts of ways
    a = api.get_editgroup_annotations(eg.editgroup_id)
    assert len(a) == 1
    assert a[0].extra['thing'] == "thang"

    # the editor persists, so this is a hack to find a "recent" one
    a2 = api.get_editor_annotations(eg.editor_id, limit=100)
    found = None
    for thing in a2:
        if thing.annotation_id == a[0].annotation_id:
            found = thing
            break
    assert thing
    assert thing.extra['thing'] == "thang"
