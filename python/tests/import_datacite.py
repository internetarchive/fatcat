"""
Test datacite importer.
"""

import datetime
import pytest
import gzip
from fatcat_tools.importers import DataciteImporter, JsonLinePusher
from fixtures import api
import json


@pytest.fixture(scope="function")
def datacite_importer(api):
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield DataciteImporter(api, issn_file, extid_map_file='tests/files/example_map.sqlite3',
                               bezerk_mode=True)

@pytest.fixture(scope="function")
def datacite_importer_existing(api):
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield DataciteImporter(api, issn_file, extid_map_file='tests/files/example_map.sqlite3',
                               bezerk_mode=False)


@pytest.mark.skip(reason="larger datacite import slows tests down")
def test_datacite_importer_huge(datacite_importer):
    last_index = datacite_importer.api.get_changelog(limit=1)[0].index
    with gzip.open('tests/files/datacite_1k_records.jsonl.gz', 'rt') as f:
        datacite_importer.bezerk_mode = True
        counts = JsonLinePusher(datacite_importer, f).run()
    assert counts['insert'] == 998
    change = datacite_importer.api.get_changelog_entry(index=last_index+1)
    release = datacite_importer.api.get_release(change.editgroup.edits.releases[0].ident)
    assert len(release.contribs) == 3


def test_datacite_importer(datacite_importer):
    last_index = datacite_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/datacite_sample.jsonl', 'r') as f:
        datacite_importer.bezerk_mode = True
        counts = JsonLinePusher(datacite_importer, f).run()
    assert counts['insert'] == 1
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = datacite_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "datacite" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.DataciteImporter" in eg.extra['agent']

    last_index = datacite_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/datacite_sample.jsonl', 'r') as f:
        datacite_importer.bezerk_mode = False
        datacite_importer.reset()
        counts = JsonLinePusher(datacite_importer, f).run()
    assert counts['insert'] == 0
    assert counts['exists'] == 1
    assert counts['skip'] == 0
    assert last_index == datacite_importer.api.get_changelog(limit=1)[0].index

def test_datacite_dict_parse(datacite_importer):
    with open('tests/files/datacite_sample.jsonl', 'r') as f:
        raw = json.load(f)
        r = datacite_importer.parse_record(raw)
        # ensure the API server is ok with format
        JsonLinePusher(datacite_importer, [json.dumps(raw)]).run()

        print(r.extra)
        assert r.title == "Triticum turgidum L. subsp. durum (Desf.) Husn. 97090"
        assert r.publisher == "International Centre for Agricultural Research in Dry Areas"
        assert r.release_type == "article"
        assert r.release_stage == "published"
        assert r.license_slug == None
        assert r.original_title == "Triticum turgidum L. subsp. durum (Desf.) Husn. 97090"
        assert r.ext_ids.doi == "10.18730/8dym9"
        assert r.ext_ids.isbn13 == None
        assert r.language == "enc"
        assert r.subtitle == None
        assert r.release_date == None
        assert r.release_year == 1986
        assert 'subtitle' not in r.extra
        assert 'subtitle' not in r.extra['datacite']
        assert 'funder' not in r.extra
        assert 'funder' not in r.extra['datacite']
        # matched by ISSN, so shouldn't be in there
        #assert extra['container_name'] == "International Journal of Quantum Chemistry"
        assert r.extra['datacite']['url'] == 'https://ssl.fao.org/glis/doi/10.18730/8DYM9'
        assert r.extra['datacite']['subjects'] == [{'subject': 'Plant Genetic Resource for Food and Agriculture'}]
        assert len(r.abstracts) == 1
        assert len(r.abstracts[0].content) == 421
        assert len(r.contribs) == 1
        assert r.contribs[0].raw_name == "GLIS Of The ITPGRFA"
        assert r.contribs[0].given_name == None
        assert r.contribs[0].surname == None
        assert len(r.refs) == 0
