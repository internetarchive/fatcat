
import os
import time
import json
import signal
import pytest
import fatcat_web


@pytest.fixture
def full_app():
    fatcat_web.app.testing = True
    fatcat_web.app.debug = False
    return fatcat_web.app

@pytest.fixture
def app(full_app):
    return full_app.test_client()


## Helpers ##################################################################
# TODO: what are these even here for?

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
