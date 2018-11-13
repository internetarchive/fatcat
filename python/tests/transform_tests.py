
import json
import pytest
from fatcat_tools.crossref_importer import FatcatCrossrefImporter
from fatcat_tools.transforms import *

from crossref import crossref_importer

def test_elastic_convert(crossref_importer):
    with open('tests/files/crossref-works.single.json', 'r') as f:
        # not a single line
        raw = json.loads(f.read())
        (r, c) = crossref_importer.parse_crossref_dict(raw)
    r.state = 'active'
    release_elastic_dict(r)
