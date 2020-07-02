
import json

from fatcat_tools.transforms import release_ingest_request
from fatcat_openapi_client import *
from fixtures import api
from import_crossref import crossref_importer


def test_basic_ingest_release(crossref_importer):
    with open('tests/files/crossref-works.single.json', 'r') as f:
        # not a single line
        raw = json.loads(f.read())
        r = crossref_importer.parse_record(raw)
    r.state = 'active'
    req = release_ingest_request(r)
    assert req is not None

def test_rich_ingest_release():
    r = ReleaseEntity(
        title="something",
        ident="iznnn644szdwva7khyxqzc5555",
        release_year=1234,
        license_slug="CC-BY-NC",
        ext_ids=ReleaseExtIds(doi="10.123/456"),
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
    ir = release_ingest_request(r)
    assert ir is not None
    assert ir['base_url'] == 'https://doi.org/10.123/456'
    assert ir['ext_ids']['doi'] == '10.123/456'
    assert ir['ext_ids'].get('pmcid') is None
