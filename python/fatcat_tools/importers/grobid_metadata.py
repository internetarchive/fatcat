#!/usr/bin/env python3

import json
import base64
import fatcat_openapi_client
from .common import EntityImporter, clean, make_rel_url

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
            editgroup_extra=eg_extra,
            **kwargs)
        self.default_link_rel = kwargs.get("default_link_rel", "web")
        self.longtail_oa = kwargs.get("longtail_oa", False)

    def want(self, raw_record):
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

        if not (fe and re):
            return None

        # lookup existing file SHA1
        existing = None
        try:
            existing = self.api.lookup_file(sha1=fe.sha1)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        # if file is already in here, presumably not actually long-tail
        # HACK: this is doing an exists check in parse_record(), which is weird
        # TODO: this is where we should check if the file actually has
        # release_ids and/or URLs associated with it
        if existing and not self.bezerk_mode:
            self.counts['exists'] += 1
            self.counts['skip'] -= 1
            return None

        release_edit = self.create_release(re)
        fe.release_ids.append(release_edit.ident)
        return fe

    def parse_grobid_json(self, obj):

        if not obj.get('title'):
            return None

        extra_grobid = dict()

        abstract = obj.get('abstract')
        if abstract and len(abstract) < MAX_ABSTRACT_BYTES and len(abstract) > 10:
            abobj = fatcat_openapi_client.ReleaseAbstract(
                mimetype="text/plain",
                content=clean(obj.get('abstract')))
            abstracts = [abobj]
        else:
            abstracts = None

        contribs = []
        for i, a in enumerate(obj.get('authors', [])):
            contribs.append(fatcat_openapi_client.ReleaseContrib(
                index=i,
                raw_name=clean(a['name']),
                given_name=clean(a.get('given_name')),
                surname=clean(a.get('surname')),
                role="author",
                extra=None))

        refs = []
        for raw in obj.get('citations', []):
            cite_extra = dict()
            year = None
            if raw.get('date'):
                try:
                    year = int(raw['date'].strip()[:4])
                except (IndexError, ValueError):
                    pass
            for key in ('volume', 'url', 'issue', 'publisher'):
                if raw.get(key):
                    cite_extra[key] = clean(raw[key])
            if raw.get('authors'):
                cite_extra['authors'] = [clean(a['name']) for a in raw['authors']]

            if not cite_extra:
                cite_extra = None
            refs.append(fatcat_openapi_client.ReleaseRef(
                key=clean(raw.get('id')),
                year=year,
                title=clean(raw['title']),
                extra=cite_extra))

        release_date = None
        release_year = None
        if obj.get('date'):
            # only returns year, ever?
            release_year = int(obj['date'][:4])

        extra = dict()
        if obj.get('doi'):
            extra['doi'] = obj['doi']
        if obj['journal'] and obj['journal'].get('name'):
            extra['container_name'] = clean(obj['journal']['name'])

        # TODO: ISSN/eISSN handling? or just journal name lookup?

        if extra_grobid:
            extra['grobid'] = extra_grobid
        if self.longtail_oa:
            extra['longtail_oa'] = True
        if not extra:
            extra = None

        title = clean(obj['title'], force_xml=True)
        if not title or len(title) < 2:
            return None

        re = fatcat_openapi_client.ReleaseEntity(
            title=title,
            release_type="article-journal",
            release_date=release_date,
            release_year=release_year,
            contribs=contribs,
            refs=refs,
            publisher=clean(obj['journal'].get('publisher')),
            volume=clean(obj['journal'].get('volume')),
            issue=clean(obj['journal'].get('issue')),
            abstracts=abstracts,
            ext_ids=fatcat_openapi_client.ReleaseExtIds(),
            extra=extra)
        return re

    def parse_file_metadata(self, sha1_key, cdx, mimetype, file_size):

        sha1 = base64.b16encode(base64.b32decode(sha1_key.replace('sha1:', ''))).decode('ascii').lower()

        fe = fatcat_openapi_client.FileEntity(
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
            fatcat_openapi_client.FileUrl(url=wayback, rel="webarchive"))
        original_url = make_rel_url(original, default_link_rel=self.default_link_rel)
        if original_url is not None:
            fe.urls.append(fatcat_openapi_client.FileUrl(rel=original_url[0], url=original_url[1]))

        return fe

    def try_update(self, entity):
        # did the exists check in 'parse_record()', because we needed to create a release
        return True

    def insert_batch(self, batch):
        self.api.create_file_auto_batch(fatcat_openapi_client.FileAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))
