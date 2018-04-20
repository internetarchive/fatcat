
import json
import pytest
import fatcat
import fatcat.sql
import tempfile
from fatcat.models import *
from fixtures import *


def test_static_routes(rich_app):
    app = rich_app

    for route in ('/health', '/robots.txt', '/', '/about'):
        rv = app.get(route)
        assert rv.status_code == 200

    assert app.get("/static/bogus/route").status_code == 404


