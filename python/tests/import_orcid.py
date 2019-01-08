
import json
import pytest
from fatcat_tools.importers import OrcidImporter
from fixtures import api


@pytest.fixture(scope="function")
def orcid_importer(api):
    yield OrcidImporter(api)

# TODO: use API to check that entities actually created...
def test_orcid_importer_batch(orcid_importer):
    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
        orcid_importer.process_batch(f)

def test_orcid_importer_badid(orcid_importer):
    with open('tests/files/0000-0001-8254-710X.json', 'r') as f:
        orcid_importer.process_batch(f)

def test_orcid_importer(orcid_importer):
    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
        orcid_importer.process_source(f)

    # fetch most recent editgroup
    changes = orcid_importer.api.get_changelog(limit=1)
    eg = changes[0].editgroup
    assert eg.description
    assert "orcid" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.OrcidImporter" in eg.extra['agent']

def test_orcid_importer_x(orcid_importer):
    with open('tests/files/0000-0003-3953-765X.json', 'r') as f:
        orcid_importer.process_source(f)
    c = orcid_importer.api.lookup_creator(orcid="0000-0003-3953-765X")
    assert c is not None

def test_orcid_dict_parse(orcid_importer):
    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
        raw = json.loads(f.readline())
        c = orcid_importer.parse_orcid_dict(raw)
        assert c.given_name == "Man-Hui"
        assert c.surname == "Li"
        assert c.display_name == "Man-Hui Li"
        assert c.orcid == "0000-0001-8254-7103"
