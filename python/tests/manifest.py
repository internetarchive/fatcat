
import json
import pytest
from fatcat.manifest_importer import FatcatManifestImporter


@pytest.fixture(scope="function")
def manifest_importer():
    yield FatcatManifestImporter("http://localhost:9411/v0")

# TODO: use API to check that entities actually created...
#def test_manifest_importer_batch(manifest_importer):
#    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
#        manifest_importer.process_batch(f)

#def test_manifest_importer(manifest_importer):
#    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
#        manifest_importer.process_source(f)

def test_manifest_row_parse(manifest_importer):
    # (sha1, mimetype, size_bytes, md5, doi, url, datetime) = row

    c = manifest_importer.parse_manifest_row(
        (None, None, None, None, None, None, None))
    assert c == None

    c = manifest_importer.parse_manifest_row(
        ("7d97e98f8af710c7e7fe703abc8f639e0ee507c4", "application/pdf", "12345", "8af710c7e7fe703abc8f639e0ee507c4", "10.1234/asdf", "https://example.com/thing.pdf", "200001010000"))
    assert c.sha1 == "7d97e98f8af710c7e7fe703abc8f639e0ee507c4"
    assert c.mimetype == "application/pdf"
    assert c.urls[0].url == "https://example.com/thing.pdf"
    assert c.urls[0].rel == "web"
    assert c.urls[1].url == "https://web.archive.org/web/200001010000/https://example.com/thing.pdf"
    assert c.urls[1].rel == "webarchive"
