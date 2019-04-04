

import json
import tempfile
import pytest
from fatcat_client.rest import ApiException
from fixtures import *


def test_release_bibtex(app):

    # "realistic" demo entity
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaam')
    assert rv.status_code == 200
    assert b'BibTeX' in rv.data
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaam.bib')
    assert rv.status_code == 200
    assert b'@article{' in rv.data

    # "dummy" demo entity
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai')
    assert rv.status_code == 200
    assert not b'BibTeX' in rv.data
    with pytest.raises(ValueError):
        rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai.bib')

