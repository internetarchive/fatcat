
import sys
import json
import itertools
import fatcat_client
from fatcat.importer_common import FatcatImporter


def value_or_none(e):
    if type(e) == dict:
        e = e.get('value')
    if type(e) == str and len(e) == 0:
        e = None
    # TODO: this is probably bogus; patched in desperation; remove?
    if e:
        try:
            e.encode()
        except UnicodeEncodeError:
            # Invalid JSON?
            print("BAD UNICODE")
            return None
    return e

class FatcatOrcidImporter(FatcatImporter):

    def parse_orcid_dict(self, obj):
        """
        obj is a python dict (parsed from json).
        returns a CreatorEntity
        """
        name = obj['person']['name']
        if name is None:
            return None
        extra = None
        given = value_or_none(name.get('given-names'))
        sur = value_or_none(name.get('family-name'))
        display = value_or_none(name.get('credit-name'))
        if display is None:
            # TODO: sorry human beings
            if given and sur:
                display = "{} {}".format(given, sur)
            elif sur:
                display = sur
            elif given:
                display = given
            else:
                # must have *some* name
                return None
        ce = fatcat_client.CreatorEntity(
            orcid=obj['orcid-identifier']['path'],
            given_name=given,
            surname=sur,
            display_name=display,
            extra=extra)
        return ce

    def create_row(self, row, editgroup_id=None):
        obj = json.loads(row)
        ce = self.parse_orcid_dict(obj)
        if ce is not None:
            ce.editgroup_id = editgroup_id
            self.api.create_creator(ce)

    def create_batch(self, batch, editgroup_id=None):
        """Reads and processes in batches (not API-call-per-line)"""
        objects = [self.parse_orcid_dict(json.loads(l))
                   for l in batch if l != None]
        objects = [o for o in objects if o != None]
        for o in objects:
            o.editgroup_id = editgroup_id
        self.api.create_creator_batch(objects)
