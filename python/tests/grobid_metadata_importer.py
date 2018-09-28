
import json
import pytest
from fatcat.grobid_metadata_importer import FatcatGrobidMetadataImporter


@pytest.fixture(scope="function")
def grobid_metadata_importer():
    yield FatcatGrobidMetadataImporter("http://localhost:9411/v0")

# TODO: use API to check that entities actually created...
#def test_grobid_metadata_importer_batch(grobid_metadata_importer):
#    with open('tests/files/example_grobid_metadata_lines.tsv', 'r') as f:
#        grobid_metadata_importer.process_batch(f)

def test_grobid_metadata_importer(grobid_metadata_importer):
    with open('tests/files/example_grobid_metadata_lines.tsv', 'r') as f:
        grobid_metadata_importer.process_source(f)

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
        raw = f.readline().split('\t')
        fe = grobid_metadata_importer.parse_file_metadata(
            raw[0], json.loads(raw[1]), raw[2], int(raw[3]))
        assert fe
        assert fe.sha1 == "38d725127246895368e4d9f950e377b4f21b6d75" # "sha1:HDLSKETSI2EVG2HE3H4VBY3XWTZBW3LV"
        assert fe.md5 == None
        assert fe.mimetype == "application/pdf"
        assert fe.size == 260608
        assert fe.urls[1].url.startswith("http://e-journal.hamzanwadi.ac.id")
        assert fe.urls[1].rel == "web"
        assert fe.urls[0].url.startswith("https://web.archive.org/")
        assert fe.urls[0].rel == "webarchive"
        assert len(fe.releases) == 0
