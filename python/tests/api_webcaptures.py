
import json
import pytest
import datetime
from copy import copy

from fatcat_client import *
from fatcat_client.rest import ApiException
from fixtures import *


def test_webcapture(api):

    eg = quick_eg(api)
    r1 = ReleaseEntity(title="test webcapture release", ext_ids=ReleaseExtIds())
    r1edit = api.create_release(eg.editgroup_id, r1)

    wc1 = WebcaptureEntity(
        original_url = "http://example.site",
        #timestamp = "2012-01-02T03:04:05Z",
        timestamp = datetime.datetime.now(datetime.timezone.utc),
        cdx = [
            WebcaptureCdxLine(
                surt="site,example,)/data/thing.tar.gz",
                #timestamp="2012-01-02T03:04:05Z",
                timestamp=datetime.datetime.now(datetime.timezone.utc),
                url="http://example.site/data/thing.tar.gz",
                mimetype="application/gzip",
                status_code=200,
                size=1234,
                sha1="455face3598611458efe1f072e58624790a67266",
                sha256="c7b49f3e84cd1b7cb0b0e3e9f632b7be7e21b4dc229df23331f880a8a7dfa75a",
            ),
            WebcaptureCdxLine(
                surt="site,example,)/README.md",
                #timestamp="2012-01-02T03:04:05Z",
                timestamp=datetime.datetime.now(datetime.timezone.utc),
                url="http://example.site/README.md",
                mimetype="text/markdown",
                status_code=200,
                size=4321,
                sha1="455face3598611458efe1f072e58624790a67266",
                sha256="429bcafa4d3d0072d5b2511e12c85c1aac1d304011d1c406da14707f7b9cd905",
            ),
        ],
        archive_urls = [
            FileUrl(rel="wayback", url="https://web.archive.org/web/"),
        ],
        release_ids = [r1edit.ident],
        extra=dict(c=1, b=2),
        edit_extra=dict(test_key="webcaptures rule"),
    )

    wc1edit = api.create_webcapture(eg.editgroup_id, wc1)
    assert wc1edit.extra == wc1.edit_extra
    api.accept_editgroup(eg.editgroup_id)
    wc2 = api.get_webcapture(wc1edit.ident)

    # get revision
    wc2_rev = api.get_webcapture_revision(wc1edit.revision)
    assert wc1edit.revision == wc2_rev.revision
    assert wc2.revision == wc2_rev.revision
    assert wc2.timestamp == wc2_rev.timestamp

    # check that fields match
    # I don't know why these aren't equal...
    #print(wc1.archive_urls)
    #print(wc2.archive_urls)
    #assert wc1.archive_urls == wc2.archive_urls
    assert wc1.archive_urls[0].rel == wc2.archive_urls[0].rel
    assert wc1.archive_urls[0].url == wc2.archive_urls[0].url
    assert wc1.cdx[0] == wc2.cdx[0]
    assert wc1.cdx[0].size == wc2.cdx[0].size
    assert wc1.cdx == wc2.cdx
    assert wc1.release_ids == wc2.release_ids
    assert wc1.timestamp == wc2.timestamp
    assert wc1.original_url == wc2.original_url
    assert wc1.extra == wc2.extra

    # TODO: check release expansion
    r1 = api.get_release(r1edit.ident, expand="webcaptures")
    print(r1)
    assert r1.webcaptures[0].cdx == wc1.cdx

    # get redirects (none)
    assert api.get_webcapture_redirects(wc2.ident) == []
    
    # delete
    eg = quick_eg(api)
    api.delete_webcapture(eg.editgroup_id, wc2.ident)
    api.accept_editgroup(eg.editgroup_id)
    wc2 = api.get_webcapture(wc2.ident)
    assert wc2.state == "deleted"

def test_webcapture_examples(api):
    wc3 = api.get_webcapture('aaaaaaaaaaaaa53xaaaaaaaaam')
    assert wc3.releases is None
    wc3 = api.get_webcapture('aaaaaaaaaaaaa53xaaaaaaaaam', expand="releases")

    assert wc3.cdx[0].surt == 'org,asheesh)/'
    assert wc3.cdx[1].sha1 == 'a637f1d27d9bcb237310ed29f19c07e1c8cf0aa5'
    assert wc3.archive_urls[1].rel == 'warc'
    assert wc3.releases[0].ident
    assert wc3.releases[0].abstracts is None
    assert wc3.releases[0].refs is None


def test_bad_webcapture(api):

    eg = quick_eg(api)

    # good (as a template)
    good = WebcaptureEntity(
        original_url="http://example.site/123.jpg",
        timestamp="2012-01-02T03:04:05Z",
        cdx=[WebcaptureCdxLine(
            surt="site,example,)/123.jpg",
            url="http://example.site/123.jpg",
            sha1="455face3598611458efe1f072e58624790a67266",
            timestamp="2012-01-02T03:04:05Z")])

    bad_list = [
        # uncomment to "test the test"
        #good,
        # CDX timestamp format
        WebcaptureEntity(
            original_url="http://example.site/123.jpg",
            timestamp="2012-01-02T03:04:05Z",
            cdx=[WebcaptureCdxLine(
                surt="site,example,)/123.jpg",
                url="http://example.site/123.jpg",
                sha1="455face3598611458efe1f072e58624790a67266",
                size=123,
                timestamp="20120102030405")]),
        # CDX timestamp format (int)
        WebcaptureEntity(
            original_url="http://example.site/123.jpg",
            timestamp="2012-01-02T03:04:05Z",
            cdx=[WebcaptureCdxLine(
                surt="site,example,)/123.jpg",
                url="http://example.site/123.jpg",
                sha1="455face3598611458efe1f072e58624790a67266",
                timestamp=20120102030405)]),
        # negative size
        WebcaptureEntity(
            original_url="http://example.site/123.jpg",
            timestamp="2012-01-02T03:04:05Z",
            cdx=[WebcaptureCdxLine(
                surt="site,example,)/123.jpg",
                url="http://example.site/123.jpg",
                sha1="455face3598611458efe1f072e58624790a67266",
                size=-123,
                timestamp="20120102030405")]),
    ]

    api.create_webcapture(eg.editgroup_id, good)
    for b in bad_list:
        with pytest.raises(fatcat_client.rest.ApiException):
            api.create_webcapture(eg.editgroup_id, b)

    with pytest.raises(ValueError):
        # missing/empty CDX url
        WebcaptureEntity(
            original_url="http://example.site/123.jpg",
            timestamp="2012-01-02T03:04:05Z",
            cdx=[WebcaptureCdxLine(
                #url="http://example.site/123.jpg",
                surt="site,example,)/123.jpg",
                sha1="455face3598611458efe1f072e58624790a67266",
                timestamp="2012-01-02T03:04:05Z",
            )])

    with pytest.raises(ValueError):
        # missing/empty CDX timestamp
        WebcaptureEntity(
            original_url="http://example.site/123.jpg",
            timestamp="2012-01-02T03:04:05Z",
            cdx=[WebcaptureCdxLine(
                url="http://example.site/123.jpg",
                surt="site,example,)/123.jpg",
                sha1="455face3598611458efe1f072e58624790a67266",
                #timestamp="2012-01-02T03:04:05Z",
            )])
