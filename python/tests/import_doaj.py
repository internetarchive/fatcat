
import datetime
import json

import elasticsearch
import fatcat_openapi_client
import pytest
from fixtures import *

from fatcat_tools.importers import DoajArticleImporter, JsonLinePusher
from fatcat_tools.transforms import entity_to_dict


@pytest.fixture(scope="function")
def doaj_importer(api, mocker):
    es_client = elasticsearch.Elasticsearch("mockbackend")
    mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        yield DoajArticleImporter(
            api,
            issn_file,
            bezerk_mode=True,
            es_client=es_client,
        )

def test_doaj_importer(doaj_importer):
    last_index = doaj_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/example_doaj_articles.json", "r") as f:
        doaj_importer.bezerk_mode = True
        doaj_importer.do_fuzzy_match = False
        counts = JsonLinePusher(doaj_importer, f).run()
    assert counts["insert"] == 5
    assert counts["exists"] == 0
    assert counts["skip"] == 0
    success_changelog = doaj_importer.api.get_changelog(limit=1)[0]
    assert last_index + 1 == success_changelog.index

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

    # cleanup file entities (so other import tests work)
    success_editgroup = doaj_importer.api.get_editgroup(success_changelog.editgroup_id)
    eg = quick_eg(doaj_importer.api)
    for release_edit in success_editgroup.edits.releases:
        doaj_importer.api.delete_release(eg.editgroup_id, release_edit.ident)
    doaj_importer.api.accept_editgroup(eg.editgroup_id)

def test_doaj_importer_existing_doi(doaj_importer):
    """
    One of the DOAJ test entities has a dummy DOI (10.123/abc); this test
    ensures that it isn't clobbered, an then that it gets updated.
    """
    with open("tests/files/example_doaj_articles.json", "r") as f:
        doaj_importer.reset()
        doaj_importer.bezerk_mode = False
        doaj_importer.do_updates = False
        doaj_importer.do_fuzzy_match = False
        counts = JsonLinePusher(doaj_importer, f).run()
    print(counts)
    assert counts["insert"] == 4
    assert counts["exists"] == 1
    assert counts["skip"] == 0
    success_changelog = doaj_importer.api.get_changelog(limit=1)[0]
    success_editgroup = doaj_importer.api.get_editgroup(success_changelog.editgroup_id)

    with open("tests/files/example_doaj_articles.json", "r") as f:
        doaj_importer.reset()
        doaj_importer.bezerk_mode = False
        doaj_importer.do_updates = True
        doaj_importer.do_fuzzy_match = False
        counts = JsonLinePusher(doaj_importer, f).run()
    print(counts)
    assert counts["insert"] == 0
    assert counts["exists"] == 4
    assert counts["update"] == 1
    update_changelog = doaj_importer.api.get_changelog(limit=1)[0]
    update_editgroup = doaj_importer.api.get_editgroup(update_changelog.editgroup_id)

    with open("tests/files/example_doaj_articles.json", "r") as f:
        doaj_importer.reset()
        doaj_importer.bezerk_mode = False
        doaj_importer.do_updates = True
        doaj_importer.do_fuzzy_match = False
        counts = JsonLinePusher(doaj_importer, f).run()
    print(counts)
    assert counts["insert"] == 0
    assert counts["exists"] == 5
    assert counts["update"] == 0

    # cleanup file entities (so other import tests work)
    eg = quick_eg(doaj_importer.api)
    for release_edit in success_editgroup.edits.releases:
        doaj_importer.api.delete_release(eg.editgroup_id, release_edit.ident)
    for release_edit in update_editgroup.edits.releases:
        print(release_edit)
        doaj_importer.api.update_release(
            eg.editgroup_id,
            release_edit.ident,
            ReleaseEntity(
                revision=release_edit.prev_revision,
                ext_ids=ReleaseExtIds(),
            ),
        )
    doaj_importer.api.accept_editgroup(eg.editgroup_id)

def test_doaj_dict_parse(doaj_importer):
    with open("tests/files/example_doaj_articles.json", "r") as f:
        raw = json.loads(f.readline())
        r = doaj_importer.parse_record(raw)

        assert r.title == "Effect of hydrogen on tensile properties and fracture behavior of PH 13-8 Mo steel"
        assert r.publisher == "Elsevier"
        assert r.release_type == "article-journal"
        assert r.release_stage == "published"
        assert r.license_slug == "cc-by-nc-nd"
        assert r.original_title is None
        assert r.ext_ids.doi == "10.1016/j.matdes.2016.06.110"
        assert r.ext_ids.doaj == "e58f08a11ecb495ead55a44ad4f89808"
        assert r.subtitle is None
        assert r.release_date is None
        assert r.release_year == 2016
        assert r.volume == "108"
        assert r.number is None
        assert r.pages == "608-617"
        assert r.version is None
        assert r.language == "en"
        # matched by ISSN, so wouldn't be defined normally
        assert r.extra['container_name'] == "Materials & Design"
        assert len(r.abstracts) == 1
        assert len(r.abstracts[0].content) == 1033
        assert len(r.contribs) == 5
        assert r.contribs[0].raw_name == "Xinfeng Li"
        assert r.contribs[0].given_name is None
        assert r.contribs[0].surname is None
        assert not r.refs

        #print(r.extra)
        assert r.extra['release_month'] == 10
        assert r.extra['country'] == 'gb'
