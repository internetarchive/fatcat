
import copy
import pytest

from fatcat_tools.cleanups import FileCleaner
from fatcat_openapi_client import *
from fixtures import *


@pytest.fixture(scope="function")
def file_cleaner(api):
    yield FileCleaner(api)

def test_url_cleanups(file_cleaner):

    f = FileEntity(
        sha1="027e7ed3ea1a40e92dd2657a1e3c992b5dc45dd2",
        urls=[],
    )

    f.urls = [
        FileUrl(url="https://web.archive.org/web/12345542/something.com/blah.pdf", rel="webarchive"),
        FileUrl(url="https://web.archive.org/web/None/something.com/blah.pdf", rel="webarchive"),
        FileUrl(url="https://archive.org/details/None/something.com/blah.pdf", rel="repository"),
    ]
    f = file_cleaner.clean_entity(f)

    # remove None wayback links
    assert len(f.urls) == 2
    for u in f.urls:
        assert 'web/None' not in u.url

    assert f == file_cleaner.clean_entity(f)
    assert f == file_cleaner.clean_entity(copy.deepcopy(f))

    # rel=repository -> rel=archive for archive.org links
    assert f.urls[1].rel == 'archive'

    # short wayback dates
    f.urls = [
        FileUrl(url="http://web.archive.org/web/20181031120933/https://www.jstage.jst.go.jp/article/jsci1978/1/1/1_1_231/_pdf", rel="webarchive"),
        FileUrl(url="http://web.archive.org/web/2018/https://www.jstage.jst.go.jp/article/jsci1978/1/1/1_1_231/_pdf", rel="webarchive"),
    ]
    f = file_cleaner.clean_entity(f)
    assert len(f.urls) == 1
    assert f.urls[0].url == 'http://web.archive.org/web/20181031120933/https://www.jstage.jst.go.jp/article/jsci1978/1/1/1_1_231/_pdf'

    assert f == file_cleaner.clean_entity(f)
    assert f == file_cleaner.clean_entity(copy.deepcopy(f))

    f.urls = [
        FileUrl(url="http://web.archive.org/web/2018/https://www.jstage.jst.go.jp/article/jsci1978/1/1/1_1_231/_pdf", rel="webarchive"),
    ]
    f = file_cleaner.clean_entity(f)
    assert len(f.urls) == 1
    assert f.urls[0].url == 'http://web.archive.org/web/2018/https://www.jstage.jst.go.jp/article/jsci1978/1/1/1_1_231/_pdf'

    assert f == file_cleaner.clean_entity(f)
    assert f == file_cleaner.clean_entity(copy.deepcopy(f))
