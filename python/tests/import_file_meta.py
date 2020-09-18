
import json
import pytest

from fatcat_tools.importers import FileMetaImporter, JsonLinePusher
from fatcat_openapi_client import FileEntity
from fixtures import *


@pytest.fixture(scope="function")
def file_meta_importer(api):
    yield FileMetaImporter(api)

def test_file_meta_importer_basic(file_meta_importer):

    # insert two file entities
    api = file_meta_importer.api
    eg = quick_eg(file_meta_importer.api)
    # with full metadata
    f1edit = api.create_file(eg.editgroup_id, FileEntity(
        size=372121,
        md5="e1fd97475c8aa102568f5d70a1bd0c07",
        sha1="0000045687dad717ed6512e395b04ec9c00995b7",
        sha256="51bdc9e40cc175089fcb60b0b188e6cbcdcddb1ff8acbe6b039b8f8fff0afff0",
        mimetype="application/pdf",
    ))
    # partial/stub metadata
    f2edit = api.create_file(eg.editgroup_id, FileEntity(
        sha1="00000376ad49f56145721503f1eb5e6e49e779fd",
        mimetype="application/pdf",
    ))
    api.accept_editgroup(eg.editgroup_id)

    with open('tests/files/example_file_meta.json', 'r') as f:
        counts = JsonLinePusher(file_meta_importer, f).run()

    assert counts['insert'] == 0
    assert counts['exists'] == 0
    assert counts['update'] == 1
    assert counts['skip-no-match'] == 4
    assert counts['skip-missing-field'] == 1
    assert counts['skip-existing-complete'] == 1

    # cleanup file entities
    eg = quick_eg(file_meta_importer.api)
    api.delete_file(eg.editgroup_id, f1edit.ident)
    api.delete_file(eg.editgroup_id, f2edit.ident)
    api.accept_editgroup(eg.editgroup_id)

def test_file_meta_dict_parse(file_meta_importer):
    with open('tests/files/example_file_meta.json', 'r') as f:
        raw = json.loads(f.readline())
        f = file_meta_importer.parse_record(raw)

        assert f.sha1 == "00000088bbc15a03ab89d8da6c356bf25aea9519"
        assert f.md5 == "f4308d58dc8806232c30edc56a896412"
        assert f.sha256 == "593f5a260129bb89ed316c9ddcb7b2f9c2e3da8adf87d29f212b423de32a2c59"
        assert f.mimetype == "application/pdf"
        assert f.size == 354118
