
import os
import json
import base64
import pytest

from fatcat_tools.importers import GrobidMetadataImporter, LinePusher
from fixtures import *

"""
WARNING: these tests are currently very fragile because they have database
side-effects. Should probably be disabled or re-written.
"""

@pytest.fixture(scope="function")
def grobid_metadata_importer(api):
    yield GrobidMetadataImporter(api)


def test_grobid_metadata_parse(grobid_metadata_importer):
    with open('tests/files/example_grobid_metadata_lines.tsv', 'r') as f:
        raw = json.loads(f.readline().split('\t')[4])
        re = grobid_metadata_importer.parse_grobid_json(raw)
        assert re
        assert re.title == "PEMBELAJARAN FISIKA DENGAN PENDEKATAN KETERAMPILAN PROSES MELALUI METODE DEMONSTRASI MENGGUNAKAN MEDIA RIIL DAN MEDIA VIRTUIL DITINJAU DARI MOTIVASI DAN GAYA BERFIKIR SISWA"
        assert len(re.contribs) == 5
        print(re.contribs)
        assert re.contribs[0].raw_name == "Wahyu Ary"
        assert re.contribs[0].given_name == "Wahyu"
        assert re.contribs[0].surname == "Ary"
        assert re.publisher is None
        if re.extra:
            assert re.extra.get('container_name') is None
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
        assert fe.md5 is None
        assert fe.mimetype == "application/pdf"
        assert fe.size == 142710
        assert fe.urls[1].url.startswith("http://via.library.depaul.edu")
        assert fe.urls[1].rel == "web"
        assert fe.urls[0].url.startswith("https://web.archive.org/")
        assert fe.urls[0].rel == "webarchive"
        assert len(fe.release_ids) == 0

def test_grobid_metadata_importer(grobid_metadata_importer):
    last_index = grobid_metadata_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_grobid_metadata_lines.tsv', 'r') as f:
        grobid_metadata_importer.bezerk_mode = True
        counts = LinePusher(grobid_metadata_importer, f).run()
    assert counts['insert'] == 10
    assert counts['inserted.release'] == 10
    assert counts['exists'] == 0
    assert counts['skip'] == 0

    # fetch most recent editgroup
    change = grobid_metadata_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "grobid" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.GrobidMetadataImporter" in eg.extra['agent']

    with open('tests/files/example_grobid_metadata_lines.tsv', 'r') as f:
        grobid_metadata_importer.reset()
        grobid_metadata_importer.bezerk_mode = False
        counts = LinePusher(grobid_metadata_importer, f).run()
    assert counts['insert'] == 0
    assert counts['inserted.release'] == 0
    assert counts['exists'] == 10
    assert counts['skip'] == 0
