
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
    munged JSON format (see ../extra/journal_metadata/).

    See guide for details on the many 'extra' fields used here.
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
        if raw_record.get('issnl'):
            return True
        return False

    def parse_record(self, row):
        """
        row is a python dict (parsed from JSON).

        returns a ContainerEntity (or None if invalid or couldn't parse)
        """

        extra = dict()
        for key in ('issne', 'issnp', 'languages', 'country', 'urls', 'abbrev',
            'coden', 'aliases', 'original_name', 'first_year', 'last_year',
            'platform', 'default_license', 'road', 'mimetypes',
            'sherpa_romeo', 'kbart'):
            if row.get(key):
                extra[key] = row[key]
        # TODO: not including for now: norwegian, dois/crossref, ia

        extra_doaj = dict()
        if row.get('doaj'):
            if row['doaj'].get('as_of'):
                extra_doaj['as_of'] = row['doaj']['as_of']
            if row['doaj'].get('works'):
                extra_doaj['works'] = row['doaj']['works']
        if extra_doaj:
            extra['doaj'] = extra_doaj

        extra_ia = dict()
        # TODO: would like an ia.longtail_ia flag
        if row.get('sim'):
            extra_ia['sim'] = {
                'year_spans': row['sim']['year_spans'],
            }
        if extra_ia:
            extra['ia'] = extra_ia

        ce = fatcat_client.ContainerEntity(
            issnl=row['issnl'],
            container_type=None, # TODO
            name=clean(row.get('name')),
            publisher=clean(row.get('publisher')),
            wikidata_qid=None, # TODO
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

