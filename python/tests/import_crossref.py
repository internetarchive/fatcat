
import json, gzip
import pytest
from fatcat_tools.importers import CrossrefImporter, JsonLinePusher
from fixtures import api


@pytest.fixture(scope="function")
def crossref_importer(api):
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield CrossrefImporter(api, issn_file, extid_map_file='tests/files/example_map.sqlite3', bezerk_mode=True)

@pytest.fixture(scope="function")
def crossref_importer_existing(api):
    with open('tests/files/ISSN-to-ISSN-L.snip.txt', 'r') as issn_file:
        yield CrossrefImporter(api, issn_file, extid_map_file='tests/files/example_map.sqlite3', bezerk_mode=False)

@pytest.mark.skip(reason="slow/huge crossref import is a corner-case and slows tests significantly")
def test_crossref_importer_huge(crossref_importer):
    last_index = crossref_importer.api.get_changelog(limit=1)[0].index
    with gzip.open('tests/files/huge_crossref_doi.json.gz', 'rt') as f:
        crossref_importer.bezerk_mode = True
        line = f.readline()
        mega_blob = [line for i in range(95)]
        counts = JsonLinePusher(crossref_importer, mega_blob).run()
    assert counts['insert'] == 95
    change = crossref_importer.api.get_changelog_entry(index=last_index+1)
    release = crossref_importer.api.get_release(change.editgroup.edits.releases[0].ident)
    assert len(release.contribs) == 1014

def test_crossref_importer(crossref_importer):
    last_index = crossref_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        crossref_importer.bezerk_mode = True
        counts = JsonLinePusher(crossref_importer, f).run()
    assert counts['insert'] == 14
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = crossref_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "crossref" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.CrossrefImporter" in eg.extra['agent']

    last_index = crossref_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        crossref_importer.bezerk_mode = False
        crossref_importer.reset()
        counts = JsonLinePusher(crossref_importer, f).run()
    assert counts['insert'] == 0
    assert counts['exists'] == 14
    assert counts['skip'] == 0
    assert last_index == crossref_importer.api.get_changelog(limit=1)[0].index

def test_crossref_mappings(crossref_importer):
    assert crossref_importer.map_release_type('journal-article') == "article-journal"
    assert crossref_importer.map_release_type('asdf') is None
    assert crossref_importer.map_release_type('component') is None
    assert crossref_importer.map_release_type('standard') == 'standard'

def test_crossref_importer_create(crossref_importer):
    crossref_importer.create_containers = True
    with open('tests/files/crossref-works.2018-01-21.badsample.json', 'r') as f:
        JsonLinePusher(crossref_importer, f).run()

def test_crossref_dict_parse(crossref_importer):
    with open('tests/files/crossref-works.single.json', 'r') as f:
        # not a single line
        raw = json.loads(f.read())
        r = crossref_importer.parse_record(raw)
        # ensure the API server is ok with format
        JsonLinePusher(crossref_importer, [json.dumps(raw)]).run()

        print(r.extra)
        assert r.title == "Renormalized perturbation theory by the moment method for degenerate states: Anharmonic oscillators"
        assert r.publisher == "Wiley-Blackwell"
        assert r.release_type == "article-journal"
        assert r.release_stage == "published"
        assert r.license_slug == "CC-BY-NC-ND"
        assert r.original_title == "Renormalized perturbation theory auf deutsch"
        assert r.ext_ids.doi == "10.1002/(sici)1097-461x(1998)66:4<261::aid-qua1>3.0.co;2-t"
        assert r.ext_ids.isbn13 == "978-3-16-148410-0"
        assert r.language == "fr"
        assert r.subtitle == None
        assert 'subtitle' not in r.extra
        assert 'subtitle' not in r.extra['crossref']
        assert 'funder' not in r.extra
        assert 'funder' not in r.extra['crossref']
        # matched by ISSN, so shouldn't be in there
        #assert extra['container_name'] == "International Journal of Quantum Chemistry"
        assert r.extra['aliases'] == ["some other title"]
        assert r.extra['crossref']['archive'] == ['Portico', 'LOCKSS']
        assert len(r.contribs) == 6
        assert r.contribs[0].raw_name == "Marcelo D. Radicioni"
        assert r.contribs[0].given_name == "Marcelo D."
        assert r.contribs[0].surname == "Radicioni"
        assert r.contribs[0].index == 0
        assert r.contribs[0].extra['seq'] == "first"
        assert r.contribs[1].raw_affiliation == "Some University"
        assert r.contribs[1].extra['more_affiliations'] == ["Some Department"]
        assert r.contribs[1].role == "author"
        assert r.contribs[4].role == "editor"
        assert r.contribs[4].index is None
        assert r.contribs[4].extra is None
        assert r.contribs[5].role == "translator"
        assert r.contribs[5].index is None
        assert len(r.refs) == 25
        assert r.refs[0].key == "BIB1"
        assert r.refs[0].year == 1972
        assert r.refs[0].locator == "1734"
        assert r.refs[0].container_name == "J. Chem. Phys."
        assert r.refs[0].extra == {"volume": "57", "authors": ["Swenson"], "doi": "10.1063/1.1678462", "medium": "DVD"}
        assert r.refs[2].key == 'BIB3'
        assert r.refs[2].extra.get('author') is None
        assert r.refs[2].container_name == "Hypervirial Theorem's, Lecture Notes in Chemistry <3"
        assert r.refs[3].container_name == "Large Order Perturbation Theory and Summation Methods in Quantum Mechanics, Lecture Notes in Chemistry"

def test_crossref_subtitle(crossref_importer):
    """
    Tests new subtitle field, explicitly
    """
    with open('tests/files/crossref-works.single.json', 'r') as f:
        # not a single line
        raw = json.loads(f.read())
        raw['subtitle'] = ["some bogus subtitle", "blah"]
        r = crossref_importer.parse_record(raw)
        # ensure the API server is ok with format
        JsonLinePusher(crossref_importer, [json.dumps(raw)]).run()

        print(r.extra)
        assert r.title == "Renormalized perturbation theory by the moment method for degenerate states: Anharmonic oscillators"
        assert r.subtitle == "some bogus subtitle"
        assert 'subtitle' not in r.extra
        assert 'subtitle' not in r.extra['crossref']

def test_stateful_checking(crossref_importer_existing):
    with open('tests/files/crossref-works.single.json', 'r') as f:
        # not a single line, a whole document
        raw = f.read()
        # might not exist yet...
        crossref_importer_existing.push_record(json.loads(raw))
        crossref_importer_existing.finish()
        # make sure we wouldn't insert again
        entity = crossref_importer_existing.parse_record(json.loads(raw))
        assert crossref_importer_existing.try_update(entity) is False
