
import pytest
from fatcat.orcid_importer import FatcatOrcidImporter

@pytest.fixture(scope="function")
def orcid_importer():
    yield FatcatOrcidImporter("http://localhost:9411/v0")

def test_orcid_importer_batch(orcid_importer):
    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
        orcid_importer.process_batch(f)

def test_orcid_importer(orcid_importer):
    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
        orcid_importer.process_source(f)
