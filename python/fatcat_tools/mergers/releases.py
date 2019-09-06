
from .common import EntityMerger
from fatcat_api_client.models import ReleaseEntity, WorkEntity


class ReleaseMerger(EntityMerger):
    """
    Hard merges a set of release entities, redirecting all entities to a single
    primary release.

    Will also redirect works (if appropriate), and re-point {files, filesets,
    webcaptures} to the new merged release.
    """

    def __init__(self, api, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Automated merge of release entities")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.ReleaseMerger')
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

    def try_merge(self, idents, primary=None):

        updated_entities = 0
        releases = dict()
        eg_id = self.get_editgroup_id()

        self.dry_run_mode

        for ident in idents:
            releases[ident] = api.get_release(ident, expand="files,filesets,webcaptures")

        # select the primary (if not already set)
        if not primary:
            primary = releases.keys()[0]

        primary_work_id = releases[primary].work_id
        updated_work_ids = []
        redirected_release_ids = []

        # execute all the release redirects
        for release in releases.values():
            if release.ident == primary:
                continue

            # file redirects
            for e in release.files:
                e.release_ids.remove(release.ident)
                if not primary in e.release_ids:
                    e.release_ids.append(primary)
                if not self.dry_run_mode:
                    api.update_file(eg_id, e.ident, e)
                updated_entities += 1
                self.counts['updated-files'] += 1

            # fileset redirects
            for e in release.filesets:
                e.release_ids.remove(release.ident)
                if not primary in e.release_ids:
                    e.release_ids.append(primary)
                if not self.dry_run_mode:
                    api.update_fileset(eg_id, e.ident, e)
                updated_entities += 1
                self.counts['updated-filesets'] += 1

            # webcapture redirects
            for e in release.webcaptures:
                e.release_ids.remove(release.ident)
                if not primary in e.release_ids:
                    e.release_ids.append(primary)
                if not self.dry_run_mode:
                    api.update_webcapture(eg_id, e.ident, e)
                updated_entities += 1
                self.counts['updated-webcaptures'] += 1

            # release redirect itself
            updated_work_ids.append(release.work_id)
            redirected_release_ids.append(release.ident)
            if not self.dry_run_mode:
                api.update_release(eg_id, release.ident, ReleaseEntity(redirect=primary))
            updated_entities += 1
            self.counts['updated-releases']


        # lastly, clean up any merged work entities
        redirected_release_ids = set(redirected_release_ids)
        updated_work_ids = list(set(updated_work_ids))
        assert primary_work_id not in updated_work_ids
        for work_id in updated_work_ids:
            work_releases = api.get_work_releases(work_id)
            rids = set([r.ident for r in work_releases])
            if rids.issubset(redirected_release_ids):
                # all the releases for this work were updated/merged; we should
                # redirect to primary work_id
                # also check we don't have any in-flight edit conflicts
                assert not work_id in self._idents_inflight
                self._idents_inflight.append(work_id)
                if not self.dry_run_mode:
                    api.update_work(eg_id, work_id, WorkEntity(redirect=primary_work_id))
                updated_entities += 1
                self.counts['updated-works'] += 1

        return updated_entities

class ReleaseGrouper(EntityMerger):
    pass
