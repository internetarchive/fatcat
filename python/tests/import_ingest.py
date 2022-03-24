import json

import pytest
from fixtures import *

from fatcat_tools.importers import (
    IngestFileResultImporter,
    IngestFilesetFileResultImporter,
    IngestFilesetResultImporter,
    IngestWebResultImporter,
    JsonLinePusher,
)


@pytest.fixture(scope="function")
def ingest_importer(api):
    yield IngestFileResultImporter(api)


@pytest.fixture(scope="function")
def ingest_web_importer(api):
    yield IngestWebResultImporter(api)


@pytest.fixture(scope="function")
def ingest_fileset_importer(api):
    yield IngestFilesetResultImporter(api)


@pytest.fixture(scope="function")
def ingest_fileset_file_importer(api):
    yield IngestFilesetFileResultImporter(api)


# TODO: use API to check that entities actually created...
def test_ingest_importer_basic(ingest_importer):
    with open("tests/files/example_ingest.json", "r") as f:
        JsonLinePusher(ingest_importer, f).run()


def test_ingest_importer(ingest_importer):
    last_index = ingest_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/example_ingest.json", "r") as f:
        ingest_importer.bezerk_mode = True
        counts = JsonLinePusher(ingest_importer, f).run()
    assert counts["insert"] == 1
    assert counts["exists"] == 0
    assert counts["skip"] == 1

    # fetch most recent editgroup
    change = ingest_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "crawled from web" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.IngestFileResultImporter" in eg.extra["agent"]

    # re-insert; should skip
    with open("tests/files/example_ingest.json", "r") as f:
        ingest_importer.reset()
        ingest_importer.bezerk_mode = False
        counts = JsonLinePusher(ingest_importer, f).run()
    assert counts["insert"] == 0
    assert counts["exists"] == 1
    assert counts["skip"] == 1


def test_ingest_importer_xml(ingest_importer):
    last_index = ingest_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/example_ingest_xml.json", "r") as f:
        ingest_importer.bezerk_mode = True
        counts = JsonLinePusher(ingest_importer, f).run()
    assert counts["insert"] == 1
    assert counts["exists"] == 0
    assert counts["skip"] == 0

    # fetch most recent editgroup
    change = ingest_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "crawled from web" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.IngestFileResultImporter" in eg.extra["agent"]

    # re-import should skip
    with open("tests/files/example_ingest_xml.json", "r") as f:
        ingest_importer.reset()
        ingest_importer.bezerk_mode = False
        counts = JsonLinePusher(ingest_importer, f).run()
    assert counts["insert"] == 0
    assert counts["exists"] == 1
    assert counts["skip"] == 0


def test_ingest_importer_web(ingest_web_importer):
    last_index = ingest_web_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/example_ingest_html.json", "r") as f:
        ingest_web_importer.bezerk_mode = True
        counts = JsonLinePusher(ingest_web_importer, f).run()
    assert counts["insert"] == 1
    assert counts["exists"] == 0
    assert counts["skip"] == 0

    # fetch most recent editgroup
    change = ingest_web_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "crawled from web" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.IngestWebResultImporter" in eg.extra["agent"]

    # re-import should skip
    with open("tests/files/example_ingest_html.json", "r") as f:
        ingest_web_importer.reset()
        ingest_web_importer.bezerk_mode = False
        counts = JsonLinePusher(ingest_web_importer, f).run()
    assert counts["insert"] == 0
    assert counts["exists"] == 1
    assert counts["skip"] == 0


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
    with open("tests/files/example_ingest.json", "r") as f:
        raw = json.loads(f.readline())
    for row in test_table:
        # print(row)

        # set dummy record stage
        eg = quick_eg(api)
        r1 = api.lookup_release(doi="10.123/abc")
        r1.release_stage = row["release_stage"]
        api.update_release(eg.editgroup_id, r1.ident, r1)
        api.accept_editgroup(eg.editgroup_id)

        # set ingest request stage
        raw["request"]["release_stage"] = row["request_stage"]
        ingest_importer.reset()
        ingest_importer.push_record(raw)
        counts = ingest_importer.finish()
        assert counts["total"] == 1
        assert counts[row["status"]] == 1


def test_ingest_dict_parse(ingest_importer):
    with open("tests/files/example_ingest.json", "r") as f:
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
    with open("tests/files/example_ingest.old.json", "r") as f:
        raw = json.loads(f.readline())

        # ancient ingest requests had no type; skip them
        f = ingest_importer.parse_record(raw)
        assert f is None
        raw["request"]["ingest_type"] = "pdf"

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


def test_ingest_fileset_dict_parse(ingest_fileset_importer):
    with open("tests/files/example_fileset_ingest_result.json", "r") as f:
        raw = json.loads(f.readline())
        fs = ingest_fileset_importer.parse_record(raw)
        assert len(fs.manifest) == 3
        assert fs.manifest[0].sha1 == "c0669e84e7b9052cc0f342e8ce7d31d59956326a"
        assert fs.manifest[0].md5 == "caf4d9fc2c6ebd0d9251ac84e0b6b006"
        assert fs.manifest[0].extra["mimetype"] == "application/x-hdf"
        assert fs.manifest[0].size == 16799750
        assert fs.manifest[0].path == "N2 on food R_2010_03_25__10_53_27___4___1_features.hdf5"
        assert (
            fs.manifest[0].extra["original_url"]
            == "https://zenodo.org/api/files/563203f6-6de5-46d9-b305-ba42604f2508/N2%20on%20food%20R_2010_03_25__10_53_27___4___1_features.hdf5"
        )
        assert len(fs.urls) == 2
        for u in fs.urls:
            if u.rel == "web":
                assert u.url == "https://zenodo.org/record/1028059"
            if u.rel == "archive-base":
                assert u.url == "https://archive.org/download/zenodo.org-1028059/"
        assert len(fs.release_ids) == 1


def test_ingest_fileset_importer(ingest_fileset_importer):
    last_index = ingest_fileset_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/example_fileset_ingest_result.json", "r") as f:
        ingest_fileset_importer.bezerk_mode = True
        counts = JsonLinePusher(ingest_fileset_importer, f).run()
    assert counts["insert"] == 7
    assert counts["exists"] == 0
    assert counts["skip"] == 13
    assert counts["skip-release-not-found"] == 13

    # fetch most recent editgroup
    change = ingest_fileset_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "filesets crawled from web" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.IngestFilesetResultImporter" in eg.extra["agent"]

    # re-insert; should skip
    with open("tests/files/example_fileset_ingest_result.json", "r") as f:
        ingest_fileset_importer.reset()
        ingest_fileset_importer.bezerk_mode = False
        counts = JsonLinePusher(ingest_fileset_importer, f).run()

    assert counts["insert"] == 0
    assert counts["exists"] == 7
    assert counts["skip"] == 13
    assert counts["skip-release-not-found"] == 13


def test_ingest_fileset_file_dict_parse(ingest_fileset_file_importer):
    with open("tests/files/example_fileset_file_ingest_result.json", "r") as f:
        raw = json.loads(f.readline())
        fe = ingest_fileset_file_importer.parse_record(raw)
        assert fe.sha1 == "6fb020064da66bb7a666c17555611cf6820fc9ae"
        assert fe.md5 == "dfc41b617564f99a12e6077a6208876f"
        assert fe.sha256 == "2febad53ff0f163a18d7cbb913275bf99ed2544730cda191458837e2b0da9d18"
        assert fe.mimetype == "image/tiff"
        assert fe.size == 410631015
        assert fe.extra["path"] == "NDVI_Diff_1990_2018_T06.tif"
        assert len(fe.urls) == 2
        for u in fe.urls:
            if u.rel == "repository":
                assert u.url == "https://ndownloader.figshare.com/files/14460875"
            if u.rel == "archive":
                assert (
                    u.url
                    == "https://archive.org/download/springernature.figshare.com-7767695-v1/NDVI_Diff_1990_2018_T06.tif"
                )
        assert len(fe.release_ids) == 1


def test_ingest_fileset_file_importer(ingest_fileset_file_importer):
    """
    Similar to the above, but specifically tests 'file'/'success-file' import pathway
    """
    last_index = ingest_fileset_file_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/example_fileset_file_ingest_result.json", "r") as f:
        ingest_fileset_file_importer.bezerk_mode = True
        counts = JsonLinePusher(ingest_fileset_file_importer, f).run()
    assert counts["insert"] == 16
    assert counts["exists"] == 0
    assert counts["skip"] == 4
    assert counts["skip-bad-hashes"] == 4

    # fetch most recent editgroup
    change = ingest_fileset_file_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "crawled from web" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.IngestFilesetFileResultImporter" in eg.extra["agent"]

    # re-insert; should skip
    with open("tests/files/example_fileset_file_ingest_result.json", "r") as f:
        ingest_fileset_file_importer.reset()
        ingest_fileset_file_importer.bezerk_mode = False
        counts = JsonLinePusher(ingest_fileset_file_importer, f).run()
    assert counts["insert"] == 0
    assert counts["exists"] == 16
    assert counts["skip"] == 4
    assert counts["skip-bad-hashes"] == 4
