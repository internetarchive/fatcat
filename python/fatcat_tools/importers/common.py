
import re
import sys
import csv
import json
import itertools
from collections import Counter

import fatcat_client
from fatcat_client.rest import ApiException

# from: https://docs.python.org/3/library/itertools.html
def grouper(iterable, n, fillvalue=None):
    "Collect data into fixed-length chunks or blocks"
    args = [iter(iterable)] * n
    return itertools.zip_longest(*args, fillvalue=fillvalue)

class FatcatImporter:

    def __init__(self, host_url, issn_map_file=None):
        conf = fatcat_client.Configuration()
        conf.host = host_url
        self.api = fatcat_client.DefaultApi(fatcat_client.ApiClient(conf))
        self._issnl_id_map = dict()
        self._orcid_id_map = dict()
        self._doi_id_map = dict()
        self._issn_issnl_map = None
        self._orcid_regex = re.compile("^\\d{4}-\\d{4}-\\d{4}-\\d{3}[\\dX]$")
        if issn_map_file:
            self.read_issn_map_file(issn_map_file)
        self.counts = Counter({'insert': 0, 'update': 0, 'processed_lines': 0})

    def describe_run(self):
        print("Processed {} lines, inserted {}, updated {}.".format(
            self.counts['processed_lines'], self.counts['insert'], self.counts['update']))

    def process_source(self, source, group_size=100):
        """Creates and auto-accepts editgroup every group_size rows"""
        eg = self.api.create_editgroup(
            fatcat_client.Editgroup(editor_id='aaaaaaaaaaaabkvkaaaaaaaaae'))
        for i, row in enumerate(source):
            self.create_row(row, editgroup=eg.id)
            if i > 0 and (i % group_size) == 0:
                self.api.accept_editgroup(eg.id)
                eg = self.api.create_editgroup(
                    fatcat_client.Editgroup(editor_id='aaaaaaaaaaaabkvkaaaaaaaaae'))
            self.counts['processed_lines'] += 1
        if i == 0 or (i % group_size) != 0:
            self.api.accept_editgroup(eg.id)

    def process_batch(self, source, size=50):
        """Reads and processes in batches (not API-call-per-)"""
        for rows in grouper(source, size):
            self.counts['processed_lines'] += len(rows)
            eg = self.api.create_editgroup(
                fatcat_client.Editgroup(editor_id='aaaaaaaaaaaabkvkaaaaaaaaae'))
            self.create_batch(rows, editgroup=eg.id)

    def process_csv_source(self, source, group_size=100, delimiter=','):
        reader = csv.DictReader(source, delimiter=delimiter)
        self.process_source(reader, group_size)

    def process_csv_batch(self, source, size=50, delimiter=','):
        reader = csv.DictReader(source, delimiter=delimiter)
        self.process_batch(reader, size)

    def is_issnl(self, issnl):
        return len(issnl) == 9 and issnl[4] == '-'

    def lookup_issnl(self, issnl):
        """Caches calls to the ISSN-L lookup API endpoint in a local dict"""
        if issnl in self._issnl_id_map:
            return self._issnl_id_map[issnl]
        container_id = None
        try:
            rv = self.api.lookup_container(issnl=issnl)
            container_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._issnl_id_map[issnl] = container_id # might be None
        return container_id

    def is_orcid(self, orcid):
        return self._orcid_regex.match(orcid) != None

    def lookup_orcid(self, orcid):
        """Caches calls to the Orcid lookup API endpoint in a local dict"""
        if not self.is_orcid(orcid):
            return None
        if orcid in self._orcid_id_map:
            return self._orcid_id_map[orcid]
        creator_id = None
        try:
            rv = self.api.lookup_creator(orcid=orcid)
            creator_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._orcid_id_map[orcid] = creator_id # might be None
        return creator_id

    def is_doi(self, doi):
        return doi.startswith("10.") and doi.count("/") >= 1

    def lookup_doi(self, doi):
        """Caches calls to the doi lookup API endpoint in a local dict"""
        assert self.is_doi(doi)
        doi = doi.lower()
        if doi in self._doi_id_map:
            return self._doi_id_map[doi]
        release_id = None
        try:
            rv = self.api.lookup_release(doi=doi)
            release_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._doi_id_map[doi] = release_id # might be None
        return release_id

    def read_issn_map_file(self, issn_map_file):
        print("Loading ISSN map file...")
        self._issn_issnl_map = dict()
        for line in issn_map_file:
            if line.startswith("ISSN") or len(line) == 0:
                continue
            (issn, issnl) = line.split()[0:2]
            self._issn_issnl_map[issn] = issnl
            # double mapping makes lookups easy
            self._issn_issnl_map[issnl] = issnl
        print("Got {} ISSN-L mappings.".format(len(self._issn_issnl_map)))

    def issn2issnl(self, issn):
        if issn is None:
            return None
        return self._issn_issnl_map.get(issn)
