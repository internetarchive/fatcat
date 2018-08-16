
import json
import pytest
from fatcat.orcid_importer import FatcatOrcidImporter


@pytest.fixture(scope="function")
def orcid_importer():
    yield FatcatOrcidImporter("http://localhost:9411/v0")

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

def test_orcid_dict_parse(orcid_importer):
    with open('tests/files/0000-0001-8254-7103.json', 'r') as f:
        raw = json.loads(f.readline())
        c = orcid_importer.parse_orcid_dict(raw)
        assert c.given_name == "Man-Hui"
        assert c.surname == "Li"
        assert c.display_name == "Man-Hui Li"
        assert c.orcid == "0000-0001-8254-7103"
