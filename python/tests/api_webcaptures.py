
import json
import pytest
import datetime
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
from fixtures import *


def test_webcapture(api):

    eg = quick_eg(api)
    r1 = ReleaseEntity(title="test webcapture release")
    r1edit = api.create_release(r1, editgroup_id=eg.editgroup_id)

    wc1 = WebcaptureEntity(
        original_url = "http://example.site",
        #timestamp = "2012-01-02T03:04:05Z",
        timestamp = datetime.datetime.now(datetime.timezone.utc),
    )
    wc1.cdx = [
        WebcaptureEntityCdx(
            surt="site,example,)/data/thing.tar.gz",
            #timestamp="2012-01-02T03:04:05Z",
            timestamp=datetime.datetime.now(datetime.timezone.utc),
            url="http://example.site/data/thing.tar.gz",
            mimetype="application/gzip",
            status_code=200,
            sha1="455face3598611458efe1f072e58624790a67266",
            sha256="c7b49f3e84cd1b7cb0b0e3e9f632b7be7e21b4dc229df23331f880a8a7dfa75a",
        ),
        WebcaptureEntityCdx(
            surt="site,example,)/README.md",
            #timestamp="2012-01-02T03:04:05Z",
            timestamp=datetime.datetime.now(datetime.timezone.utc),
            url="http://example.site/README.md",
            mimetype="text/markdown",
            status_code=200,
            sha1="455face3598611458efe1f072e58624790a67266",
            sha256="429bcafa4d3d0072d5b2511e12c85c1aac1d304011d1c406da14707f7b9cd905",
        ),
    ]
    wc1.archive_urls = [
        FileEntityUrls(rel="wayback", url="https://web.archive.org/web/"),
    ]
    wc1.release_ids = [r1edit.ident]

    wc1edit = api.create_webcapture(wc1, editgroup_id=eg.editgroup_id)
    api.accept_editgroup(eg.editgroup_id)
    wc2 = api.get_webcapture(wc1edit.ident)

    # check that fields match
    # I don't know why these aren't equal...
    #print(wc1.archive_urls)
    #print(wc2.archive_urls)
    #assert wc1.archive_urls == wc2.archive_urls
    assert wc1.archive_urls[0].rel == wc2.archive_urls[0].rel
    assert wc1.archive_urls[0].url == wc2.archive_urls[0].url
    assert wc1.cdx == wc2.cdx
    assert wc1.release_ids == wc2.release_ids
    assert wc1.timestamp == wc2.timestamp
    assert wc1.original_url == wc2.original_url

    # TODO: check release expansion
    r1 = api.get_release(r1edit.ident, expand="webcaptures")
    print(r1)
    assert r1.webcaptures[0].cdx == wc1.cdx


def test_bad_webcapture(api):

    eg = quick_eg(api)

    bad_list = [
        # good (for testing test itself)
        WebcaptureEntity(cdx=[
            WebcaptureEntityCdx(
                surt="site,example,)/123.jpg",
                url="http://example.site/123.jpg",
                sha1="455face3598611458efe1f072e58624790a67266",
                timestamp=201506071122)]),
    ]

    for b in bad_list:
        with pytest.raises(fatcat_client.rest.ApiException):
            api.create_webcapture(b, editgroup_id=eg.editgroup_id)

