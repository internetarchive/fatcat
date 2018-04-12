
import random
from fatcat import app, db
from fatcat.models import *

def populate_db():
    """
    TODO: doesn't create an edit trail (yet)
    """

    n_elkies = CreatorRevision(
        name="Noam D. Elkies",
        sortname="Elkies, N",
        orcid=None)
    n_elkies_id = CreatorId(revision_id=n_elkies.id)
    pi_work = WorkRevision(
        title="Why is π 2so close to 10?")
    pi_work_id = WorkId(revision_id=pi_work.id)
    pi_release = ReleaseRevision(
        title=pi_work.title,
        creators=[n_elkies_id],
        work_id=pi_work.id)
    pi_release_id = ReleaseId(revision_id=pi_release.id)
    pi_work.primary_release = pi_release.id

    #pi_file = File(
    #    sha1="efee52e46c86691e2b892dbeb212f3b92e92e9d3",
    #    url="http://www.math.harvard.edu/~elkies/Misc/pi10.pdf")
    db.session.add_all([n_elkies, pi_work, pi_work_id, pi_release, pi_release_id])

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
    for i in range(count):
        first = random.choice(first_names)
        last = random.choice(last_names)
        ar = CreatorRevision(
            name="{} {}".format(first, last),
            sortname="{}, {}".format(last, first[0]),
            orcid=None)
        author_revs.append(ar)
        author_ids.append(CreatorId(revision_id=ar.id))

    title_start = ("All about ", "When I grow up I want to be",
        "The final word on", "Infinity: ", "The end of")
    title_ends = ("Humankind", "Bees", "Democracy", "Avocados")
    work_revs = []
    work_ids = []
    release_revs = []
    release_ids = []
    for i in range(count):
        title = "{} {}".format(random.choice(title_start), random.choice(title_ends))
        work = WorkRevision(title=title)
        work_id = WorkId(revision_id=work.id)
        authors = set(random.sample(author_ids, 5))
        release = ReleaseRevision(
            title=work.title,
            creators=list(authors),
            work_id=work.id)
        release_id = ReleaseId(revision_id=release.id)
        work.primary_release = release.id
        authors.add(random.choice(author_ids))
        release2 = ReleaseRevision(
            title=work.title + " (again)",
            creators=list(authors),
            work_id=work.id)
        release_id2 = ReleaseId(revision_id=release2.id)
        work_revs.append(work)
        work_ids.append(work_id)
        release_revs.append(release)
        release_revs.append(release2)
        release_ids.append(release_id)
        release_ids.append(release_id2)

    db.session.add_all(author_revs)
    db.session.add_all(author_ids)
    db.session.add_all(work_revs)
    db.session.add_all(work_ids)
    db.session.add_all(release_revs)
    db.session.add_all(release_ids)

    db.session.commit()
