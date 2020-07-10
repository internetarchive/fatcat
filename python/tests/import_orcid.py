
import json
import pytest

from fatcat_tools.importers import OrcidImporter, JsonLinePusher
from fixtures import *


@pytest.fixture(scope="function")
def orcid_importer(api):
    yield OrcidImporter(api)

def test_orcid_importer_badid(orcid_importer):
    with open('tests/files/0000-0001-8254-710X.json', 'r') as f:
        JsonLinePusher(orcid_importer, f).run()

# TODO: use API to check that entities actually created...
def test_orcid_importer(orcid_importer):
    last_index = orcid_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
        orcid_importer.bezerk_mode = True
        counts = JsonLinePusher(orcid_importer, f).run()
    assert counts['insert'] == 1
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = orcid_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "orcid" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.OrcidImporter" in eg.extra['agent']

    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
        orcid_importer.reset()
        orcid_importer.bezerk_mode = False
        counts = JsonLinePusher(orcid_importer, f).run()
    assert counts['insert'] == 0
    assert counts['exists'] == 1
    assert counts['skip'] == 0

def test_orcid_importer_x(orcid_importer):
    with open('tests/files/0000-0003-3953-765X.json', 'r') as f:
        JsonLinePusher(orcid_importer, f).run()
    c = orcid_importer.api.lookup_creator(orcid="0000-0003-3953-765X")
    assert c is not None

def test_orcid_dict_parse(orcid_importer):
    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
        raw = json.loads(f.readline())
        c = orcid_importer.parse_record(raw)
        assert c.given_name == "Man-Hui"
        assert c.surname == "Li"
        assert c.display_name == "Man-Hui Li"
        assert c.orcid == "0000-0001-8254-7103"
