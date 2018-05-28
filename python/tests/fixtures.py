
import os
import time
import json
import signal
import pytest
import fatcat


@pytest.fixture
def full_app():
    fatcat.app.testing = True
    fatcat.app.debug = False
    return fatcat.app

@pytest.fixture
def app(full_app):
    return full_app.test_client()


@pytest.fixture(scope="function")
def api_client(full_app):

    pid = os.fork()
    if pid == 0:
        full_app.testing = False
        full_app.run(host="localhost", port=8444, debug=False)
        os._exit(0)

    time.sleep(0.2)
    yield fatcat.api_client.FatCatApiClient("http://localhost:8444")
    os.kill(pid, signal.SIGKILL)


## Helpers ##################################################################

def check_entity_fields(e):
    for key in ('rev', 'is_live', 'redirect_id'):
        assert key in e
    for key in ('id',):
        assert e[key] is not None

def check_release(e):
    for key in ('work', 'release_type'):
        assert key in e
    for key in ('title', ):
        assert e[key] is not None
    for key in ('refs', 'creators'):
        assert type(e[key]) == list

def check_creator(e):
    for key in ('name',):
        assert e[key] is not None

def check_container(e):
    for key in ('name',):
        assert e[key] is not None

def check_file(e):
    for key in ('size', 'sha1'):
        assert e[key] is not None
