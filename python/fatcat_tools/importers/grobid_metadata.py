#!/usr/bin/env python3

import sys
import json
import base64
import datetime
import fatcat_client
from .common import EntityImporter

MAX_ABSTRACT_BYTES=4096


class GrobidMetadataImporter(EntityImporter):
    """
    This is a complex case: we need to parse and create both file and release entities.

    The "primary" entity here is really File, not Release. If a matching File
    exists, we bail in want(); if not we insert the Release during parsing, and
    insert both.

    TODO: should instead check if the File has any releases; if not, insert and update.
    TODO: relaxing 'None' constraint on parse_record() might make this refactor-able.
    """

    def __init__(self, api, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Import of release and file metadata, as extracted from PDFs by GROBID.")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.GrobidMetadataImporter')
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra)
        self.default_link_rel = kwargs.get("default_link_rel", "web")

    def want(self, raw_record):

        fields = raw_record.split('\t')
        sha1_key = fields[0]
        sha1 = base64.b16encode(base64.b32decode(sha1_key.replace('sha1:', ''))).decode('ascii').lower()
        #cdx = json.loads(fields[1])
        #mimetype = fields[2]
        #file_size = int(fields[3])
        grobid_meta = json.loads(fields[4])

        if not grobid_meta.get('title'):
            return False

        # lookup existing file SHA1
        try:
            existing_file = self.api.lookup_file(sha1=sha1)
        except fatcat_client.rest.ApiException as err:
            if err.status != 404:
                raise err
            existing_file = None

        # if file is already in here, presumably not actually long-tail
        # TODO: this is where we should check if the file actually has
        # release_ids and/or URLs associated with it
        if existing_file and not self.bezerk_mode:
            return False
        return True

    def parse_record(self, row):

        fields = row.split('\t')
        sha1_key = fields[0]
        cdx = json.loads(fields[1])
        mimetype = fields[2]
        file_size = int(fields[3])
        grobid_meta = json.loads(fields[4])
        fe = self.parse_file_metadata(sha1_key, cdx, mimetype, file_size)
        re = self.parse_grobid_json(grobid_meta)
        assert (fe and re)

        release_edit = self.create_release(re)
        fe.release_ids.append(release_edit.ident)
        return fe

    def parse_grobid_json(self, obj):
        assert obj.get('title')

        extra = dict()

        if obj.get('abstract') and len(obj.get('abstract')) < MAX_ABSTRACT_BYTES:
            abobj = dict(
                mimetype="text/plain",
                language=None,
                content=obj.get('abstract').strip())
            abstracts = [abobj]
        else:
            abstracts = None

        contribs = []
        for i, a in enumerate(obj.get('authors', [])):
            contribs.append(fatcat_client.ReleaseContrib(
                index=i,
                raw_name=a['name'],
                role="author",
                extra=None))

        refs = []
        for raw in obj.get('citations', []):
            cite_extra = dict()
            ref = dict()
            ref['key'] = raw.get('id')
            if raw.get('title'):
                ref['title'] = raw['title'].strip()
            if raw.get('date'):
                try:
                    year = int(raw['date'].strip()[:4])
                    ref['year'] = year
                except:
                    pass
            for key in ('volume', 'url', 'issue', 'publisher'):
                if raw.get(key):
                    cite_extra[key] = raw[key].strip()
            if raw.get('authors'):
                cite_extra['authors'] = [a['name'] for a in raw['authors']]
            if cite_extra:
                cite_extra = dict(grobid=cite_extra)
            else:
                cite_extra = None
            ref['extra'] = cite_extra
            refs.append(ref)

        release_date = None
        release_year = None
        if obj.get('date'):
            # only returns year, ever?
            release_year = int(obj['date'][:4])

        if obj.get('doi'):
            extra['doi'] = obj['doi']
        if obj['journal'] and obj['journal'].get('name'):
            extra['container_name'] = obj['journal']['name']

        extra['is_longtail_oa'] = True

        # TODO: ISSN/eISSN handling? or just journal name lookup?

        if extra:
            extra = dict(grobid=extra)
        else:
            extra = None

        re = fatcat_client.ReleaseEntity(
            title=obj['title'].strip(),
            release_type="article-journal",
            release_date=release_date,
            release_year=release_year,
            contribs=contribs,
            refs=refs,
            publisher=obj['journal'].get('publisher'),
            volume=obj['journal'].get('volume'),
            issue=obj['journal'].get('issue'),
            abstracts=abstracts,
            extra=extra)
        return re

    # TODO: make this a common function somewhere
    def make_url(self, raw):
        rel = self.default_link_rel
        # TODO: this is where we could map specific domains to rel types,
        # and also filter out bad domains, invalid URLs, etc
        if "//archive.org/" in raw or "//arxiv.org/" in raw:
            # TODO: special-case the arxiv.org bulk mirror?
            rel = "repository"
        elif "//web.archive.org/" in raw or "//archive.is/" in raw:
            rel = "webarchive"
        return fatcat_client.FileEntityUrls(url=raw, rel=rel)

    def parse_file_metadata(self, sha1_key, cdx, mimetype, file_size):

        sha1 = base64.b16encode(base64.b32decode(sha1_key.replace('sha1:', ''))).decode('ascii').lower()

        fe = fatcat_client.FileEntity(
            sha1=sha1,
            size=int(file_size),
            mimetype=mimetype,
            release_ids=[],
            urls=[],
        )

        # parse URLs and CDX
        original = cdx['url']
        assert len(cdx['dt']) >= 8
        wayback = "https://web.archive.org/web/{}/{}".format(
            cdx['dt'],
            original)
        fe.urls.append(
            fatcat_client.FileEntityUrls(url=wayback, rel="webarchive"))
        original_url = self.make_url(original)
        if original_url is not None:
            fe.urls.append(original_url)

        return fe

    def try_update(entity):
        # we did this in want()
        return True

    def insert_batch(self, batch):
        self.api.create_file_batch(batch,
            autoaccept=True,
            description=self.editgroup_description,
            extra=json.dumps(self.editgroup_extra))

