
import io
import pytest
from bs4 import BeautifulSoup

from fatcat_tools.importers import DblpReleaseImporter, DblpContainerImporter, Bs4XmlLargeFilePusher, JsonLinePusher
from fixtures import *


@pytest.fixture(scope="function")
def dblp_importer(api):
    with open('tests/files/dblp_container_map.tsv', 'r') as tsv_file:
        yield DblpReleaseImporter(api, tsv_file, bezerk_mode=True)

@pytest.fixture(scope="function")
def dblp_importer_existing(api):
    with open('tests/files/dblp_container_map.tsv', 'r') as tsv_file:
        yield DblpReleaseImporter(api, tsv_file, bezerk_mode=False)

@pytest.fixture(scope="function")
def dblp_container_importer(api):
    with open('tests/files/dblp_container_map.tsv', 'r') as tsv_file:
        with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
            yield DblpContainerImporter(api, issn_file, tsv_file, io.StringIO(), bezerk_mode=True)

def test_dblp_importer(dblp_importer):
    last_index = dblp_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_dblp.xml', 'rb') as f:
        dblp_importer.bezerk_mode = True
        counts = Bs4XmlLargeFilePusher(dblp_importer, f, dblp_importer.ELEMENT_TYPES, use_lxml=True).run()
    #print(counts)
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
    for r_edit in eg.edits.releases:
        release = dblp_importer.api.get_release(r_edit.ident)
        #print(release.ext_ids.dblp)
        if release.ext_ids.dblp == "conf/er/Norrie08":
            break
    assert release.ext_ids.dblp == "conf/er/Norrie08"
    assert release.contribs[0].raw_name == "Moira C. Norrie"
    assert release.contribs[1].raw_name == "Michael H. Böhlen"

    last_index = dblp_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_dblp.xml', 'rb') as f:
        dblp_importer.bezerk_mode = False
        dblp_importer.reset()
        counts = Bs4XmlLargeFilePusher(dblp_importer, f, dblp_importer.ELEMENT_TYPES, use_lxml=True).run()
    #print(counts)
    assert counts['insert'] == 0
    assert counts['exists'] == 3
    assert counts['skip'] == 1
    assert last_index == dblp_importer.api.get_changelog(limit=1)[0].index

def test_dblp_container_importer(dblp_container_importer):
    last_index = dblp_container_importer.api.get_changelog(limit=1)[0].index
    output_tsv_map = io.StringIO()
    with open('tests/files/example_dblp_containers.json', 'r') as f:
        dblp_container_importer.bezerk_mode = True
        dblp_container_importer.dblp_container_map_output = output_tsv_map
        counts = JsonLinePusher(dblp_container_importer, f).run()
    assert counts['insert'] == 10
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = dblp_container_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "dblp" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.DblpContainerImporter" in eg.extra['agent']

    # check that entity name mangling was fixed on import
    eg = dblp_container_importer.api.get_editgroup(eg.editgroup_id)
    for c_edit in eg.edits.containers:
        container = dblp_container_importer.api.get_container(c_edit.ident)
        if container.issnl == "1877-3273":
            break
    assert container.name == "Atlantis Thinking Machines"
    assert container.issnl == "1877-3273"
    assert container.container_type == "book-series"
    assert container.extra['dblp']['prefix'] == "series/atlantis"
    assert container.extra['urls'] == ["http://link.springer.com/bookseries/10077"]

    last_index = dblp_container_importer.api.get_changelog(limit=1)[0].index
    output_tsv_map.seek(0)
    #print(output_tsv_map.read())
    output_tsv_map.seek(0)
    with open('tests/files/example_dblp_containers.json', 'r') as f:
        dblp_container_importer.reset()
        dblp_container_importer.bezerk_mode = False
        dblp_container_importer.dblp_container_map_output = io.StringIO()
        dblp_container_importer.read_dblp_container_map_file(output_tsv_map)
        counts = JsonLinePusher(dblp_container_importer, f).run()
    print(counts)
    assert counts['insert'] == 0
    assert counts['exists'] == 10
    assert counts['skip'] == 0
    assert last_index == dblp_container_importer.api.get_changelog(limit=1)[0].index

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
    #assert r1.extra['container_name'] == "Commun. ACM"
    assert r1.extra['dblp']['type'] == "article"
