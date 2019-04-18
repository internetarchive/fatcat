
import sys
import json
import base64
import sqlite3
import itertools
import fatcat_client
from .common import EntityImporter, clean, make_rel_url


def b32_hex(s):
    s = s.strip().split()[0].lower()
    if s.startswith("sha1:"):
        s = s[5:]
    if len(s) != 32:
        return s
    return base64.b16encode(base64.b32decode(s.upper())).lower().decode('utf-8')


ARABESQUE_MATCH_WHERE_CLAUSE='WHERE hit = 1 AND identifier IS NOT NULL'

class ArabesqueMatchImporter(EntityImporter):
    """
    Importer for arabesque crawl report .sqlite files, which contain
    file/identifier matches based on URL/identifier seedlists.

    Uses a SQL query to iterate through table looking for rows with:

    - GROBID status 200
    - known SHA-1 (base32 format)
    - known timestamp

    Looks up release (by identifier) and file (by SHA-1). If no release exists,
    bail.

    If no file exists, create one from metadata (using both direct and wayback
    URLs), link to release, and insert.

    If file exists, optionally update it:

    - if no release match, match to the release
    - if new URL not included, add it (and wayback)

    Config options:
    - default URL rel
    - crawl id (for editgroup metadata)
    - identifier type

    TODO:
    - a mode to insert bare files even if identifier not known?
    """

    def __init__(self, api, extid_type, require_grobid=True, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Match web crawl files to releases based on identifier/URL seedlist")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.ArabesqueMatchImporter')
        if kwargs.get('crawl_id'):
            eg_extra['crawl_id'] = eg_extra.get('crawl_id')
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)
        assert extid_type in ('doi', 'pmcid', 'pmid')
        self.extid_type = extid_type
        self.default_link_rel = kwargs.get("default_link_rel", "web")
        self.default_mime = kwargs.get("default_mime", None)
        self.do_updates = kwargs.get("do_updates", False)
        self.require_grobid = require_grobid
        if self.require_grobid:
            print("Requiring GROBID status == 200")
        else:
            print("NOT checking GROBID status column")

    def want(self, row):
        if self.require_grobid and not row['postproc_status'] == "200":
            return False
        if (row['hit'] == True
                and row['final_sha1']
                and row['final_timestamp'] and row['final_timestamp'] != "-"
                and row['final_mimetype']
                and row['hit'] == True
                and row['identifier']):
            return True
        else:
            return False

    def parse_record(self, row):

        extid = row['identifier'].strip()

        # check/cleanup DOI
        if self.extid_type == 'doi':
            self.extid_type.replace('http://doi.org/', '')
            self.extid_type.replace('https://doi.org/', '')
            if not extid.startswith('10.'):
                self.counts['skip-bad-doi']
                return None

        # lookup extid
        try:
            re = self.api.lookup_release(**{self.extid_type: extid})
        except fatcat_client.rest.ApiException as err:
            if err.status != 404:
                raise err
            # bail on 404 (release not in DB)
            self.counts['skip-extid-not-found'] += 1
            return None

        url = make_rel_url(row['final_url'], self.default_link_rel)
        if not url:
            self.counts['skip-url'] += 1
            return None
        wayback = "https://web.archive.org/web/{}/{}".format(
            row['final_timestamp'],
            row['final_url'])
        urls = [url, ("webarchive", wayback)]

        urls = [fatcat_client.FileEntityUrls(rel=rel, url=url) for (rel, url) in urls]

        fe = fatcat_client.FileEntity(
            sha1=b32_hex(row['final_sha1']),
            mimetype=row['final_mimetype'],
            release_ids=[re.ident],
            urls=urls,
        )
        return fe

    def try_update(self, fe):
        # lookup sha1, or create new entity
        existing = None
        try:
            existing = self.api.lookup_file(sha1=fe.sha1)
        except fatcat_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        if not existing:
            return True

        if (fe.release_ids[0] in existing.release_ids) and existing.urls:
            # TODO: could still, in theory update with the new URL?
            self.counts['exists'] += 1
            return False

        if not self.do_updates:
            self.counts['skip-update-disabled'] += 1
            return False

        if set(fe.release_ids) == set(existing.release_ids):
            existing_urls = set([u.url for u in existing.urls])
            new_urls = set([u.url for u in fe.urls])
            if existing_urls.issuperset(new_urls):
                self.counts['skip-update-nothing-new'] += 1
                return False

        # merge the existing into this one and update
        existing.urls = list(set([(u.rel, u.url) for u in fe.urls + existing.urls]))
        existing.urls = [fatcat_client.FileEntityUrls(rel=rel, url=url) for (rel, url) in existing.urls]
        existing.release_ids = list(set(fe.release_ids + existing.release_ids))
        existing.mimetype = existing.mimetype or fe.mimetype
        self.api.update_file(existing.ident, existing, editgroup_id=self.get_editgroup_id())
        self.counts['update'] += 1
        return False

    def insert_batch(self, batch):
        self.api.create_file_batch(batch,
            autoaccept=True,
            description=self.editgroup_description,
            extra=json.dumps(self.editgroup_extra))

