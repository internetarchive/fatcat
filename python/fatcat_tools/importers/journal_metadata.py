
import sys
import json
import itertools
import fatcat_client
from .common import EntityImporter, clean


def or_none(s):
    if s is None:
        return None
    if len(s) == 0:
        return None
    return s

def truthy(s):
    if s is None:
        return None
    s = s.lower()

    if s in ('true', 't', 'yes', 'y', '1'):
        return True
    elif s in ('false', 'f', 'no', 'n', '0'):
        return False
    else:
        return None

class JournalMetadataImporter(EntityImporter):
    """
    Imports journal metadata ("containers") by ISSN, currently from a custom
    (data munged) .csv file format

    CSV format (generated from git.archive.org/webgroup/oa-journal-analysis):

        ISSN-L,in_doaj,in_road,in_norwegian,in_crossref,title,publisher,url,lang,ISSN-print,ISSN-electronic,doi_count,has_doi,is_oa,is_kept,publisher_size,url_live,url_live_status,url_live_final_status,url_live_final_url,url_live_status_simple,url_live_final_status_simple,url_domain,gwb_pdf_count
    """

    def __init__(self, api, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Automated import of container-level metadata, by ISSN. Metadata from Internet Archive munging.")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.JournalMetadataImporter')
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra)

    def want(self, raw_record):
        if raw_record.get('ISSN-L'):
            return True
        return False

    def parse_record(self, row):
        """
        row is a python dict (parsed from CSV).
        returns a ContainerEntity (or None if invalid or couldn't parse)
        """
        title = or_none(row['title'])
        issnl = or_none(row['ISSN-L'])
        if title is None or issnl is None:
            return None
        extra = dict(
            in_doaj=truthy(row['in_doaj']),
            in_road=truthy(row['in_road']),
            in_norwegian=truthy(row['in_norwegian']),
            language=or_none(row['lang']),
            url=or_none(row['url']),
            ISSNp=or_none(row['ISSN-print']),
            ISSNe=or_none(row['ISSN-electronic']),
            is_oa=truthy(row['is_oa']),
            is_kept=truthy(row['is_kept']),
        )
        ce = fatcat_client.ContainerEntity(
            issnl=issnl,
            name=clean(title),
            publisher=or_none(clean(row['publisher'])),
            extra=extra)
        return ce

    def try_update(self, ce):

        existing = None
        try:
            existing = self.api.lookup_container(issnl=ce.issnl)
        except fatcat_client.rest.ApiException as err:
            if err.status != 404:
                raise err
            # doesn't exist, need to update
            return True

        # eventually we'll want to support "updates", but for now just skip if
        # entity already exists
        if existing:
            self.counts['exists'] += 1
            return False
        
        return True

    def insert_batch(self, batch):
        self.api.create_container_batch(batch,
            autoaccept=True,
            description=self.editgroup_description,
            extra=json.dumps(self.editgroup_extra))

