
import pytest
from dotenv import load_dotenv
import elasticsearch

import fatcat_web
import fatcat_openapi_client
from fatcat_openapi_client import *
from fatcat_tools import authenticated_api


ES_CONTAINER_STATS_RESP = {
    'timed_out': False,
    'aggregations': {
        'container_stats': {'buckets': {
            'is_preserved': {'doc_count': 461939},
            'in_kbart': {'doc_count': 461939},
            'in_web': {'doc_count': 2797},
        }},
        'preservation': {
            'buckets': [
            {'key': 'bright', 'doc_count': 444},
            {'key': 'dark', 'doc_count': 111},
            ],
            'sum_other_doc_count': 0,
        },
        'release_type': {
            'buckets': [
            {'key': 'article-journal', 'doc_count': 456},
            {'key': 'book', 'doc_count': 123},
            ],
            'sum_other_doc_count': 0,
        },
    },
    'hits': {'total': 461939, 'hits': [], 'max_score': 0.0},
    '_shards': {'successful': 5, 'total': 5, 'skipped': 0, 'failed': 0},
    'took': 50
}

# TODO: this should not be empty
ES_CONTAINER_RANDOM_RESP = {
    'timed_out': False,
    'hits': {'total': 461939, 'hits': [], 'max_score': 0.0},
    '_shards': {'successful': 5, 'total': 5, 'skipped': 0, 'failed': 0},
    'took': 50
}

ES_RELEASE_EMPTY_RESP = {
    'timed_out': False,
    'hits': {'total': 0, 'hits': [], 'max_score': 0.0},
    '_shards': {'successful': 5, 'total': 5, 'skipped': 0, 'failed': 0},
    'took': 50
}

@pytest.fixture
def full_app(mocker):
    load_dotenv(dotenv_path="./example.env")
    fatcat_web.app.testing = True
    fatcat_web.app.debug = False
    fatcat_web.app.config['WTF_CSRF_ENABLED'] = False

    # mock out ES client requests, so they at least fail fast
    fatcat_web.app.es_client = elasticsearch.Elasticsearch("mockbackend")
    mocker.patch('elasticsearch.connection.Urllib3HttpConnection.perform_request')
    return fatcat_web.app

@pytest.fixture
def app(full_app):
    return full_app.test_client()

@pytest.fixture
def app_admin(app):
    ADMIN_DEV_TOKEN = "AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wNC0wNFQyMzozMjo0NloAAAYgrN3jjy0mgEqIydTFfsOLYSS55dz6Fh2d1CGMNQFLwcQ="
    rv = app.post('/auth/token_login',
        data=dict(token=ADMIN_DEV_TOKEN),
        follow_redirects=True)
    assert rv.status_code == 200
    return app

@pytest.fixture
def api():
    load_dotenv(dotenv_path="./example.env")
    api_client = authenticated_api("http://localhost:9411/v0")
    api_client.editor_id = "aaaaaaaaaaaabkvkaaaaaaaaae"
    return api_client

@pytest.fixture
def api_dummy_entities(api):
    """
    This is a sort of bleh fixture. Should probably create an actual
    object/class with create/accept/clean-up methods?
    """

    c1 = CreatorEntity(display_name="test creator deletion")
    j1 = ContainerEntity(name="test journal deletion")
    r1 = ReleaseEntity(title="test release creator deletion", ext_ids=ReleaseExtIds())
    f1 = FileEntity()
    fs1 = FilesetEntity()
    wc1 = WebcaptureEntity(
        timestamp="2000-01-01T12:34:56Z",
        original_url="http://example.com/",
    )

    eg = quick_eg(api)
    c1 = api.get_creator(api.create_creator(eg.editgroup_id, c1).ident)
    j1 = api.get_container(api.create_container(eg.editgroup_id, j1).ident)
    r1 = api.get_release(api.create_release(eg.editgroup_id, r1).ident)
    w1 = api.get_work(r1.work_id)
    f1 = api.get_file(api.create_file(eg.editgroup_id, f1).ident)
    fs1 = api.get_fileset(api.create_fileset(eg.editgroup_id, fs1).ident)
    wc1 = api.get_webcapture(api.create_webcapture(eg.editgroup_id, wc1).ident)

    return {
        "api": api,
        "editgroup": eg,
        "creator": c1,
        "container": j1,
        "file": f1,
        "fileset": fs1,
        "webcapture": wc1,
        "release": r1,
        "work": w1,
    }

def test_get_changelog_entry(api):
    """Check that fixture is working"""
    cl = api.get_changelog_entry(1)
    assert cl

## Helpers ##################################################################

def quick_eg(api_inst):
    eg = api_inst.create_editgroup(fatcat_openapi_client.Editgroup())
    return eg
