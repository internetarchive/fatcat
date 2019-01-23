

import pytest
from fatcat_tools.importers import CrossrefImporter, OrcidImporter
from fixtures import api


def test_issnl_mapping_lookup(api):
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        fi = CrossrefImporter(api, issn_map_file=issn_file)

    assert fi.issn2issnl('0000-0027') == '0002-0027'
    assert fi.issn2issnl('0002-0027') == '0002-0027'
    assert fi.issn2issnl('9999-0027') == None

    assert fi.lookup_issnl('9999-9999') == None

def test_identifiers(api):

    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        ci = CrossrefImporter(api, issn_map_file=issn_file)

    assert ci.is_issnl("1234-5678") == True
    assert ci.is_issnl("1234-5678.") == False
    assert ci.is_issnl("12345678") == False
    assert ci.is_issnl("1-2345678") == False

    oi = OrcidImporter(api)

    assert oi.is_orcid("0000-0003-3118-6591") == True
    assert oi.is_orcid("0000-0003-3953-765X") == True
    assert oi.is_orcid("0000-00x3-3118-659") == False
    assert oi.is_orcid("0000-00033118-659") == False
    assert oi.is_orcid("0000-0003-3118-659.") == False

