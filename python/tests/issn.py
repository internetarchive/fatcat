
import pytest
from fatcat_tools.issn_importer import FatcatIssnImporter


@pytest.fixture(scope="function")
def issn_importer():
    yield FatcatIssnImporter("http://localhost:9411/v0")

# TODO: use API to check that entities actually created...
def test_issn_importer_batch(issn_importer):
    with open('tests/files/journal_extra_metadata.snip.csv', 'r') as f:
        issn_importer.process_csv_batch(f)

def test_issn_importer(issn_importer):
    with open('tests/files/journal_extra_metadata.snip.csv', 'r') as f:
        issn_importer.process_csv_source(f)
