
import json

import pytest
from fixtures import *

from fatcat_tools.importers import FilesetImporter, JsonLinePusher


@pytest.fixture(scope="function")
def fileset_importer(api):
    yield FilesetImporter(api)

# TODO: use API to check that entities actually created...
def test_fileset_importer_basic(fileset_importer):
    with open('tests/files/fileset_ltjp7k2nrbes3or5h4na5qgxlu.json', 'r') as f:
        JsonLinePusher(fileset_importer, f).run()

def test_fileset_importer(fileset_importer):
    last_index = fileset_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/fileset_ltjp7k2nrbes3or5h4na5qgxlu.json', 'r') as f:
        fileset_importer.bezerk_mode = True
        counts = JsonLinePusher(fileset_importer, f).run()
    assert counts['insert'] == 1
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = fileset_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "generic fileset" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.FilesetImporter" in eg.extra['agent']

    # re-insert; should skip
    with open('tests/files/fileset_ltjp7k2nrbes3or5h4na5qgxlu.json', 'r') as f:
        fileset_importer.reset()
        fileset_importer.bezerk_mode = False
        counts = JsonLinePusher(fileset_importer, f).run()
    assert counts['insert'] == 0
    assert counts['exists'] == 1
    assert counts['skip'] == 0

def test_fileset_dict_parse(fileset_importer):
    with open('tests/files/fileset_ltjp7k2nrbes3or5h4na5qgxlu.json', 'r') as f:
        raw = json.loads(f.readline())
        fs = fileset_importer.parse_record(raw)

        assert fs.manifest[0].sha1 == "cc9bd558ca79b30b2966714da7ef4129537fde0c"
        assert fs.manifest[0].md5 == "742c40404c9a4dbbd77c0985201c639f"
        assert fs.manifest[0].sha256 == "3a7c07ad17ce3638d5a1dd21f995a496e430b952eef00270ad741d506984370f"
        assert fs.manifest[0].size == 640500
        assert fs.manifest[0].path == "070111_LatA_100nM.txt"
        assert fs.manifest[0].extra['mimetype'] == "text/plain"
        assert len(fs.urls) == 3
        for u in fs.urls:
            if u.rel == "repo":
                assert u.url == "https://merritt.cdlib.org/d/ark%3A%2Fb5068%2Fd1rp49/1/"
        assert len(fs.release_ids) == 1
