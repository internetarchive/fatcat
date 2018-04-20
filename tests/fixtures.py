
import pytest
import fatcat
import fatcat.sql
from fatcat.models import *


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

@pytest.fixture
def rich_app(app):
    assert app.post('/v0/work',
        data=json.dumps(dict(
            title="dummy",
            work_type="thing",
            extra=dict(a=1, b="zing"))),
        headers={"content-type": "application/json"}
        ).status_code == 200
    return app


## Helpers ##################################################################

def check_entity_fields(e):
    for key in ('rev', 'is_live', 'redirect_id'):
        assert key in e
    for key in ('id',):
        assert e[key] is not None

def check_release(e):
    for key in ('work', 'release_type'):
        assert key in e
    for key in ('title'):
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
