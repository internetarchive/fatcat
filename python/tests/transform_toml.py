import json

from fatcat_openapi_client import ReleaseEntity
from fixtures import api
from import_crossref import crossref_importer

from fatcat_tools.transforms import entity_from_toml, entity_to_toml


def test_basic_toml(crossref_importer):
    with open("tests/files/crossref-works.single.json") as f:
        # not a single line
        raw = json.loads(f.read())
        r = crossref_importer.parse_record(raw)
    r.state = "active"
    toml_str = entity_to_toml(r)
    r2 = entity_from_toml(toml_str, ReleaseEntity)
    assert r == r2

    toml_str = entity_to_toml(r, pop_fields=["ident", "revision", "blah", "extra"])
    r3 = entity_from_toml(toml_str, ReleaseEntity)
    assert r != r3
