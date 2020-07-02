
from fixtures import *


def test_static_routes(app):
    for route in ('/health.json', '/robots.txt', '/', '/about', '/rfc',
            '/static/fatcat.jpg'):
        rv = app.get(route)
        assert rv.status_code == 200

    assert app.get("/search").status_code == 302
    assert app.get("/static/bogus/route").status_code == 404
