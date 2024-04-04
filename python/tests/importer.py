from fixtures import *

from fatcat_tools.importers import CrossrefImporter, OrcidImporter


def test_issnl_mapping_lookup(api):
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        fi = CrossrefImporter(api, issn_map_file=issn_file)

    assert fi.issn2issnl("0000-0027") == "0002-0027"
    assert fi.issn2issnl("0002-0027") == "0002-0027"
    assert fi.issn2issnl("9999-0027") is None

    assert fi.lookup_issnl("9999-9999") is None


def test_identifiers(api):

    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        ci = CrossrefImporter(api, issn_map_file=issn_file)

    assert ci.is_issnl("1234-5678") is True
    assert ci.is_issnl("1234-5678.") is False
    assert ci.is_issnl("12345678") is False
    assert ci.is_issnl("1-2345678") is False
