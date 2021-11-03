
import json
import pytest
from fixtures import *


def test_release_bibtex(app, api):

    # "realistic" demo entity
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaam')
    assert rv.status_code == 200
    assert b'BibTeX' in rv.data
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaam.bib')
    assert rv.status_code == 200
    assert b'@article{' in rv.data
    rv = app.get('/release/ccccccccccccccccccccccccca.bib')
    assert rv.status_code == 404
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaam/citeproc?style=bibtex')
    assert rv.status_code == 200
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaam/citeproc?style=csl-json')
    assert rv.status_code == 200
    # could also rv.get_json() here
    json.loads(rv.data.decode('utf-8'))
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaam/citeproc?style=modern-language-association')
    assert rv.status_code == 200
    assert rv.data.decode('utf-8').startswith('Ioannidis, J.. Why Most Published Research Findings Are False')

    # "dummy" demo entity; very minimal metadata
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai')
    assert rv.status_code == 200
    assert b'BibTeX' in rv.data
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai.bib')
    assert rv.status_code == 200
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/citeproc?style=modern-language-association')
    assert rv.status_code == 200
    rv = app.get('/release/aaaaaaaaaaaaarceaaaaaaaaai/citeproc?style=csl-json')
    assert rv.status_code == 200

    # create release which can not have citeproc run on it (no authors)
    eg = quick_eg(api)
    r1 = ReleaseEntity(
        title="some title",
        ext_ids=ReleaseExtIds(),
    )
    r1edit = api.create_release(eg.editgroup_id, r1)
    api.accept_editgroup(eg.editgroup_id)

    rv = app.get('/release/{}'.format(r1edit.ident))
    assert rv.status_code == 200
    assert b'BibTeX' not in rv.data
    with pytest.raises(ValueError):
        rv = app.get('/release/{}.bib'.format(r1edit.ident))

    # create release can have citeproc run on it (no authors)
    eg = quick_eg(api)
    r2 = ReleaseEntity(
        title="some title again",
        contribs=[
            ReleaseContrib(
                given_name="Paul",
                surname="Otlet"),
        ],
        ext_ids=ReleaseExtIds(),
    )
    r2edit = api.create_release(eg.editgroup_id, r2)
    api.accept_editgroup(eg.editgroup_id)

    rv = app.get('/release/{}'.format(r2edit.ident))
    assert rv.status_code == 200
    assert b'BibTeX' in rv.data
    rv = app.get('/release/{}.bib'.format(r2edit.ident))
    assert rv.status_code == 200
