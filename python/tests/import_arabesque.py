import json

import pytest
from fixtures import *

from fatcat_tools.importers import ArabesqueMatchImporter, JsonLinePusher, SqlitePusher


@pytest.fixture(scope="function")
def arabesque_importer(api):
    yield ArabesqueMatchImporter(api, extid_type="doi", crawl_id="DUMMY123")


# TODO: use API to check that entities actually created...
def test_arabesque_importer_basic(arabesque_importer):
    SqlitePusher(
        arabesque_importer, "tests/files/arabesque_example.sqlite3", "crawl_result"
    ).run()


def test_arabesque_importer_json(arabesque_importer):
    with open("tests/files/arabesque_example.json", "r") as f:
        JsonLinePusher(arabesque_importer, f).run()


def test_arabesque_importer(arabesque_importer):
    last_index = arabesque_importer.api.get_changelog(limit=1)[0].index
    arabesque_importer.bezerk_mode = True
    counts = SqlitePusher(
        arabesque_importer, "tests/files/arabesque_example.sqlite3", "crawl_result"
    ).run()
    assert counts["insert"] == 1
    assert counts["exists"] == 0
    assert counts["skip"] == 490

    # fetch most recent editgroup
    change = arabesque_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "identifier/url seedlist" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert eg.extra["crawl_id"] == "DUMMY123"
    assert "fatcat_tools.ArabesqueMatchImporter" in eg.extra["agent"]

    # re-insert; should skip
    arabesque_importer.reset()
    arabesque_importer.bezerk_mode = False
    counts = SqlitePusher(
        arabesque_importer, "tests/files/arabesque_example.sqlite3", "crawl_result"
    ).run()
    assert counts["insert"] == 0
    assert counts["exists"] == 1
    assert counts["skip"] == 490


def test_arabesque_dict_parse(arabesque_importer):
    with open("tests/files/arabesque_example.json", "r") as f:
        raw = json.loads(f.readline())
        f = arabesque_importer.parse_record(raw)
        assert f.sha1 == "bdd78be55800bb1c9a5e47005bac5e4124793c7b"
        assert f.mimetype == "application/pdf"
        assert len(f.urls) == 2
        for u in f.urls:
            if u.rel == "web":
                assert u.url.startswith("https://files.eccomasproceedia.org/")
            if u.rel == "webarchive":
                assert u.url.startswith("https://web.archive.org/")
        assert len(f.release_ids) == 1
