
import fatcat_openapi_client

from fatcat_tools import entity_from_dict
from .common import EntityImporter


class FilesetImporter(EntityImporter):
    """
    General purpose importer for fileset entities. Simply fileset schema JSON
    and inserts.

    By default requires release_ids to be non-empty, and will check each
    release_id to see if a fileset is already associated; if so, skips the
    import. This behavior may change in the future, and can be disabled.

    Currently only creates (insert), no updates.
    """

    def __init__(self, api, **kwargs):

        eg_desc = kwargs.pop('editgroup_description', None) or "Generic Fileset entity import"
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.FilesetImporter')
        kwargs['do_updates'] = bool(kwargs.get("do_updates", False))
        self.skip_release_fileset_check = bool(kwargs.get("skip_release_fileset_check", False))
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

        # bezerk mode doesn't make sense for this importer
        assert self.bezerk_mode is False

    def want(self, row):
        if not row.get('release_ids'):
            self.counts['skip-no-release-ids'] += 1
            return False
        if not row.get('urls'):
            self.counts['skip-no-urls'] += 1
            return False
        if not row.get('manifest'):
            self.counts['skip-no-files'] += 1
            return False

        for f in row.get('manifest'):
            for k in ('sha1', 'md5'):
                if not f.get(k):
                    self.counts['skip-missing-file-field'] += 1
                    return False
        return True

    def parse_record(self, row):

        fse = entity_from_dict(
            row,
            fatcat_openapi_client.FilesetEntity,
            api_client=self.api.api_client,
        )
        fse = self.generic_fileset_cleanups(fse)
        return fse

    def try_update(self, fse):

        if not self.skip_release_fileset_check:
            for release_id in fse.release_ids:
                # don't catch 404, that would be an error
                release = self.api.get_release(release_id, expand='filesets', hide='abstracts,refs')
                assert release.state == 'active'
                if release.filesets:
                    self.counts['exists'] += 1
                    self.counts['exists-via-release-filesets'] += 1
                    return False

        # do the insert
        return True

    def insert_batch(self, batch):
        self.api.create_fileset_auto_batch(fatcat_openapi_client.FilesetAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))
