
import pytest
from bs4 import BeautifulSoup
from fixtures import *

from fatcat_tools.importers import ArxivRawImporter, Bs4XmlFilePusher


@pytest.fixture(scope="function")
def arxiv_importer(api):
    ari = ArxivRawImporter(api, bezerk_mode=True)
    ari._test_override = True
    return ari

def test_arxiv_importer(arxiv_importer):
    last_index = arxiv_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/arxivraw_1810.09584.xml', 'r') as f:
        arxiv_importer.bezerk_mode = True
        counts = Bs4XmlFilePusher(arxiv_importer, f, "record").run()
    assert counts['insert'] == 2
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = arxiv_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "arxiv" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.ArxivRawImporter" in eg.extra['agent']

    last_index = arxiv_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/arxivraw_1810.09584.xml', 'r') as f:
        arxiv_importer.bezerk_mode = False
        arxiv_importer.reset()
        counts = Bs4XmlFilePusher(arxiv_importer, f, "record").run()
    assert counts['insert'] == 0
    assert counts['exists'] == 2
    assert counts['skip'] == 0
    assert last_index == arxiv_importer.api.get_changelog(limit=1)[0].index

def test_arxiv_xml_parse(arxiv_importer):
    with open('tests/files/arxivraw_1810.09584.xml', 'r') as f:
        soup = BeautifulSoup(f, "xml")
        r = arxiv_importer.parse_record(soup.find_all("record")[0])

    r1 = r[0]
    r2 = r[1]
    print(r1.extra)
    print(r2.extra)
    assert r1.work_id == r2.work_id
    assert r1.title == "Martingale theory for housekeeping heat"
    assert r1.subtitle is None
    assert r1.original_title is None
    assert r1.release_type == "article"
    assert r1.release_stage == "submitted"
    assert r2.release_stage == "accepted"
    assert r1.license_slug == "ARXIV-1.0"
    assert r1.version == "v1"
    assert r2.version == "v2"
    assert r1.ext_ids.arxiv == "1810.09584v1"
    assert r2.ext_ids.arxiv == "1810.09584v2"
    assert r1.ext_ids.doi is None
    assert r2.ext_ids.doi == "10.1209/0295-5075/124/60006"
    assert r1.release_year == 2018
    assert str(r1.release_date) == "2018-10-22"
    assert r2.release_year == 2019
    assert str(r2.release_date) == "2019-01-13"
    # matched by ISSN, so shouldn't be in there?
    #assert extra['container_name'] == "Abstracts of the Papers Communicated to the Royal Society of London"
    assert len(r1.contribs) == 4
    assert r1.extra['arxiv']['categories'] == ['cond-mat.stat-mech', 'physics.bio-ph', 'physics.data-an']
    assert r1.extra['arxiv']['base_id'] == '1810.09584'
    assert r1.extra['superceded'] is True

    assert r1.contribs[0].raw_name == "Raphael Chetrite"
    assert r1.contribs[0].role == "author"
    assert r1.contribs[1].raw_name == "Shamik Gupta"
    assert r1.contribs[2].raw_name == "Izaak Neri"
    assert r1.contribs[3].raw_name == "Édgar Roldán"
    assert r1.contribs[3].role == "author"

    assert len(r1.contribs) == 4
    assert r1.contribs == r2.contribs

    assert r1.abstracts[0].content.startswith("The housekeeping heat is the energy exchanged")
    # order isn't deterministic
    assert "application/x-latex" in [a.mimetype for a in r1.abstracts]
    assert "text/plain" in [a.mimetype for a in r1.abstracts]

    assert r1.abstracts == r2.abstracts

    assert r1.extra['arxiv']['comments'] == "7 pages, 2 figures"
    assert r1.extra['arxiv']['categories'] == ["cond-mat.stat-mech", "physics.bio-ph", "physics.data-an"]

    assert not r2.extra.get('superceded')
    r2.extra['superceded'] = True
    assert r1.extra == r2.extra

    assert not r1.refs
