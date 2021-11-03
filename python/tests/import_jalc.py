import pytest
from bs4 import BeautifulSoup
from fixtures import *

from fatcat_tools.importers import Bs4XmlFilePusher, Bs4XmlLinesPusher, JalcImporter


@pytest.fixture(scope="function")
def jalc_importer(api):
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        yield JalcImporter(
            api, issn_file, extid_map_file="tests/files/example_map.sqlite3", bezerk_mode=True
        )


@pytest.fixture(scope="function")
def jalc_importer_existing(api):
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        yield JalcImporter(
            api, issn_file, extid_map_file="tests/files/example_map.sqlite3", bezerk_mode=False
        )


def test_jalc_importer(jalc_importer):
    last_index = jalc_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/jalc_lod_sample.xml", "r") as f:
        jalc_importer.bezerk_mode = True
        counts = Bs4XmlFilePusher(jalc_importer, f, "Description").run()
    assert counts["insert"] == 2
    assert counts["exists"] == 0
    assert counts["skip"] == 0

    # fetch most recent editgroup
    change = jalc_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "jalc" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.JalcImporter" in eg.extra["agent"]

    last_index = jalc_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/jalc_lod_sample.xml", "r") as f:
        jalc_importer.bezerk_mode = False
        jalc_importer.reset()
        counts = Bs4XmlFilePusher(jalc_importer, f, "Description").run()
    assert counts["insert"] == 0
    assert counts["exists"] == 2
    assert counts["skip"] == 0
    assert last_index == jalc_importer.api.get_changelog(limit=1)[0].index


def test_jalc_importer_lines(jalc_importer):
    last_index = jalc_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/jalc_rdf_sample_100.xml", "r") as f:
        jalc_importer.bezerk_mode = True
        counts = Bs4XmlLinesPusher(jalc_importer, f, "<rdf:Description").run()
    assert counts["insert"] == 93
    assert counts["exists"] == 0
    assert counts["skip"] == 0

    # fetch most recent editgroup
    change = jalc_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "jalc" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.JalcImporter" in eg.extra["agent"]

    last_index = jalc_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/jalc_rdf_sample_100.xml", "r") as f:
        jalc_importer.bezerk_mode = False
        jalc_importer.reset()
        counts = Bs4XmlLinesPusher(jalc_importer, f, "<rdf:Description").run()
    assert counts["insert"] == 0
    assert counts["exists"] == 93
    assert counts["skip"] == 0
    assert last_index == jalc_importer.api.get_changelog(limit=1)[0].index


def test_jalc_xml_parse(jalc_importer):
    with open("tests/files/jalc_lod_sample.xml", "r") as f:
        soup = BeautifulSoup(f, "xml")
        r = jalc_importer.parse_record(soup.find_all("Description")[0])

    print(r.extra)
    assert r.title == "New carbides in the Ni-Ti-Mo-C system"
    assert r.subtitle is None
    assert r.original_title == "Ni-Ti-Mo-C系に出現する新炭化物相について"
    assert r.publisher == "Japan Society of Powder and Powder Metallurgy"
    assert r.release_type == "article-journal"
    assert r.release_stage == "published"
    assert r.license_slug is None
    assert r.ext_ids.doi == "10.2497/jjspm.36.898"
    assert r.language == "ja"
    assert r.volume == "36"
    assert r.issue == "8"
    assert r.pages == "898-902"
    assert r.release_year == 1989
    # matched by ISSN, so shouldn't be in there?
    # assert extra['container_name'] == "International Journal of Quantum Chemistry"
    assert len(r.contribs) == 4

    assert r.contribs[0].raw_name == "Yasuhiko Hashimoto"
    assert r.contribs[0].given_name == "Yasuhiko"
    assert r.contribs[0].surname == "Hashimoto"
    assert r.contribs[0].extra["original_name"]["raw_name"] == "橋本 雍彦"
    assert r.contribs[0].extra["original_name"]["given_name"] == "雍彦"
    assert r.contribs[0].extra["original_name"]["surname"] == "橋本"

    assert r.contribs[3].raw_name == "Teruo Takahashi"
    assert r.contribs[3].given_name == "Teruo"
    assert r.contribs[3].surname == "Takahashi"
    assert r.contribs[3].extra["original_name"]["raw_name"] == "高橋 輝男"
    assert r.contribs[3].extra["original_name"]["given_name"] == "輝男"
    assert r.contribs[3].extra["original_name"]["surname"] == "高橋"

    assert not r.refs
