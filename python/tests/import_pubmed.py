
import json, gzip
import pytest
from fatcat_tools.importers import PubmedImporter, Bs4XmlFilePusher
from fixtures import api
from bs4 import BeautifulSoup


@pytest.fixture(scope="function")
def pubmed_importer(api):
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield PubmedImporter(api, issn_file, extid_map_file='tests/files/example_map.sqlite3', bezerk_mode=True, lookup_refs=True)

@pytest.fixture(scope="function")
def pubmed_importer_existing(api):
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield PubmedImporter(api, issn_file, extid_map_file='tests/files/example_map.sqlite3', bezerk_mode=False, lookup_refs=True)

def test_pubmed_importer(pubmed_importer):
    last_index = pubmed_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/pubmedsample_2019.xml', 'r') as f:
        pubmed_importer.bezerk_mode = True
        counts = Bs4XmlFilePusher(pubmed_importer, f, "PubmedArticle").run()
    assert counts['insert'] == 176
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = pubmed_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "pubmed" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.PubmedImporter" in eg.extra['agent']

    last_index = pubmed_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/pubmedsample_2019.xml', 'r') as f:
        pubmed_importer.bezerk_mode = False
        pubmed_importer.reset()
        counts = Bs4XmlFilePusher(pubmed_importer, f, "PubmedArticle").run()
    assert counts['insert'] == 0
    assert counts['exists'] == 176
    assert counts['skip'] == 0
    assert last_index == pubmed_importer.api.get_changelog(limit=1)[0].index

def test_pubmed_xml_parse(pubmed_importer):
    with open('tests/files/pubmedsample_2019.xml', 'r') as f:
        soup = BeautifulSoup(f, "xml")
        r1 = pubmed_importer.parse_record(soup.find_all("PubmedArticle")[0])
        r2 = pubmed_importer.parse_record(soup.find_all("PubmedArticle")[-1])

    assert r1.title == "Hospital debt management and cost reimbursement"
    assert r1.subtitle == None
    assert r1.original_title == None
    assert r1.publisher == None
    assert r1.release_type == "article-journal"
    assert r1.release_stage == "published"
    assert r1.license_slug == None
    assert r1.ext_ids.doi == None
    assert r1.ext_ids.pmid == "973217"
    assert r1.language == "en"
    assert r1.volume == "3"
    assert r1.issue == "1"
    assert r1.pages == "69-81"
    assert r1.release_date == None # not "1976-12-03", which is medline ingest date
    assert r1.release_year == 1976
    # matched by ISSN, so shouldn't be in there?
    #assert extra['container_name'] == "Abstracts of the Papers Communicated to the Royal Society of London"
    assert len(r1.contribs) == 1

    assert r1.contribs[0].raw_name == "F R Blume"
    assert r1.contribs[0].given_name == "F R"
    assert r1.contribs[0].surname == "Blume"

    print(r1.extra)
    assert r1.extra['pubmed']['pub_types'] == ['Journal Article']
    assert not r1.refs

    assert r2.title == "Synthesis and Antibacterial Activity of Metal(loid) Nanostructures by Environmental Multi-Metal(loid) Resistant Bacteria and Metal(loid)-Reducing Flavoproteins"
    assert r2.subtitle == None
    assert r2.original_title == None
    assert r2.publisher == None
    assert r2.release_type == "article-journal"
    assert r2.release_stage == "published"
    assert r2.license_slug == None
    assert r2.ext_ids.doi == "10.3389/fmicb.2018.00959"
    assert r2.ext_ids.pmid == "29869640"
    assert r2.ext_ids.pmcid == "PMC5962736"
    assert r2.language == "en"
    assert r2.volume == "9"
    assert r2.issue == None
    assert r2.pages == "959"
    assert str(r2.release_date) == "2018-05-15"
    assert r2.release_year == 2018
    # matched by ISSN, so shouldn't be in there?
    #assert extra['container_name'] == "Frontiers in microbiology"

    assert len(r2.contribs) > 3
    assert r2.contribs[0].raw_name == "Maximiliano Figueroa"
    assert r2.contribs[0].given_name == "Maximiliano"
    assert r2.contribs[0].surname == "Figueroa"
    assert r2.contribs[0].raw_affiliation == "Laboratorio Microbiología Molecular, Departamento de Biología, Facultad de Química y Biología, Universidad de Santiago de Chile, Santiago, Chile."
    assert r2.contribs[4].surname == "Muñoz-Villagrán"
    assert r2.contribs[7].surname == "Latorre"
    assert r2.contribs[7].raw_affiliation == "Mathomics, Centro de Modelamiento Matemático, Universidad de Chile, Beauchef, Santiago, Chile."
    assert r2.contribs[7].extra['more_affiliations'] == [
        "Fondap-Center of Genome Regulation, Facultad de Ciencias, Universidad de Chile, Santiago, Chile.",
        "Laboratorio de Bioinformática y Expresión Génica, INTA, Universidad de Chile, Santiago, Chile.",
        "Instituto de Ciencias de la Ingeniería, Universidad de O'Higgins, Rancagua, Chile.",
    ]
    assert r2.contribs[-1].raw_name == "Felipe Arenas"

    assert r2.abstracts[0].content.startswith("Microbes are suitable candidates to recover and decontaminate different environments from soluble metal ions, either via reduction")
    assert r2.abstracts[0].lang == "en"

    print(r2.extra)
    assert r2.extra['pubmed']['pub_types'] == ['Journal Article']

    assert r2.refs[0].extra['unstructured'] == "Microbiology. 2009 Jun;155(Pt 6):1840-6"
    assert r2.refs[0].extra['pmid'] == "19383690"

