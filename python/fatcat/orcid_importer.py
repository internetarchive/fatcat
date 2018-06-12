
import sys
import json
import itertools
import fatcat_client

def value_or_none(e):
    if type(e) == dict:
        e = e.get('value')
    if type(e) == str and len(e) == 0:
        e = None
    return e

# from: https://docs.python.org/3/library/itertools.html
def grouper(iterable, n, fillvalue=None):
    "Collect data into fixed-length chunks or blocks"
    args = [iter(iterable)] * n
    return itertools.zip_longest(*args, fillvalue=fillvalue)

class FatcatOrcidImporter:

    def __init__(self, host_url):
        conf = fatcat_client.Configuration()
        conf.host = host_url
        self.api = fatcat_client.DefaultApi(fatcat_client.ApiClient(conf))

    def parse_orcid_dict(self, obj):
        """
        obj is a python dict (parsed from json).
        returns a CreatorEntity
        """
        name = obj['person']['name']
        if name is None:
            return None
        extra = None
        given = value_or_none(name.get('given-name'))
        sur = value_or_none(name.get('family-name'))
        display = value_or_none(name.get('credit-name'))
        if display is None:
            # TODO: sorry human beings
            display = "{} {}".format(given, sur)
        ce = fatcat_client.CreatorEntity(
            orcid=obj['orcid-identifier']['path'],
            given_name=given,
            surname=sur,
            display_name=display,
            extra=extra)
        return ce

    def process_line(self, line, editgroup_id=None):
        """Doesn't accept the editgroup"""
        obj = json.loads(line)
        ce = self.parse_orcid_dict(obj)
        if ce is not None:
            ce.editgroup_id = editgroup_id
            self.api.create_creator(ce)

    def process_source(self, source, group_size=100):
        """Creates and auto-accepts editgropu every group_size lines"""
        eg = self.api.create_editgroup(fatcat_client.Editgroup(editor_id=1))
        for i, line in enumerate(source):
            self.process_line(line, editgroup_id=eg.id)
            if i > 0 and (i % group_size) == 0:
                self.api.accept_editgroup(eg)
                eg = self.api.create_editgroup(fatcat_client.Editgroup(editor_id=1))
        if i == 0 or (i % group_size) != 0:
            self.api.accept_editgroup(eg.id)

    def process_batch(self, source, size=50):
        """Reads and processes in batches (not API-call-per-line)"""
        for lines in grouper(source, size):
            objects = [self.parse_orcid_dict(json.loads(l))
                       for l in lines if l != None]
            objects = [o for o in objects if o != None]
            eg = self.api.create_editgroup(fatcat_client.Editgroup(editor_id=1))
            for o in objects:
                o.editgroup_id = eg.id
            self.api.create_creator_batch(objects)
            self.api.accept_editgroup(eg.id)
            print("inserted {}".format(len(objects)))
