
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


    'extra' fields:

        doaj
            as_of: datetime of most recent check; if not set, not actually in DOAJ
            seal: bool
            work_level: bool (are work-level publications deposited with DOAJ?)
            archiving: array, can include 'library' or 'other'
        road
            as_of: datetime of most recent check; if not set, not actually in ROAD
        pubmed (TODO: delete?)
            as_of: datetime of most recent check; if not set, not actually indexed in pubmed
        norwegian (TODO: drop this?)
            as_of: datetime of most recent check; if not set, not actually indexed in pubmed
            id (integer)
            level (integer; 0-2)
        kbart
            lockss
                year_rle
                volume_rle
            portico
                ...
            clockss
                ...
        sherpa_romeo
            color
        jstor
            year_rle
            volume_rle
        scopus
            id
            TODO: print/electronic distinction?
        wos
            id
        doi
            crossref_doi: DOI of the title in crossref (if exists)
            prefixes: array of strings (DOI prefixes, up to the '/'; any registrar, not just Crossref)
        ia
            sim
                nap_id
                year_rle
                volume_rle
            longtail: boolean
            homepage
                as_of: datetime of last attempt
                url
                status: HTTP/heritrix status of homepage crawl

        issnp: string
        issne: string
        coden: string
        abbrev: string
        oclc_id: string (TODO: lookup?)
        lccn_id: string (TODO: lookup?)
        dblb_id: string
        default_license: slug
        original_name: native name (if name is translated)
        platform: hosting platform: OJS, wordpress, scielo, etc
        mimetypes: array of strings (eg, 'application/pdf', 'text/html')
        first_year: year (integer)
        last_year: if publishing has stopped
        primary_language: single ISO code, or 'mixed'
        languages: array of ISO codes
        region: TODO: continent/world-region
        nation: shortcode of nation
        discipline: TODO: highest-level subject; "life science", "humanities", etc
        field: TODO: narrower description of field
        subjects: TODO?
        url: homepage
        is_oa: boolean. If true, can assume all releases under this container are "Open Access"
        TODO: domains, if exclusive?
        TODO: fulltext_regex, if a known pattern?

    For KBART, etc:
        We "over-count" on the assumption that "in-progress" status works will soon actually be preserved.
        year and volume spans are run-length-encoded arrays, using integers:
            - if an integer, means that year is preserved
            - if an array of length 2, means everything between the two numbers (inclusive) is preserved
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

