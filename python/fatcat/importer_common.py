
import sys
import json
import itertools
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
        self._issn_issnl_map = None
        if issn_map_file:
            self.read_issn_map_file(issn_map_file)

    def process_source(self, source, group_size=100):
        """Creates and auto-accepts editgropu every group_size rows"""
        eg = self.api.create_editgroup(fatcat_client.Editgroup(editor_id=1))
        for i, row in enumerate(source):
            self.create_row(row, editgroup_id=eg.id)
            if i > 0 and (i % group_size) == 0:
                self.api.accept_editgroup(eg)
                eg = self.api.create_editgroup(fatcat_client.Editgroup(editor_id=1))
        if i == 0 or (i % group_size) != 0:
            self.api.accept_editgroup(eg.id)

    def process_batch(self, source, size=50):
        """Reads and processes in batches (not API-call-per-)"""
        for rows in grouper(source, size):
            eg = self.api.create_editgroup(fatcat_client.Editgroup(editor_id=1))
            self.create_batch(rows, eg.id)
            self.api.accept_editgroup(eg.id)

    def lookup_issnl(self, issnl):
        """Caches calls to the ISSN-L lookup API endpoint in a local dict"""
        assert len(issnl) == 9 and issnl[4] == '-'
        if issnl in self._issnl_id_map:
            return self._issnl_id_map[issn]
        container_id = None
        try:
            rv = self.api.lookup_container(issnl=issnl)
            container_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._issnl_id_map[issnl] = container_id # might be None
        return container_id

    def lookup_orcid(self, orcid):
        """Caches calls to the Orcid lookup API endpoint in a local dict"""
        assert len(orcid) == 19 and orcid[4] == '-'
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

    def read_issn_map_file(self, issn_map_file):
        self._issn_issnl_map = dict()
        for line in issn_map_file:
            if line.startswith("ISSN") or len(line) == 0:
                continue
            (issn, issnl) = line.split()[0:2]
            self._issn_issnl_map[issn] = issnl
            # double mapping makes lookups easy
            self._issn_issnl_map[issnl] = issnl

    def issn2issnl(self, issn):
        if issn is None:
            return None
        self._issn_issnl_map.get(issn)
