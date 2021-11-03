
import json

import pytest
from fixtures import *

from fatcat_tools.importers import (
    IngestFileResultImporter,
    IngestWebResultImporter,
    JsonLinePusher,
)


@pytest.fixture(scope="function")
def ingest_importer(api):
    yield IngestFileResultImporter(api)

@pytest.fixture(scope="function")
def ingest_web_importer(api):
    yield IngestWebResultImporter(api)

# TODO: use API to check that entities actually created...
def test_ingest_importer_basic(ingest_importer):
    with open('tests/files/example_ingest.json', 'r') as f:
        JsonLinePusher(ingest_importer, f).run()

def test_ingest_importer(ingest_importer):
    last_index = ingest_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_ingest.json', 'r') as f:
        ingest_importer.bezerk_mode = True
        counts = JsonLinePusher(ingest_importer, f).run()
    assert counts['insert'] == 1
    assert counts['exists'] == 0
    assert counts['skip'] == 1

    # fetch most recent editgroup
    change = ingest_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "crawled from web" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.IngestFileResultImporter" in eg.extra['agent']

    # re-insert; should skip
    with open('tests/files/example_ingest.json', 'r') as f:
        ingest_importer.reset()
        ingest_importer.bezerk_mode = False
        counts = JsonLinePusher(ingest_importer, f).run()
    assert counts['insert'] == 0
    assert counts['exists'] == 1
    assert counts['skip'] == 1

def test_ingest_importer_xml(ingest_importer):
    last_index = ingest_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_ingest_xml.json', 'r') as f:
        ingest_importer.bezerk_mode = True
        counts = JsonLinePusher(ingest_importer, f).run()
    print(counts)
    assert counts['insert'] == 1
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = ingest_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "crawled from web" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.IngestFileResultImporter" in eg.extra['agent']

    # re-import should skip
    with open('tests/files/example_ingest_xml.json', 'r') as f:
        ingest_importer.reset()
        ingest_importer.bezerk_mode = False
        counts = JsonLinePusher(ingest_importer, f).run()
    assert counts['insert'] == 0
    assert counts['exists'] == 1
    assert counts['skip'] == 0

def test_ingest_importer_web(ingest_web_importer):
    last_index = ingest_web_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_ingest_html.json', 'r') as f:
        ingest_web_importer.bezerk_mode = True
        counts = JsonLinePusher(ingest_web_importer, f).run()
    print(counts)
    assert counts['insert'] == 1
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = ingest_web_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "crawled from web" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.IngestWebResultImporter" in eg.extra['agent']

    # re-import should skip
    with open('tests/files/example_ingest_html.json', 'r') as f:
        ingest_web_importer.reset()
        ingest_web_importer.bezerk_mode = False
        counts = JsonLinePusher(ingest_web_importer, f).run()
    assert counts['insert'] == 0
    assert counts['exists'] == 1
    assert counts['skip'] == 0

def test_ingest_importer_stage(ingest_importer, api):
    """
    Tests that ingest importer correctly handles release stage matching
    """
    test_table = [
        dict(request_stage=None, release_stage=None, status="insert"),
        dict(request_stage="published", release_stage=None, status="insert"),
        dict(request_stage=None, release_stage="draft", status="insert"),
        dict(request_stage="published", release_stage="published", status="insert"),
        dict(request_stage="draft", release_stage="published", status="skip-release-stage"),
        dict(request_stage="published", release_stage="draft", status="skip-release-stage"),
    ]
    ingest_importer.bezerk_mode = True
    with open('tests/files/example_ingest.json', 'r') as f:
        raw = json.loads(f.readline())
    for row in test_table:
        #print(row)

        # set dummy record stage
        eg = quick_eg(api)
        r1 = api.lookup_release(doi="10.123/abc")
        r1.release_stage = row['release_stage']
        api.update_release(eg.editgroup_id, r1.ident, r1)
        api.accept_editgroup(eg.editgroup_id)

        # set ingest request stage
        raw['request']['release_stage'] = row['request_stage']
        ingest_importer.reset()
        ingest_importer.push_record(raw)
        counts = ingest_importer.finish()
        print(counts)
        assert counts["total"] == 1
        assert counts[row['status']] == 1

def test_ingest_dict_parse(ingest_importer):
    with open('tests/files/example_ingest.json', 'r') as f:
        raw = json.loads(f.readline())
        f = ingest_importer.parse_record(raw)
        assert f.sha1 == "00242a192acc258bdfdb151943419437f440c313"
        assert f.md5 == "f4de91152c7ab9fdc2a128f962faebff"
        assert f.mimetype == "application/pdf"
        assert f.size == 255629
        assert len(f.urls) == 2
        for u in f.urls:
            if u.rel == "web":
                assert u.url.startswith("http://journals.plos.org")
            if u.rel == "webarchive":
                assert u.url.startswith("https://web.archive.org/")
        assert len(f.release_ids) == 1

def test_ingest_dict_parse_old(ingest_importer):
    with open('tests/files/example_ingest.old.json', 'r') as f:
        raw = json.loads(f.readline())

        # ancient ingest requests had no type; skip them
        f = ingest_importer.parse_record(raw)
        assert f is None
        raw['request']['ingest_type'] = 'pdf'

        f = ingest_importer.parse_record(raw)
        assert f.sha1 == "00242a192acc258bdfdb151943419437f440c313"
        assert f.md5 == "f4de91152c7ab9fdc2a128f962faebff"
        assert f.mimetype == "application/pdf"
        assert f.size == 255629
        assert len(f.urls) == 2
        for u in f.urls:
            if u.rel == "web":
                assert u.url.startswith("http://journals.plos.org")
            if u.rel == "webarchive":
                assert u.url.startswith("https://web.archive.org/")
        assert len(f.release_ids) == 1
