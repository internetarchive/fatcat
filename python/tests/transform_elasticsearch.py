
import json
import pytest
from fatcat_tools import *
from fatcat_openapi_client import *
from fixtures import api
from import_journal_metadata import journal_metadata_importer

from import_crossref import crossref_importer
from import_matched import matched_importer

def test_basic_elasticsearch_convert(crossref_importer):
    with open('tests/files/crossref-works.single.json', 'r') as f:
        # not a single line
        raw = json.loads(f.read())
        r = crossref_importer.parse_record(raw)
    r.state = 'active'
    release_to_elasticsearch(r)

def test_rich_elasticsearch_convert():
    r = ReleaseEntity(
        title="something",
        release_year=1234,
        license_slug="CC-BY-NC",
        ext_ids=ReleaseExtIds(),
        refs=[
            ReleaseRef(),
            ReleaseRef(target_release_id="iznnn644szdwva7khyxqzc73bi"),
        ],
    )
    r.state = 'active'
    r.container = ContainerEntity(
        name="dummy journal",
        extra={
            "ia": {
                "sim": {
                    "year_spans": [[1000, 1100]],
                },
            },
            "kbart": {
                "lockss": {
                    "year_spans": [[1200, 1300]],
                },
                "jstor": {
                    "year_spans": [[1950, 1960], [1980, 2005]],
                },
            },
            "sherpa_romeo": {"color": "blue"},
            "doaj": {"as_of": "2010-02-03"},
        },
    )
    r.files = [FileEntity(
        mimetype="application/pdf",
        urls=[
            FileUrl(rel="dweb", url="dat://a954329dlk/thingie"),
            FileUrl(rel="webarchive", url="https://web.archive.org/web/20001122030405/http://example.com"),
            FileUrl(rel="web", url="https://archive.org/details/blah/file.pdf"),
        ],
        extra={
            "shadows": {},
        },
    )]
    es = release_to_elasticsearch(r)
    assert es['release_year'] == r.release_year
    assert es['in_ia'] == True
    assert es['in_jstor'] == False
    assert es['in_ia_sim'] == False
    assert es['in_ia'] == True
    assert es['in_web'] == True
    assert es['in_dweb'] == True
    assert es['is_oa'] == True
    assert es['is_longtail_oa'] == False
    assert es['ref_count'] == 2
    assert es['ref_linked_count'] == 1

def test_elasticsearch_release_from_json():
    r = entity_from_json(open('./tests/files/release_etodop5banbndg3faecnfm6ozi.json', 'r').read(), ReleaseEntity)
    es = release_to_elasticsearch(r)

    assert es['subtitle'] == "Correpondence"
    assert es['ident'] == "etodop5banbndg3faecnfm6ozi"
    assert es['container_name'] == "BJOG: an International Journal of Obstetrics and Gynaecology"
    assert es['first_page'] == "1404"
    assert es['issue'] == "11"
    assert es['volume'] == "118"
    assert es['number'] == None
    assert es['in_ia_sim'] == True
    assert es['in_kbart'] == True

def test_elasticsearch_container_transform(journal_metadata_importer):
    with open('tests/files/journal_metadata.sample.json', 'r') as f:
        raw = json.loads(f.readline())
        c = journal_metadata_importer.parse_record(raw)
    c.state = 'active'
    es = container_to_elasticsearch(c)
    assert es['publisher'] == c.publisher

def test_elasticsearch_file_transform(matched_importer):
    f = entity_from_json(open('./tests/files/file_bcah4zp5tvdhjl5bqci2c2lgfa.json', 'r').read(), FileEntity)

    f.state = 'active'
    es = file_to_elasticsearch(f)
    assert es['sha1'] == f.sha1
    assert es['sha256'] == f.sha256
    assert es['md5'] == f.md5
    assert es['size_bytes'] == f.size
    assert es['mimetype'] == f.mimetype
    assert es['in_ia'] == True

    assert 'web' in es['rels']
    assert 'www.zhros.ru' in es['hosts']
    assert 'zhros.ru' in es['domains']
    assert not '.archive.org' in (es['hosts'] + es['domains'])
    assert not 'archive.org' in (es['hosts'] + es['domains'])
    assert not 'web.archive.org' in (es['hosts'] + es['domains'])

def test_elasticsearch_changelog_transform(matched_importer):
    ce = entity_from_json(open('./tests/files/changelog_3469683.json', 'r').read(), ChangelogEntry)

    es = changelog_to_elasticsearch(ce)
    assert es['index'] == 3469683
    # len("2020-01-30T05:04:39") => 19
    assert es['timestamp'][:19] == "2020-01-30T05:04:39.738601Z"[:19]
    assert es['editor_id'] == "scmbogxw25evtcesfcab5qaboa"
    assert es['username'] == "crawl-bot"
    assert es['is_bot'] == True
    assert es['is_admin'] == True
    assert es['agent'] == "fatcat_tools.IngestFileResultImporter"

    assert es['total'] == 50
    assert es['files'] == 50
    assert es['new_files'] == 50
    assert es['created'] == 50

    assert es['releases'] == 0
    assert es['new_releases'] == 0
    assert es['updated'] == 0
    assert es['deleted'] == 0
