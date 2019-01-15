
import os
import time
import json
import signal
import pytest
from dotenv import load_dotenv
import fatcat_web
import fatcat_client


@pytest.fixture
def full_app():
    load_dotenv(dotenv_path="./env.example")
    fatcat_web.app.testing = True
    fatcat_web.app.debug = False
    return fatcat_web.app

@pytest.fixture
def app(full_app):
    return full_app.test_client()

@pytest.fixture
def api():
    load_dotenv(dotenv_path="./env.example")
    conf = fatcat_client.Configuration()
    conf.host = "http://localhost:9411/v0"
    conf.api_key["Authorization"] = os.getenv("FATCAT_API_AUTH_TOKEN")
    conf.api_key_prefix["Authorization"] = "Bearer"
    api_client = fatcat_client.DefaultApi(fatcat_client.ApiClient(conf))
    api_client.editor_id = "aaaaaaaaaaaabkvkaaaaaaaaae"
    return api_client

def test_get_changelog_entry(api):
    """Check that fixture is working"""
    cl = api.get_changelog_entry(1)
    assert cl

## Helpers ##################################################################

def quick_eg(api_inst):
    eg = api_inst.create_editgroup(fatcat_client.Editgroup())
    return eg

