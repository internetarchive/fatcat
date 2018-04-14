
import random
import hashlib
from fatcat import db
from fatcat.models import *

def insert_example_works():
    """
    TODO: doesn't create an edit trail (yet)
    """

    n_elkies = CreatorRev(
        name="Noam D. Elkies",
        sortname="Elkies, N",
        orcid=None)
    n_elkies_id = CreatorIdent(revision=n_elkies)
    pi_work = WorkRev(
        title="Why is π^2 so close to 10?",
        work_type="journal-article")
    pi_work_id = WorkIdent(revision=pi_work)
    pi_release = ReleaseRev(
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
    #ligo_collab = CreatorRev(name="LIGO Scientific Collaboration")
    #ligo_paper = ReleaseRev(
    #    title="Full Band All-sky Search for Periodic Gravitational Waves in the O1 LIGO Data")
    db.session.commit()


def insert_random_works(count=100):
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
        ar = CreatorRev(
            name="{} {}".format(first, last),
            sortname="{}, {}".format(last, first[0]),
            orcid=None)
        author_revs.append(ar)
        author_ids.append(CreatorIdent(revision=ar))

    container_revs = []
    container_ids = []
    for _ in range(5):
        cr = ContainerRev(
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
        work = WorkRev(title=title)
        work_id = WorkIdent(revision=work)
        authors = set(random.sample(author_ids, 5))
        release = ReleaseRev(
            title=work.title,
            creators=[ReleaseContrib(creator=a) for a in list(authors)],
            #work=work,
            container=random.choice(container_ids))
        release_id = ReleaseIdent(revision=release)
        work.primary_release = release
        authors.add(random.choice(author_ids))
        release2 = ReleaseRev(
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
        file_rev = FileRev(
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
