
import json
import pytest
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
from fixtures import *


def test_fileset(api):

    eg = quick_eg(api)
    r1 = ReleaseEntity(title="test fileset release", ext_ids=ReleaseExtIds())
    r1edit = api.create_release(eg.editgroup_id, r1)

    fs1 = FilesetEntity(
        manifest = [
            FilesetFile(
                path="data/thing.tar.gz",
                size=54321,
                md5="540da3ea6e448d8dfb057c05225f853a",
                sha1="1dab6a0e110f9b5d70b18db0abf051f7f93faf06",
                sha256="c7b49f3e84cd1b7cb0b0e3e9f632b7be7e21b4dc229df23331f880a8a7dfa75a",
                extra={"a": 1, "b": 3},
            ),
            FilesetFile(
                path="README.md",
                size=54210,
                md5="5f83592b5249671719bbed6ce91ecfa8",
                sha1="455face3598611458efe1f072e58624790a67266",
                sha256="429bcafa4d3d0072d5b2511e12c85c1aac1d304011d1c406da14707f7b9cd905",
                extra={"x": 1, "y": "q"},
            ),
        ],
        urls = [
            FilesetUrl(url="https://archive.org/download/fileset-123/", rel="repository"),
            FilesetUrl(url="https://humble-host.com/~user123/dataset/", rel="web"),
        ],
        release_ids = [r1edit.ident],
        extra=dict(t=4, u=9),
        edit_extra=dict(test_key="filesets rule"),
    )

    fs1edit = api.create_fileset(eg.editgroup_id, fs1)
    assert fs1edit.extra == fs1.edit_extra
    api.accept_editgroup(eg.editgroup_id)
    fs2 = api.get_fileset(fs1edit.ident)

    # get revision
    fs2_rev = api.get_fileset_revision(fs1edit.revision)
    assert fs1edit.revision == fs2_rev.revision
    assert fs2.revision == fs2_rev.revision

    # check that fields match
    assert fs1.urls == fs2.urls
    assert fs1.manifest == fs2.manifest
    assert fs1.release_ids == fs2.release_ids
    assert fs1.extra == fs2.extra

    # expansion
    r1 = api.get_release(r1edit.ident, expand="filesets")
    assert r1.filesets[0].manifest == fs1.manifest

    # get redirects (none)
    assert api.get_fileset_redirects(fs2.ident) == []
    
    # delete
    eg = quick_eg(api)
    api.delete_fileset(eg.editgroup_id, fs2.ident)
    api.accept_editgroup(eg.editgroup_id)
    fs2 = api.get_fileset(fs2.ident)
    assert fs2.state == "deleted"

def test_fileset_examples(api):
    fs3 = api.get_fileset('aaaaaaaaaaaaaztgaaaaaaaaam')

    assert fs3.urls[0].url == 'http://other-personal-blog.name/dataset/'
    assert fs3.urls[1].rel == 'archive'
    assert fs3.manifest[1].md5 == 'f4de91152c7ab9fdc2a128f962faebff'
    assert fs3.manifest[1].extra['mimetype'] == 'application/gzip'

def test_bad_fileset(api):

    eg = quick_eg(api)

    bad_list = [
        # good (for testing test itself)
        #FilesetEntity(manifest=[FilesetFile(path="123.jpg", size=1234)]),
        #FilesetEntity(urls=[FilesetUrl(url="thing", rel="blah")]),
        FilesetEntity(manifest=[FilesetFile(path="123.jpg", size="big")]),
        FilesetEntity(release_ids=["asdf"]),
    ]

    for b in bad_list:
        with pytest.raises(fatcat_client.rest.ApiException):
            api.create_fileset(eg.editgroup_id, b)

