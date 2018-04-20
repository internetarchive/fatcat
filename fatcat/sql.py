
import json
import time
import random
import hashlib
from fatcat import db
from fatcat.models import *

def populate_db():
    admin_editor = Editor(id=1, username="admin", is_admin=True)
    db.session.add(admin_editor)
    db.session.commit()

def add_crossref_via_model(meta):

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
        author_ids.append(CreatorIdent(rev=ar))

    # container
    container = ContainerRev(
        issn=meta['ISSN'][0],
        name=meta['container-title'][0],
        #container_id=None,
        publisher=meta['publisher'],
        sortname=meta['short-container-title'][0])
    container_id = ContainerIdent(rev=container)

    # work and release
    work = WorkRev(title=title)
    work_id = WorkIdent(rev=work)
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
    release_id = ReleaseIdent(rev=release)
    work.primary_release = release_id
    extra = json.dumps({
        'crossref': {
            'links': meta.get('link', []),
            'subject': meta['subject'],
            'type': meta['type'],
            'alternative-id': meta.get('alternative-id', []),
        }
    }, indent=None).encode('utf-8')
    extra_json = ExtraJson(json=extra, sha1=hashlib.sha1(extra).hexdigest())
    release.extra_json_id = extra_json.sha1

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

def accept_editgroup(eg):

    # check if already accepted
    # XXX: add a test for this
    assert ChangelogEntry.query.filter(ChangelogEntry.edit_group_id==eg.id).count() == 0

    # start transaction (TODO: explicitly?)

    # for each entity type:
    for cls in (WorkEdit, ReleaseEdit, CreatorEdit, ContainerEdit, FileEdit):
        edits = cls.query.filter(cls.edit_group_id==eg.id).all()
        # for each entity edit->ident:
        for edit in edits:
            # update entity ident state (activate, redirect, delete)
            edit.ident.is_live = True
            edit.ident.rev_id = edit.rev_id
            edit.ident.redirect_id = edit.redirect_id
            db.session.add(edit.ident)

    # append log/changelog row
    cle = ChangelogEntry(
        edit_group_id=eg.id,
        # TODO: is this UTC?
        timestamp=int(time.time()))
    db.session.add(cle)

    # update edit group state
    db.session.add(eg)

    db.session.commit()
