
import pytest
from fatcat_tools.importers import IssnImporter
from fixtures import api


@pytest.fixture(scope="function")
def issn_importer(api):
    yield IssnImporter(api)

# TODO: use API to check that entities actually created...
def test_issn_importer_batch(issn_importer):
    with open('tests/files/journal_extra_metadata.snip.csv', 'r') as f:
        issn_importer.process_csv_batch(f)

def test_issn_importer(issn_importer):
    with open('tests/files/journal_extra_metadata.snip.csv', 'r') as f:
        issn_importer.process_csv_source(f)

    # fetch most recent editgroup
    changes = issn_importer.api.get_changelog(limit=1)
    eg = changes[0].editgroup
    assert eg.description
    assert "container" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.IssnImporter" in eg.extra['agent']
