import datetime
import json

from fatcat_openapi_client import (
    ChangelogEntry,
    ContainerEntity,
    FileEntity,
    FileUrl,
    ReleaseEntity,
    ReleaseExtIds,
    ReleaseRef,
)
from fixtures import api
from import_crossref import crossref_importer
from import_journal_metadata import journal_metadata_importer

from fatcat_tools.transforms import (
    changelog_to_elasticsearch,
    container_to_elasticsearch,
    entity_from_json,
    file_to_elasticsearch,
    release_to_elasticsearch,
)


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
                    "year_spans": [[1000, 1300], [1950, 1960], [1980, 2005]],
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
    assert es['file_count'] == 1
    assert es['fileset_count'] == 0
    assert es['webcapture_count'] == 0
    assert es['ref_count'] == 2
    assert es['ref_linked_count'] == 1

    assert es['preservation'] == "bright"
    assert es['is_oa'] == True
    assert es['is_longtail_oa'] == False
    assert es['is_preserved'] == True
    assert es['in_web'] == True
    assert es['in_dweb'] == True
    assert es['in_ia'] == True
    assert es['in_ia_sim'] == False
    assert es['in_kbart'] == True
    assert es['in_jstor'] == True

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

    assert es['preservation'] == "dark"
    assert es['is_oa'] == False
    assert es['is_longtail_oa'] == False
    assert es['is_preserved'] == True
    assert es['in_web'] == False
    assert es['in_dweb'] == False
    assert es['in_ia'] == False
    assert es['in_ia_sim'] == True
    assert es['in_kbart'] == True
    assert es['in_jstor'] == False

    # this release has a fileset, and no file
    r = entity_from_json(open('./tests/files/release_3mssw2qnlnblbk7oqyv2dafgey.json', 'r').read(), ReleaseEntity)
    es = release_to_elasticsearch(r)

    assert es['title'] == "Jakobshavn Glacier Bed Elevation"
    assert es['ident'] == "3mssw2qnlnblbk7oqyv2dafgey"
    assert es['file_count'] == 0
    assert es['fileset_count'] == 1
    assert es['webcapture_count'] == 0

    assert es['preservation'] == "dark"
    assert es['is_oa'] == True
    assert es['is_longtail_oa'] == False
    assert es['is_preserved'] == True
    assert es['in_web'] == True
    assert es['in_dweb'] == True
    assert es['in_ia'] == False
    assert es['in_ia_sim'] == False
    assert es['in_kbart'] == False
    assert es['in_jstor'] == False

    # this release has a web capture, and no file (edited the JSON to remove file)
    r = entity_from_json(open('./tests/files/release_mjtqtuyhwfdr7j2c3l36uor7uy.json', 'r').read(), ReleaseEntity)
    es = release_to_elasticsearch(r)

    assert es['title'] == "Rethinking Personal Digital Archiving, Part 1"
    assert es['ident'] == "mjtqtuyhwfdr7j2c3l36uor7uy"
    assert es['file_count'] == 0
    assert es['fileset_count'] == 0
    assert es['webcapture_count'] == 1

    assert es['preservation'] == "bright"
    assert es['is_oa'] == True
    assert es['is_longtail_oa'] == False
    assert es['is_preserved'] == True
    assert es['in_web'] == True
    assert es['in_dweb'] == False
    assert es['in_ia'] == True
    assert es['in_ia_sim'] == False
    assert es['in_kbart'] == False
    assert es['in_jstor'] == False

def test_elasticsearch_container_transform(journal_metadata_importer):
    with open('tests/files/journal_metadata.sample.json', 'r') as f:
        raw1 = json.loads(f.readline())
        raw2 = json.loads(f.readline())
        c1 = journal_metadata_importer.parse_record(raw1)
        c1.state = 'active'
        c2 = journal_metadata_importer.parse_record(raw2)
        c2.state = 'active'

    c1.extra['publisher_type'] = "big5"
    c1.extra['discipline'] = "history"
    es = container_to_elasticsearch(c1)
    assert es['publisher'] == c1.publisher
    assert es['discipline'] == c1.extra['discipline']
    assert es['publisher_type'] == c1.extra['publisher_type']
    assert es['keepers'] == []

    stats = {
        "ident": "en4qj5ijrbf5djxx7p5zzpjyoq",
        "in_kbart": 11136,
        "in_web": 9501,
        "is_preserved": 11136,
        "issnl": "2050-084X",
        "preservation": {
            "bright": 9501,
            "dark": 1635,
            "none": 0,
            "shadows_only": 0,
            "total": 11136
        },
        "release_type": {
            "_unknown": 9,
            "article-journal": 11124,
            "editorial": 2,
            "letter": 1
        },
        "total": 11136
    }
    es = container_to_elasticsearch(c2, stats=stats)
    assert es['name'] == c2.name
    assert es['publisher'] == c2.publisher
    assert es['keepers'] == list(c2.extra['kbart'].keys()) == ["portico"]
    assert es['any_kbart'] == True


def test_elasticsearch_file_transform():
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
    assert 'archive.org' in (es['hosts'] + es['domains'])
    assert 'web.archive.org' in (es['hosts'] + es['domains'])
    # old regression
    assert not '.archive.org' in (es['hosts'] + es['domains'])

def test_elasticsearch_changelog_transform():
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

def test_elasticsearch_release_kbart_year():
    this_year = datetime.date.today().year
    r = ReleaseEntity(
        title="something",
        release_year=this_year,
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
            "kbart": {
                "lockss": {
                    "year_spans": [[1900, this_year - 2]],
                },
            },
        },
    )
    es = release_to_elasticsearch(r)
    assert es['release_year'] == this_year

    assert es['preservation'] == "none"
    assert es['is_oa'] == True
    assert es['is_longtail_oa'] == False
    assert es['is_preserved'] == False
    assert es['in_web'] == False
    assert es['in_dweb'] == False
    assert es['in_ia'] == False
    assert es['in_ia_sim'] == False
    assert es['in_kbart'] == False
    assert es['in_jstor'] == False

    r.container = ContainerEntity(
        name="dummy journal",
        extra={
            "kbart": {
                "lockss": {
                    "year_spans": [[1900, this_year - 1]],
                },
            },
        },
    )
    es = release_to_elasticsearch(r)
    assert es['release_year'] == this_year

    assert es['preservation'] == "dark"
    assert es['is_oa'] == True
    assert es['is_longtail_oa'] == False
    assert es['is_preserved'] == True
    assert es['in_web'] == False
    assert es['in_dweb'] == False
    assert es['in_ia'] == False
    assert es['in_ia_sim'] == False
    assert es['in_kbart'] == True
    assert es['in_jstor'] == False
