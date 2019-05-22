
import json
import pytest
from fatcat_tools import *
from fatcat_client import *
from fixtures import api
from import_journal_metadata import journal_metadata_importer

from import_crossref import crossref_importer

def test_basic_elasticsearch_convert(crossref_importer):
    with open('tests/files/crossref-works.single.json', 'r') as f:
        # not a single line
        raw = json.loads(f.read())
        r = crossref_importer.parse_record(raw)
    r.state = 'active'
    release_to_elasticsearch(r)

def test_rich_elasticsearch_convert(crossref_importer):
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

def test_elasticsearch_from_json():
    r = entity_from_json(open('./tests/files/math_universe.json', 'r').read(), ReleaseEntity)
    release_to_elasticsearch(r)

def test_elasticsearch_container_convert(journal_metadata_importer):
    with open('tests/files/journal_metadata.sample.json', 'r') as f:
        raw = json.loads(f.readline())
        c = journal_metadata_importer.parse_record(raw)
    c.state = 'active'
    es = container_to_elasticsearch(c)
    assert es['publisher'] == c.publisher
