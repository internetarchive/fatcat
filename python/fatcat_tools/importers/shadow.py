
import sys
import json
import sqlite3
import itertools
import fatcat_openapi_client

from fatcat_tools.normal import *
from .common import EntityImporter, make_rel_url, SANE_MAX_RELEASES, SANE_MAX_URLS


class ShadowLibraryImporter(EntityImporter):
    """
    Importer for shadow library files (matched to releases)

    Input format is JSON with keys:
    - shadow
        - shadow_corpus (string slug)
        - shadow_id (string)
        - doi
        - pmid
        - isbn13
    - file_meta
        - sha1hex
        - sha256hex
        - md5hex
        - size_bytes
        - mimetype
    - cdx (may be null)
        - url
        - datetime
    """

    def __init__(self, api, **kwargs):

        eg_desc = kwargs.pop('editgroup_description', None) or "Import of 'Shadow Library' file/release matches"
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.ShadowLibraryImporter')
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)
        self.default_link_rel = kwargs.get("default_link_rel", "web")

    def want(self, raw_record):
        """
        Only want to import records with complete file-level metadata
        """
        fm = raw_record['file_meta']
        if not (fm['mimetype'] and fm['md5hex'] and fm['sha256hex'] and fm['size_bytes']):
            self.counts['skip-file-meta-incomplete'] += 1
            return False
        if fm['mimetype'] != 'application/pdf':
            self.counts['skip-not-pdf'] += 1
            return False
        return True

    def parse_record(self, obj):
        """
        We do the release lookup in this method. Try DOI, then PMID, last ISBN13.
        """

        shadow_corpus = obj['shadow']['shadow_corpus']
        assert shadow_corpus == shadow_corpus.strip().lower()
        doi = clean_doi(obj['shadow'].get('doi'))
        pmid = clean_pmid(obj['shadow'].get('pmid'))
        isbn13 = clean_isbn13(obj['shadow'].get('isbn13'))
        shadow_id = obj['shadow'].get('shadow_id').strip()
        assert shadow_id

        extra = { '{}_id'.format(shadow_corpus): shadow_id }
        for (ext_type, ext_id) in [('doi', doi), ('pmid', pmid), ('isbn13', isbn13)]:
            if not ext_id:
                continue
            extra['{}_{}'.format(shadow_corpus, ext_type)] = ext_id

        # lookup release via several idents
        re = None
        for (ext_type, ext_id) in [('doi', doi), ('pmid', pmid), ('isbn13', isbn13)]:
            if not ext_id:
                continue
            try:
                re = self.api.lookup_release(**{ext_type: ext_id})
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status not in (404, 400):
                    raise err
                re = None
            if re:
                break

        if not re:
            self.counts['skip-release-not-found'] += 1
            return None

        release_ids = [re.ident,]

        # parse single CDX into URLs (if exists)
        urls = []
        if obj.get('cdx'):
            url = make_rel_url(obj['cdx']['url'], default_link_rel=self.default_link_rel)
            if url != None:
                urls.append(url)
            wayback = "https://web.archive.org/web/{}/{}".format(
                obj['cdx']['datetime'],
                obj['cdx']['url'])
            urls.append(("webarchive", wayback))
        urls = [fatcat_openapi_client.FileUrl(rel=rel, url=url) for (rel, url) in urls]

        fe = fatcat_openapi_client.FileEntity(
            md5=obj['file_meta']['md5hex'],
            sha1=obj['file_meta']['sha1hex'],
            sha256=obj['file_meta']['sha256hex'],
            size=int(obj['file_meta']['size_bytes']),
            mimetype=obj['file_meta']['mimetype'] or None,
            release_ids=release_ids,
            urls=urls,
            extra=dict(shadows=extra),
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
            return True

        if existing.extra.get('shadows') and list(fe.extra['shadows'].keys())[0] in existing.extra['shadows']:
            # already imported from this shadow library; skip
            self.counts['exists'] += 1
            return False

        # check for edit conflicts
        if existing.ident in [e.ident for e in self._edits_inflight]:
            self.counts['skip-update-inflight'] += 1
            return False
        if fe.sha1 in [e.sha1 for e in self._edits_inflight]:
            raise Exception("Inflight insert; shouldn't happen")

        # minimum viable "existing" URL cleanup to fix dupes and broken links:
        # remove 'None' wayback URLs, and set archive.org rel 'archive'
        existing.urls = [u for u in existing.urls if not ('://web.archive.org/web/None/' in u.url)]
        for i in range(len(existing.urls)):
            u = existing.urls[i]
            if u.rel == 'repository' and '://archive.org/download/' in u.url:
                existing.urls[i].rel = 'archive'

        # merge the existing into this one and update
        existing.urls = list(set([(u.rel, u.url) for u in fe.urls + existing.urls]))
        existing.urls = [fatcat_openapi_client.FileUrl(rel=rel, url=url) for (rel, url) in existing.urls]
        if not existing.extra.get('shadows'):
            existing.extra['shadows'] = fe.extra['shadows']
        else:
            existing.extra['shadows'].update(fe.extra['shadows'])

        # do these "plus ones" because we really want to do these updates when possible
        if len(existing.urls) > SANE_MAX_URLS + 1:
            self.counts['skip-update-too-many-url'] += 1
            return None
        existing.release_ids = list(set(fe.release_ids + existing.release_ids))
        if len(existing.release_ids) > SANE_MAX_RELEASES + 1:
            self.counts['skip-update-too-many-releases'] += 1
            return None
        existing.mimetype = existing.mimetype or fe.mimetype
        existing.size = existing.size or fe.size
        existing.md5 = existing.md5 or fe.md5
        existing.sha1 = existing.sha1 or fe.sha1
        existing.sha256 = existing.sha256 or fe.sha256
        edit = self.api.update_file(self.get_editgroup_id(), existing.ident, existing)
        self._edits_inflight.append(edit)
        self.counts['update'] += 1
        return False

    def insert_batch(self, batch):
        self.api.create_file_auto_batch(fatcat_openapi_client.FileAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))

