
import pytest
from fatcat_tools.importers import JournalMetadataImporter, CsvPusher
from fixtures import api


@pytest.fixture(scope="function")
def journal_metadata_importer(api):
    yield JournalMetadataImporter(api)

# TODO: use API to check that entities actually created...
def test_journal_metadata_importer_batch(journal_metadata_importer):
    with open('tests/files/journal_extra_metadata.snip.csv', 'r') as f:
        CsvPusher(journal_metadata_importer, f).run()

def test_journal_metadata_importer(journal_metadata_importer):
    with open('tests/files/journal_extra_metadata.snip.csv', 'r') as f:
        journal_metadata_importer.bezerk_mode = True
        journal_metadata_importer.serial_mode = True
        CsvPusher(journal_metadata_importer, f).run()

    # fetch most recent editgroup
    changes = journal_metadata_importer.api.get_changelog(limit=1)
    eg = changes[0].editgroup
    assert eg.description
    assert "container" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.JournalMetadataImporter" in eg.extra['agent']
