#!/usr/bin/env python3

import sys
import json
import base64
import datetime
import fatcat_client
from fatcat_tools.importers.common import FatcatImporter

MAX_ABSTRACT_BYTES=4096


class FatcatGrobidMetadataImporter(FatcatImporter):

    def __init__(self, host_url, default_link_rel="web"):
        super().__init__(host_url)
        self.default_link_rel = default_link_rel

    def parse_grobid_json(self, obj):

        if not obj.get('title'):
            return None

        release = dict()
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
            c = dict(raw_name=a['name'], role="author")
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

        release_type = "journal-article"
        release_date = None
        if obj.get('date'):
            # TODO: only returns year, ever? how to handle?
            release_date = datetime.datetime(year=int(obj['date'][:4]), month=1, day=1)

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

        # lookup existing SHA1, or create new entity
        try:
            existing_file = self.api.lookup_file(sha1=sha1)
        except fatcat_client.rest.ApiException as err:
            if err.status != 404:
                raise err
            existing_file = None

        if existing_file:
            # if file is already in here, presumably not actually long-tail
            return None
        fe = fatcat_client.FileEntity(
            sha1=sha1,
            size=int(file_size),
            mimetype=mimetype,
            releases=[],
            urls=[],
        )

        # parse URLs and CDX
        original = cdx['url']
        wayback = "https://web.archive.org/web/{}/{}".format(
            cdx['dt'],
            original)
        fe.urls.append(
            fatcat_client.FileEntityUrls(url=wayback, rel="webarchive"))
        original_url = self.make_url(original)
        if original_url != None:
            fe.urls.append(original_url)

        return fe

    def create_row(self, row, editgroup=None):
        if not row:
            return
        fields = row.split('\t')
        sha1_key = fields[0]
        cdx = json.loads(fields[1])
        mimetype = fields[2]
        file_size = int(fields[3])
        grobid_meta = json.loads(fields[4])
        fe = self.parse_file_metadata(sha1_key, cdx, mimetype, file_size)
        re = self.parse_grobid_json(grobid_meta)
        if fe and re:
            release_entity = self.api.create_release(re, editgroup=editgroup)
            # release ident can't already be in release list because we just
            # created it
            fe.releases.append(release_entity.ident)
            file_entity = self.api.create_file(fe, editgroup=editgroup)
            self.counts['insert'] += 1

    # NB: batch mode not implemented
