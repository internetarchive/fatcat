
import pytest
from bs4 import BeautifulSoup

from fatcat_tools.importers import DblpReleaseImporter, Bs4XmlLargeFilePusher
from fixtures import *


@pytest.fixture(scope="function")
def dblp_importer(api):
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield DblpReleaseImporter(api, issn_file, bezerk_mode=True, lookup_refs=True)

@pytest.fixture(scope="function")
def dblp_importer_existing(api):
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield DblpReleaseImporter(api, issn_file, bezerk_mode=False, lookup_refs=True)

def test_dblp_importer(dblp_importer):
    last_index = dblp_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_dblp.xml', 'rb') as f:
        dblp_importer.bezerk_mode = True
        counts = Bs4XmlLargeFilePusher(dblp_importer, f, dblp_importer.ELEMENT_TYPES, use_lxml=True).run()
    print(counts)
    assert counts['insert'] == 3
    assert counts['exists'] == 0
    assert counts['skip'] == 1

    # fetch most recent editgroup
    change = dblp_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "dblp" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.DblpReleaseImporter" in eg.extra['agent']

    # check that entity name mangling was fixed on import
    eg = dblp_importer.api.get_editgroup(eg.editgroup_id)
    release = dblp_importer.api.get_release(eg.edits.releases[0].ident)
    assert release.contribs[1].raw_name == "Michael H. Böhlen"

    last_index = dblp_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_dblp.xml', 'rb') as f:
        dblp_importer.bezerk_mode = False
        dblp_importer.reset()
        counts = Bs4XmlLargeFilePusher(dblp_importer, f, dblp_importer.ELEMENT_TYPES, use_lxml=True).run()
    print(counts)
    assert counts['insert'] == 0
    assert counts['exists'] == 3
    assert counts['skip'] == 1
    assert last_index == dblp_importer.api.get_changelog(limit=1)[0].index

def test_dblp_xml_parse(dblp_importer):
    with open('tests/files/example_dblp_article.xml', 'r') as f:
        soup = BeautifulSoup(f, "xml")
        r1 = dblp_importer.parse_record(soup.find_all("article")[0])

    assert r1.title == "Jim Gray, astronomer"
    assert r1.contribs[0].raw_name == "Alexander S. Szalay"
    # tested above, in LXML import path
    #assert r1.contribs[1].raw_name == "Michael H. Bohlen"
    assert r1.contribs[2].raw_name == "Nicolas Heist"
    # XXX: assert r1.contribs[2].extra['orcid'] == "0000-0002-4354-9138"
    assert r1.contribs[3].raw_name == "Jens Lehmann"
    assert r1.ext_ids.dblp == "journals/cacm/Szalay08"
    assert r1.ext_ids.doi == "10.1145/1400214.1400231"
    assert r1.pages == "58-65"
    assert r1.issue == "11"
    assert r1.volume == "51"
    assert r1.release_year == 2008
    assert r1.extra['container_name'] == "Commun. ACM"
    assert r1.extra['dblp']['type'] == "article"
