import json

import pytest
from fixtures import *

from fatcat_tools.importers import JsonLinePusher, MatchedImporter


@pytest.fixture(scope="function")
def matched_importer(api):
    yield MatchedImporter(api)


# TODO: use API to check that entities actually created...
def test_matched_importer_basic(matched_importer):
    with open("tests/files/example_matched.json") as f:
        JsonLinePusher(matched_importer, f).run()


def test_matched_importer(matched_importer):
    last_index = matched_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/example_matched.json") as f:
        matched_importer.bezerk_mode = True
        counts = JsonLinePusher(matched_importer, f).run()
    assert counts["insert"] == 2
    assert counts["exists"] == 0
    assert counts["skip"] == 11

    # fetch most recent editgroup
    change = matched_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "file-to-release" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.MatchedImporter" in eg.extra["agent"]

    # re-insert; should skip
    with open("tests/files/example_matched.json") as f:
        matched_importer.reset()
        matched_importer.bezerk_mode = False
        counts = JsonLinePusher(matched_importer, f).run()
    assert counts["insert"] == 0
    assert counts["exists"] == 2
    assert counts["skip"] == 11


def test_matched_dict_parse(matched_importer):
    with open("tests/files/example_matched.json") as f:
        raw = json.loads(f.readline())
        f = matched_importer.parse_record(raw)
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
