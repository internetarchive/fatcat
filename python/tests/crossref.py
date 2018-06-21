
import pytest
from fatcat.crossref_importer import FatcatCrossrefImporter


@pytest.fixture(scope="function")
def crossref_importer():
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield FatcatCrossrefImporter("http://localhost:9411/v0", issn_file)

def test_crossref_importer_batch(crossref_importer):
    with open('tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        crossref_importer.process_batch(f)

def test_crossref_importer(crossref_importer):
    with open('tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        crossref_importer.process_source(f)

def test_crossref_importer_create(crossref_importer):
    crossref_importer.create_containers = True
    with open('tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        crossref_importer.process_source(f)
