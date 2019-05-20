
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
    rv = app.get('/release/ccccccccccccccccccccccccca.bib')
    assert rv.status_code == 404
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaam/citeproc?style=csl-json')
    assert rv.status_code == 200
    # could also rv.get_json() here
    json.loads(rv.data.decode('utf-8'))
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaam/citeproc?style=modern-language-association')
    assert rv.status_code == 200
    assert rv.data.decode('utf-8').startswith('Ioannidis, John. “Why Most Published Research Findings Are False”. 2.8 (2005)')

    # "dummy" demo entity
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai')
    assert rv.status_code == 200
    assert not b'BibTeX' in rv.data
    with pytest.raises(ValueError):
        rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai.bib')

