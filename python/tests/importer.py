

import pytest
from fatcat.importer_common import FatcatImporter


def test_issnl_mapping_lookup():
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        fi = FatcatImporter("http://localhost:9411/v0", issn_file)

    assert fi.issn2issnl('0000-0027') == '0002-0027'
    assert fi.issn2issnl('0002-0027') == '0002-0027'
    assert fi.issn2issnl('9999-0027') == None

    assert fi.lookup_issnl('9999-9999') == None
