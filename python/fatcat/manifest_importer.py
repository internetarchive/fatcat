
import sys
import json
import sqlite3
import itertools
import fatcat_client
from fatcat.importer_common import FatcatImporter


QUERY = "SELECT files_metadata.sha1, files_metadata.mimetype, files_metadata.size_bytes, files_metadata.md5, files_id_doi.doi, urls.url, urls.datetime from files_metadata JOIN files_id_doi ON files_metadata.sha1 = files_id_doi.sha1 JOIN urls ON files_metadata.sha1 = urls.sha1 ORDER BY files_metadata.sha1"

class FatcatManifestImporter(FatcatImporter):

    def parse_manifest_row(self, row):
        """
        obj is a python dict (parsed from json).
        returns a CreatorEntity
        """
        (sha1, mimetype, size_bytes, md5, doi, url, datetime) = row
        
        if url is None:
            return None
        release_ids = None
        if doi is not None:
            release_id = self.lookup_doi(doi.lower())
            if release_id:
                release_ids = [release_id,]
        if datetime is None:
            datetime = "1"
        urls = []
        if "//archive.org/" in url or "//arxiv.org/" in url:
            # TODO: special-case the arxiv.org bulk mirror?
            urls.append(fatcat_client.FileEntityUrls(url=url, rel="repository"))
        elif "//web.archive.org/" in url or "//archive.is/" in url:
            urls.append(fatcat_client.FileEntityUrls(url=url, rel="webarchive"))
        else:
            urls.append(fatcat_client.FileEntityUrls(url=url, rel="web"))
            urls.append(fatcat_client.FileEntityUrls(
                url="https://web.archive.org/web/{}/{}".format(datetime, url),
                rel="webarchive"))
            
        extra = None
        fe = fatcat_client.FileEntity(
            sha1=sha1,
            mimetype=mimetype,
            size=size_bytes,
            md5=md5,
            urls=urls,
            releases=release_ids,
            extra=extra)
        return fe

    def create_entity(self, entity, editgroup_id=None):
        if entity is not None:
            entity.editgroup_id = editgroup_id
            self.api.create_file(entity)

    def process_db(self, db_path, size=100):
        # TODO: multiple DOIs per sha1
        # TODO: multiple URLs per sha1 (with schema change)
        
        db = sqlite3.connect(db_path)
        last_sha1 = None

        print("Counting rows...")
        total_count = int(list(db.execute("SELECT COUNT(*) FROM files_metadata;"))[0][0])
        print("{} rows to process".format(total_count))

        eg = self.api.create_editgroup(fatcat_client.Editgroup(editor_id=1))
        i = 0
        j = -1
        for row in db.execute(QUERY):
            j = j+1
            if row[0] == last_sha1:
                continue
            else:
                last_sha1 = row[0]
            fe = self.parse_manifest_row(row)
            if fe is None:
                continue
            self.create_entity(fe, editgroup_id=eg.id)
            if i > 0 and (i % size) == 0:
                self.api.accept_editgroup(eg.id)
                eg = self.api.create_editgroup(fatcat_client.Editgroup(editor_id=1))
                print("Finished a batch; row {} of {} ({:.2f}%).\tTotal inserted: {}".format(
                    j, total_count, 100.0*j/total_count, i))
            i = i + 1
        if i == 0 or (i % size) != 0:
            self.api.accept_editgroup(eg.id)
        print("Done! Inserted {}".format(i))
