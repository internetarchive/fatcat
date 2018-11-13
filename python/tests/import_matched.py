
import json
import pytest
from fatcat_tools.importers.matched import FatcatMatchedImporter


@pytest.fixture(scope="function")
def matched_importer():
    yield FatcatMatchedImporter("http://localhost:9411/v0")

# TODO: use API to check that entities actually created...
def test_matched_importer_batch(matched_importer):
    with open('tests/files/example_matched.json', 'r') as f:
        matched_importer.process_batch(f)

def test_matched_importer(matched_importer):
    with open('tests/files/example_matched.json', 'r') as f:
        matched_importer.process_source(f)

def test_matched_dict_parse(matched_importer):
    with open('tests/files/example_matched.json', 'r') as f:
        raw = json.loads(f.readline())
        f = matched_importer.parse_matched_dict(raw)
        assert f.sha1 == "00242a192acc258bdfdb151943419437f440c313"
        assert f.md5 == "f4de91152c7ab9fdc2a128f962faebff"
        assert f.mimetype == "application/pdf"
        assert f.size == 255629
        assert f.urls[1].url.startswith("http://journals.plos.org")
        assert f.urls[1].rel == "web"
        assert f.urls[0].url.startswith("https://web.archive.org/")
        assert f.urls[0].rel == "webarchive"
        assert len(f.releases) == 1
