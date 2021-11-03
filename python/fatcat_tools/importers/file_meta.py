import fatcat_openapi_client

from .common import EntityImporter


class FileMetaImporter(EntityImporter):
    """
    The purpose of this importer is to update file-level metadata for file
    entities that are missing some fields.

    It should *only* update entities, never create (insert) them.

    In particular, during early bootstrapping over 18 million file entities were
    imported which were missing file size, mimetype, md5, and/or sha256.
    """

    def __init__(self, api, require_grobid=True, **kwargs):

        eg_desc = kwargs.pop("editgroup_description", None) or "File metadata updates"
        eg_extra = kwargs.pop("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.FileMetaImporter")
        kwargs["do_updates"] = kwargs.get("do_updates", True)
        super().__init__(api, editgroup_description=eg_desc, editgroup_extra=eg_extra, **kwargs)

    def want(self, row):
        for k in ("sha1hex", "sha256hex", "md5hex", "size_bytes", "mimetype"):
            if not row.get(k):
                self.counts["skip-missing-field"] += 1
                return False
        return True

    def parse_record(self, row):

        # bezerk mode doesn't make sense for this importer
        assert self.bezerk_mode is False

        file_meta = row
        fe = fatcat_openapi_client.FileEntity(
            md5=file_meta["md5hex"],
            sha1=file_meta["sha1hex"],
            sha256=file_meta["sha256hex"],
            size=file_meta["size_bytes"],
            mimetype=file_meta["mimetype"],
        )
        return fe

    def try_update(self, fe):

        # lookup sha1, or create new entity
        existing = None
        try:
            existing = self.api.lookup_file(sha1=fe.sha1)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        if not existing:
            self.counts["skip-no-match"] += 1
            return False

        if existing.md5 and existing.sha256 and existing.size and existing.mimetype:
            self.counts["skip-existing-complete"] += 1
            return False

        existing.md5 = existing.md5 or fe.md5
        existing.sha256 = existing.sha256 or fe.sha256
        existing.size = existing.size or fe.size
        existing.mimetype = existing.mimetype or fe.mimetype

        # generic file entity cleanups
        existing = self.generic_file_cleanups(existing)

        self.api.update_file(self.get_editgroup_id(), existing.ident, existing)
        self.counts["update"] += 1
        return False
