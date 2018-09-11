

import pytest
from fatcat.importer_common import FatcatImporter


def test_issnl_mapping_lookup():
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        fi = FatcatImporter("http://localhost:9411/v0", issn_file)

    assert fi.issn2issnl('0000-0027') == '0002-0027'
    assert fi.issn2issnl('0002-0027') == '0002-0027'
    assert fi.issn2issnl('9999-0027') == None

    assert fi.lookup_issnl('9999-9999') == None

def test_identifiers():

    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        fi = FatcatImporter("http://localhost:9411/v0", issn_file)

    assert fi.is_issnl("1234-5678") == True
    assert fi.is_issnl("1234-5678.") == False
    assert fi.is_issnl("12345678") == False
    assert fi.is_issnl("1-2345678") == False

    assert fi.is_doi("10.1234/56789") == True
    assert fi.is_doi("101234/56789") == False
    assert fi.is_doi("10.1234_56789") == False

    assert fi.is_orcid("0000-0003-3118-6591") == True
    assert fi.is_orcid("0000-0003-3953-765X") == True
    assert fi.is_orcid("0000-00x3-3118-659") == False
    assert fi.is_orcid("0000-00033118-659") == False
    assert fi.is_orcid("0000-0003-3118-659.") == False

