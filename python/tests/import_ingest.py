
import json
import pytest
from fatcat_tools.importers import IngestFileResultImporter, JsonLinePusher
from fixtures import api


@pytest.fixture(scope="function")
def ingest_importer(api):
    yield IngestFileResultImporter(api)

# TODO: use API to check that entities actually created...
def test_ingest_importer_basic(ingest_importer):
    with open('tests/files/example_ingest.json', 'r') as f:
        JsonLinePusher(ingest_importer, f).run()

@pytest.mark.skip("tests not flushed out yet")
def test_ingest_importer(ingest_importer):
    last_index = ingest_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_ingest.json', 'r') as f:
        ingest_importer.bezerk_mode = True
        counts = JsonLinePusher(ingest_importer, f).run()
    assert counts['insert'] == 2
    assert counts['exists'] == 0
    assert counts['skip'] == 11

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
    assert counts['exists'] == 2
    assert counts['skip'] == 11

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
