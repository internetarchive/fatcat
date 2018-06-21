
import sys
import json
import itertools
import fatcat_client
from fatcat.importer_common import FatcatImporter

# CSV format (generated from git.archive.org/webgroup/oa-journal-analysis):
# ISSN-L,in_doaj,in_road,in_norwegian,in_crossref,title,publisher,url,lang,ISSN-print,ISSN-electronic,doi_count,has_doi,is_oa,is_kept,publisher_size,url_live,url_live_status,url_live_final_status,url_live_final_url,url_live_status_simple,url_live_final_status_simple,url_domain,gwb_pdf_count

class FatcatIssnImporter(FatcatImporter):

    def parse_issn_row(self, row):
        """
        row is a python dict (parsed from CSV).
        returns a ContainerEntity
        """
        extra = dict(
            in_doaj=row['in_doaj'],
            in_road=row['in_road'],
            language=row['lang'],
            url=row['url'],
            ISSNp=row['ISSN-print'],
            ISSNe=row['ISSN-electronic'],
            is_oa=row['is_oa'],
            is_kept=row['is_kept'],
        )
        ce = fatcat_client.ContainerEntity(
            issnl=row['ISSN-L'],
            name=row['title'],
            publisher=row['publisher'],
            abbrev=None,
            coden=None,
            extra=extra)
        return ce

    def create_row(self, row, editgroup_id=None):
        ce = self.parse_issn_row(row)
        if ce is not None:
            ce.editgroup_id = editgroup_id
            self.api.create_container(ce)

    def create_batch(self, batch, editgroup_id=None):
        """Reads and processes in batches (not API-call-per-line)"""
        objects = [self.parse_issn_row(l)
                   for l in batch if l != None]
        objects = [o for o in objects if o != None]
        for o in objects:
            o.editgroup_id = editgroup_id
        self.api.create_container_batch(objects)
