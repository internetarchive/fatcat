
import json
import random
import hashlib
from fatcat import db
from fatcat.models import *

def populate_db():
    """
    TODO: doesn't create an edit trail (yet)
    """

    n_elkies = CreatorRevision(
        name="Noam D. Elkies",
        sortname="Elkies, N",
        orcid=None)
    n_elkies_id = CreatorIdent(revision=n_elkies)
    pi_work = WorkRevision(
        title="Why is π^2 so close to 10?",
        work_type="journal-article")
    pi_work_id = WorkIdent(revision=pi_work)
    pi_release = ReleaseRevision(
        title=pi_work.title,
        work_ident_id=pi_work.id,
        release_type="journal-article")
    pi_contrib = ReleaseContrib(creator=n_elkies_id)
    pi_release.creators.append(pi_contrib)
    pi_release_id = ReleaseIdent(revision=pi_release)
    pi_work.primary_release = pi_release

    # TODO:
    #pi_file = File(
    #    sha1="efee52e46c86691e2b892dbeb212f3b92e92e9d3",
    #    url="http://www.math.harvard.edu/~elkies/Misc/pi10.pdf")
    db.session.add_all([n_elkies, n_elkies_id, pi_work, pi_work_id, pi_release,
        pi_release_id])

    # TODO:
    #ligo_collab = CreatorRevision(name="LIGO Scientific Collaboration")
    #ligo_paper = ReleaseRevision(
    #    title="Full Band All-sky Search for Periodic Gravitational Waves in the O1 LIGO Data")
    db.session.commit()


def populate_complex_db(count=100):
    """
    TODO: doesn't create an edit trail (yet)
    """

    first_names = ("Sarah", "Robin", "Halko", "Jefferson", "Max", "桃井",
        "Koizumi", "Rex", "Billie", "Tenzin")
    last_names = ("Headroom", "はるこ", "Jun'ichirō", "Wong", "Smith")

    author_revs = []
    author_ids = []
    for _ in range(count):
        first = random.choice(first_names)
        last = random.choice(last_names)
        ar = CreatorRevision(
            name="{} {}".format(first, last),
            sortname="{}, {}".format(last, first[0]),
            orcid=None)
        author_revs.append(ar)
        author_ids.append(CreatorIdent(revision=ar))

    container_revs = []
    container_ids = []
    for _ in range(5):
        cr = ContainerRevision(
            name="The Fake Journal of Stuff",
            #container_id=None,
            publisher="Big Paper",
            sortname="Fake Journal of Stuff",
            issn="1234-5678")
        container_revs.append(cr)
        container_ids.append(ContainerIdent(revision=cr))

    title_start = ("All about ", "When I grow up I want to be",
        "The final word on", "Infinity: ", "The end of")
    title_ends = ("Humankind", "Bees", "Democracy", "Avocados", "«küßî»", "“ЌύБЇ”")
    work_revs = []
    work_ids = []
    release_revs = []
    release_ids = []
    file_revs = []
    file_ids = []
    for _ in range(count):
        title = "{} {}".format(random.choice(title_start), random.choice(title_ends))
        work = WorkRevision(title=title)
        work_id = WorkIdent(revision=work)
        authors = set(random.sample(author_ids, 5))
        release = ReleaseRevision(
            title=work.title,
            creators=[ReleaseContrib(creator=a) for a in list(authors)],
            #work=work,
            container=random.choice(container_ids))
        release_id = ReleaseIdent(revision=release)
        work.primary_release = release
        authors.add(random.choice(author_ids))
        release2 = ReleaseRevision(
            title=work.title + " (again)",
            creators=[ReleaseContrib(creator=a) for a in list(authors)],
            #work=work,
            container=random.choice(container_ids))
        release_id2 = ReleaseIdent(revision=release2)
        work_revs.append(work)
        work_ids.append(work_id)
        release_revs.append(release)
        release_revs.append(release2)
        release_ids.append(release_id)
        release_ids.append(release_id2)

        file_content = str(random.random()) * random.randint(3,100)
        file_sha = hashlib.sha1(file_content.encode('utf-8')).hexdigest()
        file_rev = FileRevision(
            sha1=file_sha,
            size=len(file_content),
            url="http://archive.invalid/{}".format(file_sha),
            releases=[FileRelease(release=release_id), FileRelease(release=release_id2)],
        )
        file_id = FileIdent(revision=file_rev)
        file_revs.append(file_rev)
        file_ids.append(file_id)

    db.session.add_all(author_revs)
    db.session.add_all(author_ids)
    db.session.add_all(work_revs)
    db.session.add_all(work_ids)
    db.session.add_all(release_revs)
    db.session.add_all(release_ids)
    db.session.add_all(container_revs)
    db.session.add_all(container_ids)
    db.session.add_all(file_revs)
    db.session.add_all(file_ids)

    db.session.commit()

def add_crossref(meta):

    title = meta['title'][0]

    # authors
    author_revs = []
    author_ids = []
    for am in meta['author']:
        ar = CreatorRevision(
            name="{} {}".format(am['given'], am['family']),
            sortname="{}, {}".format(am['family'], am['given']),
            orcid=None)
        author_revs.append(ar)
        author_ids.append(CreatorIdent(revision=ar))

    # container
    container = ContainerRevision(
        issn=meta['ISSN'][0],
        name=meta['container-title'][0],
        #container_id=None,
        publisher=meta['publisher'],
        sortname=meta['short-container-title'][0])
    container_id = ContainerIdent(revision=container)

    # work and release
    work = WorkRevision(title=title)
    work_id = WorkIdent(revision=work)
    release = ReleaseRevision(
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
        "rev": work.revision_id,
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
    creators = [c.creator_id for c in WorkContrib.query.filter(WorkContrib.work == work).all()]
    #releases = [r.id for r in ReleaseIdent.query.filter(ReleaseIdent.revision.work_id==work.id).all()]
    releases = []
    hydro.update({
        "work_type": work.revision.work_type,
        "title": work.revision.title,
        "edit_id": work.revision.edit_id,
        "primary": primary,
        "creators": creators,
        "releases": releases,
    })
    return hydro

def hydrate_release(rid):

    wid = int(rid)
    release = ReleaseIdent.query.filter(ReleaseIdent.id==rid).first_or_404()

    return {
        "_type": "release",
        "id": rid,
        "revision": release.revision_id,
        "edit_id": release.revision.edit_id,
        "is_live": release.live,

        "work_id": release.revision.work_ident_id,
        "release_type": release.revision.release_type,
        "title": release.revision.title,
        "creators": [],
        "releases": [],
        "files": [],
        "references": [],
    }
