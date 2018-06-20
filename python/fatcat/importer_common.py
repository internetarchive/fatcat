
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

    def __init__(self, host_url):
        conf = fatcat_client.Configuration()
        conf.host = host_url
        self.api = fatcat_client.DefaultApi(fatcat_client.ApiClient(conf))
        self._issnl_map = dict()

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
        if issnl in self._issnl_map:
            return self._issnl_map[issn]
        container_id = None
        try:
            rv = self.api.lookup_container(issnl=issnl)
            container_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._issnl_map[issnl] = container_id # might be None
        return container_id
