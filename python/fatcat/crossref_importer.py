
import sys
import json
import itertools
import fatcat_client
from fatcat.importer_common import FatcatImporter


class FatcatCrossrefImporter(FatcatImporter):

    # TODO: overload __init__ to handle create_containers

    def parse_crossref_dict(self, obj):
        """
        obj is a python dict (parsed from json).
        returns a ReleaseEntity
        """

        # contribs
        contribs = []
        for i, am in enumerate(obj['author']):
            contribs.append(fatcat_client.ReleaseContrib(
                creator_id=None, # TODO: orcid lookup
                index=i,
                # Sorry humans :(
                raw="{} {}".format(am['given'], am['family']),
                role="author"))

        # container
        # TODO: ISSN vs. ISSN-L
        issn = obj.get('ISSN', [None])[0]
        container_id = self.lookup_issnl(issn)

        ## TODO: create containers in-line like this?
        #container = dict(
        #    issn=issn,
        #    name=obj['container-title'][0],
        #    container=container_id,
        #    #sortname=obj['short-container-title'][0])
        #    publisher=obj['publisher'])
        #if container_id is None and self.create_containers and issn != None:
        #    rv = self.post('/v0/container', data=dict(
        #        issn=container['issn'],
        #        publisher=container['publisher']))
        #    assert rv.status_code == 201
        #    container_id = rv.json()['id']
        #    print("created container: {}".format(issn))
        #    container['id'] = container_id
        #    self._issn_map[issn] = container_id

        # references
        refs = []
        for i, rm in enumerate(obj.get('reference', [])):
            refs.append(fatcat_client.ReleaseRef(
                index=i,
                target_release_id=None, # TODO: DOI lookup: rm.get("DOI", None),
                # TODO: all these
                key=None,
                year=None,
                container_title=None,
                title=None,
                locator=None,
                # TODO: how to generate a proper stub here from k/v objdata?
                # TODO: just dump JSON here if we didn't get a match?
                raw="| ".join(rm.values())))

        # work
        we = fatcat_client.WorkEntity(
            work_type=obj['type'],
        )

        # release
        extra = dict(crossref={
            'links': obj.get('link', []),
            'subject': obj.get('subject'),
            'crossref-type': obj['type'],
            'alternative-id': obj.get('alternative-id', [])})

        re = fatcat_client.ReleaseEntity(
            work_id='null', # XXX:
            title=obj['title'][0],
            contribs=contribs,
            refs=refs,
            container_id=container_id,
            release_type=obj['type'],
            doi=obj['DOI'],
            release_date=obj['created']['date-time'],
            #license=obj.get('license', [dict(URL=None)])[0]['URL'] or None,
            issue=obj.get('issue'),
            volume=obj.get('volume'),
            pages=obj.get('page'),
            extra=extra)
        return (we, re)

    def create_row(self, row, editgroup_id=None):
        if row is None:
            continue
        obj = json.loads(row)
        both = self.parse_crossref_dict(obj)
        if both is not None:
            (we, re) = both
            we.editgroup_id = editgroup_id
            re.editgroup_id = editgroup_id
            created = self.api.create_work(we)
            re.work_id = created.ident
            self.api.create_release(re)

    def create_batch(self, batch, editgroup_id=None):
        """Current work/release pairing disallows batch creation of releases.
        Could do batch work creation and then match against releases, but meh."""
        for row in batch:
            self.create_row(row, editgroup_id)
