
import json
import datetime

import pytest

from fatcat_tools.importers import DoajArticleImporter, JsonLinePusher
from fatcat_tools.transforms import entity_to_dict
import fatcat_openapi_client
from fixtures import api
import json


@pytest.fixture(scope="function")
def doaj_importer(api):
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        yield DoajArticleImporter(api, issn_file, bezerk_mode=True)

@pytest.fixture(scope="function")
def doaj_importer_existing(api):
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        yield DoajArticleImporter(api, issn_file, bezerk_mode=False)

def test_doaj_importer(doaj_importer):
    last_index = doaj_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/example_doaj_articles.json", "r") as f:
        doaj_importer.bezerk_mode = True
        counts = JsonLinePusher(doaj_importer, f).run()
    assert counts["insert"] == 5
    assert counts["exists"] == 0
    assert counts["skip"] == 0

    # fetch most recent editgroup
    change = doaj_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "doaj" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.DoajArticleImporter" in eg.extra["agent"]

    last_index = doaj_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/example_doaj_articles.json", "r") as f:
        doaj_importer.bezerk_mode = False
        doaj_importer.reset()
        counts = JsonLinePusher(doaj_importer, f).run()
    assert counts["insert"] == 0
    assert counts["exists"] == 5
    assert counts["skip"] == 0
    assert last_index == doaj_importer.api.get_changelog(limit=1)[0].index


def test_doaj_dict_parse(doaj_importer):
    with open("tests/files/example_doaj_articles.json", "r") as f:
        raw = json.loads(f.readline())
        r = doaj_importer.parse_record(raw)
        # ensure the API server is ok with format
        JsonLinePusher(doaj_importer, [json.dumps(raw)]).run()

        assert r.title == "Effect of hydrogen on tensile properties and fracture behavior of PH 13-8 Mo steel"
        assert r.publisher == "Elsevier"
        assert r.release_type == "article-journal"
        assert r.release_stage == "published"
        assert r.license_slug == "cc-by-nc-nd"
        assert r.original_title == None
        assert r.ext_ids.doi == "10.1016/j.matdes.2016.06.110"
        assert r.ext_ids.doaj == "e58f08a11ecb495ead55a44ad4f89808"
        assert r.subtitle == None
        assert r.release_date == None
        assert r.release_year == 2016
        assert r.volume == "108"
        assert r.number == None
        assert r.pages == "608-617"
        assert r.version == None
        assert r.language == "en"
        # matched by ISSN, so wouldn't be defined normally
        assert r.extra['container_name'] == "Materials & Design"
        assert len(r.abstracts) == 1
        assert len(r.abstracts[0].content) == 1033
        assert len(r.contribs) == 5
        assert r.contribs[0].raw_name == "Xinfeng Li"
        assert r.contribs[0].given_name == None
        assert r.contribs[0].surname == None
        assert not r.refs

        #print(r.extra)
        assert r.extra['release_month'] == 10
        assert r.extra['country'] == 'gb'
