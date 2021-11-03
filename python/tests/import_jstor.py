import pytest
from bs4 import BeautifulSoup
from fixtures import *

from fatcat_tools.importers import Bs4XmlFilePusher, JstorImporter


@pytest.fixture(scope="function")
def jstor_importer(api):
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        yield JstorImporter(
            api, issn_file, extid_map_file="tests/files/example_map.sqlite3", bezerk_mode=True
        )


@pytest.fixture(scope="function")
def jstor_importer_existing(api):
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        yield JstorImporter(
            api, issn_file, extid_map_file="tests/files/example_map.sqlite3", bezerk_mode=False
        )


def test_jstor_importer(jstor_importer):
    last_index = jstor_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/jstor-article-10.2307_111039.xml", "r") as f:
        jstor_importer.bezerk_mode = True
        counts = Bs4XmlFilePusher(jstor_importer, f, "article").run()
    assert counts["insert"] == 1
    assert counts["exists"] == 0
    assert counts["skip"] == 0

    # fetch most recent editgroup
    change = jstor_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "jstor" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.JstorImporter" in eg.extra["agent"]

    last_index = jstor_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/jstor-article-10.2307_111039.xml", "r") as f:
        jstor_importer.bezerk_mode = False
        jstor_importer.reset()
        counts = Bs4XmlFilePusher(jstor_importer, f, "article").run()
    assert counts["insert"] == 0
    assert counts["exists"] == 1
    assert counts["skip"] == 0
    assert last_index == jstor_importer.api.get_changelog(limit=1)[0].index


def test_jstor_xml_parse(jstor_importer):
    with open("tests/files/jstor-article-10.2307_111039.xml", "r") as f:
        soup = BeautifulSoup(f, "xml")
        r = jstor_importer.parse_record(soup.find_all("article")[0])

    print(r.extra)
    assert (
        r.title
        == "On the Universal Law of Attraction, Including that of Gravitation, as a Particular Case of Approximation Deducible from the Principle that Equal and Similar Particles of Matter Move Similarly, Relatively to Each other. [Abstract]"
    )
    assert r.subtitle is None
    assert r.original_title is None
    assert r.publisher == "The Royal Society"
    assert r.release_type == "abstract"
    assert r.release_stage == "published"
    assert r.license_slug is None
    assert r.ext_ids.doi is None
    assert r.ext_ids.jstor == "111039"
    assert r.language == "en"
    assert r.volume == "5"
    assert r.issue is None
    assert r.pages == "831-832"
    # None because jan 1st
    assert r.release_date is None
    assert r.release_year == 1843
    # matched by ISSN, so shouldn't be in there?
    # assert extra['container_name'] == "Abstracts of the Papers Communicated to the Royal Society of London"
    assert len(r.contribs) == 1
    assert r.extra["jstor"]["journal_ids"] == ["abstpapecommroya", "j100687"]

    assert r.contribs[0].raw_name == "John Kinnersley Smythies"
    assert r.contribs[0].given_name == "John Kinnersley"
    assert r.contribs[0].surname == "Smythies"

    assert r.refs is None
