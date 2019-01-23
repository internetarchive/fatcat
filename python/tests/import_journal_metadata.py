
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
    last_index = journal_metadata_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/journal_extra_metadata.snip.csv', 'r') as f:
        journal_metadata_importer.bezerk_mode = True
        counts = CsvPusher(journal_metadata_importer, f).run()
    assert counts['insert'] == 9
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = journal_metadata_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "container" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.JournalMetadataImporter" in eg.extra['agent']

    with open('tests/files/journal_extra_metadata.snip.csv', 'r') as f:
        journal_metadata_importer.reset()
        journal_metadata_importer.bezerk_mode = False
        counts = CsvPusher(journal_metadata_importer, f).run()
    assert counts['insert'] == 0
    assert counts['exists'] == 9
    assert counts['skip'] == 0
