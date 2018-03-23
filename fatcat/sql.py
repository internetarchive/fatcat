
from fatcat import app, db
from fatcat.models import *

def populate_db():

    # XXX: doesn't create an edit trail (yet)
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

