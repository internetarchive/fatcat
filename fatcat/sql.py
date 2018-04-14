
import json
import random
import hashlib
from fatcat import db
from fatcat.models import *

def populate_db():
    pass

def add_crossref(meta):

    title = meta['title'][0]

    # authors
    author_revs = []
    author_ids = []
    for am in meta['author']:
        ar = CreatorRev(
            name="{} {}".format(am['given'], am['family']),
            sortname="{}, {}".format(am['family'], am['given']),
            orcid=None)
        author_revs.append(ar)
        author_ids.append(CreatorIdent(revision=ar))

    # container
    container = ContainerRev(
        issn=meta['ISSN'][0],
        name=meta['container-title'][0],
        #container_id=None,
        publisher=meta['publisher'],
        sortname=meta['short-container-title'][0])
    container_id = ContainerIdent(revision=container)

    # work and release
    work = WorkRev(title=title)
    work_id = WorkIdent(revision=work)
    release = ReleaseRev(
        title=title,
        creators=[ReleaseContrib(creator=a) for a in author_ids],
        #work=work,
        container=container_id,
        release_type=meta['type'],
        doi=meta['DOI'],
        date=meta['created']['date-time'],
        license=meta.get('license', [dict(URL=None)])[0]['URL'] or None,
        issue=meta.get('issue', None),
        volume=meta.get('volume', None),
        pages=meta.get('page', None))
    release_id = ReleaseIdent(revision=release)
    work.primary_release = release
    extra = json.dumps({
        'crossref': {
            'links': meta.get('link', []),
            'subject': meta['subject'],
            'type': meta['type'],
            'alternative-id': meta.get('alternative-id', []),
        }
    }, indent=None).encode('utf-8')
    extra_json = ExtraJson(json=extra, sha1=hashlib.sha1(extra).hexdigest())
    release.extra_json = extra_json.sha1

    # references
    for i, rm in enumerate(meta.get('reference', [])):
        ref = ReleaseRef(
            release_rev=release,
            doi=rm.get("DOI", None),
            index=i+1,
            # TODO: how to generate a proper stub here from k/v metadata?
            stub="| ".join(rm.values()))
        release.refs.append(ref)

    db.session.add_all([work, work_id, release, release_id, container,
        container_id, extra_json])
    db.session.add_all(author_revs)
    db.session.add_all(author_ids)
    db.session.commit()

def hydrate_work(wid):

    wid = int(wid)
    work = WorkIdent.query.filter(WorkIdent.id==wid).first_or_404()
    hydro = {
        "_type": "work",
        "id": wid,
        "rev": work.rev_id,
        "is_live": work.live,
        "redirect_id": work.redirect_id,
    }
    if not work.revision:
        # TODO: look up edit id here from changelog?
        hydro["edit_id"] = None
        return hydro

    primary = None
    if work.revision.primary_release_id:
        primary = hydrate_release(work.revision.primary_release_id)
    #releases = [r.id for r in ReleaseIdent.query.filter(ReleaseIdent.revision.work_id==work.id).all()]
    releases = []
    hydro.update({
        "work_type": work.revision.work_type,
        "title": work.revision.title,
        "primary": primary,
        "releases": releases,
    })
    return hydro

def hydrate_release(rid):

    wid = int(rid)
    release = ReleaseIdent.query.filter(ReleaseIdent.id==rid).first_or_404()

    return {
        "_type": "release",
        "id": rid,
        "revision": release.rev_id,
        #"edit_id": release.revision.edit_id,
        "is_live": release.live,

        "work_id": release.revision.work_ident_id,
        "release_type": release.revision.release_type,
        "title": release.revision.title,
        "creators": [],
        "releases": [],
        "files": [],
        "references": [],
    }
