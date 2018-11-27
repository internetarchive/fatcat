
import json
import pytest
from fatcat_tools import *
from fatcat_client import *

from import_crossref import crossref_importer

def test_elasticsearch_convert(crossref_importer):
    with open('tests/files/crossref-works.single.json', 'r') as f:
        # not a single line
        raw = json.loads(f.read())
        (r, c) = crossref_importer.parse_crossref_dict(raw)
    r.state = 'active'
    release_to_elasticsearch(r)

def test_elasticsearch_from_json():
    r = entity_from_json(open('./tests/files/math_universe.json', 'r').read(), ReleaseEntity)
    release_to_elasticsearch(r)
