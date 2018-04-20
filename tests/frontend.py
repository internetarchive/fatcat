
import os
import json
import pytest
import fatcat
import fatcat.sql
from fatcat.models import *
import unittest
import tempfile


@pytest.fixture
def app():
    fatcat.app.config['SQLALCHEMY_DATABASE_URI'] = 'sqlite://'
    fatcat.app.testing = True
    fatcat.app.debug = True
    fatcat.db.session.remove()
    fatcat.db.drop_all()
    fatcat.db.create_all()
    fatcat.sql.populate_db()
    return fatcat.app.test_client()


def test_static_routes(app):

    for route in ('/health', '/robots.txt', '/', '/about'):
        rv = app.get(route)
        assert rv.status_code == 200

    assert app.get("/static/bogus/route").status_code == 404


