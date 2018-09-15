
import sys
import json
import sqlite3
import itertools
import fatcat_client
from fatcat.importer_common import FatcatImporter

#row = row.split('\t')
#assert len(row) == 2
#sha1 = row[0].replace('sha1:')
#sha1 = base64.b16encode(base64.b32decode(sha1)).lower()
#print(sha1)
#dois = [d.lower() for d in json.loads(row[1])]

class FatcatMatchedImporter(FatcatImporter):
    """
    Input format is JSON with keys:
    - dois (list)
    - sha1 (hex)
    - md5 (hex)
    - sha256 (hex)
    - size (int)
    - cdx (list of objects)
        - dt
        - url
    - mimetype
    - urls (list of strings... or objects?)

    Future handlings/extensions:
    - core_id, wikidata_id, pmcid, pmid: not as lists
    """

    def __init__(self, host_url, skip_file_update=False, default_mime=None,
            default_link_rel="web"):
        super().__init__(host_url)
        self.default_mime = default_mime
        self.default_link_rel = default_link_rel
        self.skip_file_update = skip_file_update

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

    def parse_matched_dict(self, obj):
        sha1 = obj['sha1']
        dois = [d.lower() for d in obj.get('dois', [])]

        # lookup sha1, or create new entity
        fe = None
        if not self.skip_file_update:
            try:
                fe = self.api.lookup_file(sha1=sha1)
            except fatcat_client.rest.ApiException as err:
                if err.status != 404:
                    raise err
        if fe is None:
            fe = fatcat_client.FileEntity(
                sha1=sha1,
                releases=[],
                urls=[],
            )

        # lookup dois
        re_list = set()
        for doi in dois:
            try:
                re = self.api.lookup_release(doi=doi)
            except fatcat_client.rest.ApiException as err:
                if err.status != 404:
                    raise err
                re = None
            if re is None:
                print("DOI not found: {}".format(doi))
            else:
                re_list.add(re.ident)
        if len(re_list) == 0:
            return None
        if fe.releases == set(re_list):
            return None
        re_list.update(fe.releases)
        fe.releases = list(re_list)

        # parse URLs and CDX
        existing_urls = [feu.url for feu in fe.urls]
        for url in obj.get('url', []):
            if url not in existing_urls:
                url = self.make_url(url)
                if url != None:
                    fe.urls.append(url)
        for cdx in obj.get('cdx', []):
            original = cdx['url']
            wayback = "https://web.archive.org/web/{}/{}".format(
                cdx['dt'],
                original)
            if wayback not in existing_urls:
                fe.urls.append(
                    fatcat_client.FileEntityUrls(url=wayback, rel="webarchive"))
            if original not in existing_urls:
                url = self.make_url(original)
                if url != None:
                    fe.urls.append(url)

        if obj.get('size') != None:
            fe.size = int(obj['size'])
        fe.sha256 = obj.get('sha256', fe.sha256)
        fe.md5 = obj.get('md5', fe.sha256)
        if obj.get('mimetype') is None:
            if fe.mimetype is None:
                fe.mimetype = self.default_mime
        else:
            fe.mimetype = obj.get('mimetype')
        return fe

    def create_row(self, row, editgroup=None):
        obj = json.loads(row)
        fe = self.parse_matched_dict(obj)
        if fe is not None:
            if fe.ident is None:
                self.api.create_file(fe, editgroup=editgroup)
                self.insert_count = self.insert_count + 1
            else:
                self.api.update_file(fe.ident, fe, editgroup=editgroup)
                self.update_count = self.update_count + 1

    def create_batch(self, batch, editgroup=None):
        """Reads and processes in batches (not API-call-per-line)"""
        objects = [self.parse_matched_dict(json.loads(l))
                   for l in batch if l != None]
        new_objects = [o for o in objects if o != None and o.ident == None]
        update_objects = [o for o in objects if o != None and o.ident != None]
        for obj in update_objects:
            self.api.update_file(obj.ident, obj, editgroup=editgroup)
        if len(new_objects) > 0:
            self.api.create_file_batch(new_objects, autoaccept="true", editgroup=editgroup)
        self.update_count = self.update_count + len(update_objects)
        self.insert_count = self.insert_count + len(new_objects)
