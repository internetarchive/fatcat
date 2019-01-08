
import os
import json
import base64
import pytest
from fatcat_tools.importers import GrobidMetadataImporter
from fixtures import api

"""
WARNING: these tests are currently very fragile because they have database
side-effects. Should probably be disabled or re-written.
"""

@pytest.fixture(scope="function")
def grobid_metadata_importer(api):
    yield GrobidMetadataImporter(api)

# TODO: use API to check that entities actually created...
#def test_grobid_metadata_importer_batch(grobid_metadata_importer):
#    with open('tests/files/example_grobid_metadata_lines.tsv', 'r') as f:
#        grobid_metadata_importer.process_batch(f)

def test_grobid_metadata_parse(grobid_metadata_importer):
    with open('tests/files/example_grobid_metadata_lines.tsv', 'r') as f:
        raw = json.loads(f.readline().split('\t')[4])
        re = grobid_metadata_importer.parse_grobid_json(raw)
        assert re
        assert re.title == "PEMBELAJARAN FISIKA DENGAN PENDEKATAN KETERAMPILAN PROSES MELALUI METODE DEMONSTRASI MENGGUNAKAN MEDIA RIIL DAN MEDIA VIRTUIL DITINJAU DARI MOTIVASI DAN GAYA BERFIKIR SISWA"
        assert len(re.contribs) == 5
        print(re.contribs)
        assert re.contribs[0].raw_name == "Wahyu Ary"
        assert re.publisher == None
        assert re.extra.get('container_name') == None
        assert len(re.refs) == 27

def test_file_metadata_parse(grobid_metadata_importer):
    with open('tests/files/example_grobid_metadata_lines.tsv', 'r') as f:
        f.readline()
        raw = f.readline().split('\t')
        # randomize sha1 so tests are repeatable
        random_sha1 = "sha1:{}".format(base64.b32encode(os.urandom(20)).decode('utf-8').upper())
        fe = grobid_metadata_importer.parse_file_metadata(
            random_sha1, json.loads(raw[1]), raw[2], int(raw[3]))
        assert fe
        #assert fe.sha1 == "d4a841744719518bf8bdd5d91576ccedc55efbb5" # "sha1:2SUEC5CHDFIYX6F52XMRK5WM5XCV565V"
        assert fe.md5 == None
        assert fe.mimetype == "application/pdf"
        assert fe.size == 142710
        assert fe.urls[1].url.startswith("http://via.library.depaul.edu")
        assert fe.urls[1].rel == "web"
        assert fe.urls[0].url.startswith("https://web.archive.org/")
        assert fe.urls[0].rel == "webarchive"
        assert len(fe.release_ids) == 0

def test_grobid_metadata_importer(grobid_metadata_importer):
    with open('tests/files/example_grobid_metadata_lines.tsv', 'r') as f:
        grobid_metadata_importer.process_source(f)

    # fetch most recent editgroup
    changes = grobid_metadata_importer.api.get_changelog(limit=1)
    eg = changes[0].editgroup
    assert eg.description
    assert "grobid" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.GrobidMetadataImporter" in eg.extra['agent']
