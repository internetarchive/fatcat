
import json
import pytest
from fatcat_tools.importers import ShadowLibraryImporter, JsonLinePusher
from fixtures import api


@pytest.fixture(scope="function")
def shadow_importer(api):
    yield ShadowLibraryImporter(api)

# TODO: use API to check that entities actually created...
def test_shadow_importer_basic(shadow_importer):
    with open('tests/files/example_shadow.json', 'r') as f:
        JsonLinePusher(shadow_importer, f).run()

def test_shadow_importer(shadow_importer):
    last_index = shadow_importer.api.get_changelog(limit=1)[0].index
    with open('tests/files/example_shadow.json', 'r') as f:
        shadow_importer.bezerk_mode = True
        counts = JsonLinePusher(shadow_importer, f).run()
    assert counts['insert'] == 2
    assert counts['exists'] == 0
    assert counts['skip'] == 8

    # fetch most recent editgroup
    change = shadow_importer.api.get_changelog_entry(index=last_index+1)
    eg = change.editgroup
    assert eg.description
    assert "shadow library" in eg.description.lower()
    assert eg.extra['git_rev']
    assert "fatcat_tools.ShadowLibraryImporter" in eg.extra['agent']

    # re-insert; should skip
    with open('tests/files/example_shadow.json', 'r') as f:
        shadow_importer.reset()
        shadow_importer.bezerk_mode = False
        counts = JsonLinePusher(shadow_importer, f).run()
    assert counts['insert'] == 0
    assert counts['exists'] == 2
    assert counts['skip'] == 8

def test_shadow_dict_parse(shadow_importer):
    with open('tests/files/example_shadow.json', 'r') as f:
        raw = json.loads(f.readline())
        f = shadow_importer.parse_record(raw)

        assert f.sha1 == "0000002922264275f11cca7b1c3fb662070d0dd7"
        assert f.md5 == "debd8db178fa08a7a0aaec6e42832a8e"
        assert f.sha256 == "b4728210cc0f70d8a8f8c39bd97fcbbab3eaca4309ac4bdfbce5df3b66c82f79"
        assert f.mimetype == "application/pdf"
        assert f.size == 206121
        assert len(f.urls) == 2
        for u in f.urls:
            if u.rel == "publisher":
                assert u.url.startswith("https://link.springer.com/content/pdf/10.1007%2Fs11626-008-9119-8.pdf")
            if u.rel == "webarchive":
                assert u.url.startswith("https://web.archive.org/")
                assert "20180729135948" in u.url
        assert len(f.release_ids) == 1

