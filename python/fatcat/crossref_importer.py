
import sys
import json
import itertools
import fatcat_client
from fatcat.importer_common import FatcatImporter


class FatcatCrossrefImporter(FatcatImporter):

    def __init__(self, host_url, issn_map_file, create_containers=True):
        super().__init__(host_url, issn_map_file)
        self.create_containers = create_containers

    def parse_crossref_dict(self, obj):
        """
        obj is a python dict (parsed from json).
        returns a ReleaseEntity
        """

        # contribs
        contribs = []
        for i, am in enumerate(obj['author']):
            creator_id = None
            if 'ORCID' in am.keys():
                creator_id = self.lookup_orcid(am['ORCID'].split('/')[-1])
            contribs.append(fatcat_client.ReleaseContrib(
                creator_id=creator_id,
                index=i+1,
                # Sorry humans :(
                raw="{} {}".format(am['given'], am['family']),
                role="author"))

        # container
        issn = obj.get('ISSN', [None])[0]
        issnl = self.issn2issnl(issn)
        container_id = None
        if issnl:
            container_id = self.lookup_issnl(issnl)
        publisher = obj['publisher']

        ce = None
        if container_id is None and self.create_containers and issnl != None:
            ce = fatcat_client.ContainerEntity(
                issnl=issnl,
                publisher=publisher,
                name=obj['container-title'][0])
            print("created container: {}".format(issnl))

        # references
        refs = []
        for i, rm in enumerate(obj.get('reference', [])):
            try:
                year = int(rm.get('year'))
            except:
                year = None
            refs.append(fatcat_client.ReleaseRef(
                index=i+1,
                target_release_id=None, # TODO: DOI lookup: rm.get("DOI", None),
                # unreliable for crossref: key=rm['key'].split('|')[-1],
                year=year,
                container_title=rm.get('volume-title'),
                title=rm.get('title'),
                locator=rm.get('first-page'),
                # TODO: just dump JSON somewhere here?
                raw=rm.get('unstructured')))

        # work
        we = fatcat_client.WorkEntity(
            work_type=obj['type'],
        )

        # release
        extra = dict(crossref={
            'links': obj.get('link', []),
            'subject': obj.get('subject'),
            'type': obj['type'],
            'license': obj.get('license', [dict(URL=None)])[0]['URL'] or None,
            'alternative-id': obj.get('alternative-id', [])})

        re = fatcat_client.ReleaseEntity(
            work_id='tbd', # gets set later, I promise!
            title=obj['title'][0],
            contribs=contribs,
            refs=refs,
            container_id=container_id,
            release_type=obj['type'],
            doi=obj['DOI'].lower(),
            release_date=obj['created']['date-time'],
            issue=obj.get('issue'),
            volume=obj.get('volume'),
            pages=obj.get('page'),
            extra=extra)
        return (we, re, ce)

    def create_row(self, row, editgroup_id=None):
        if row is None:
            return
        obj = json.loads(row)
        entities = self.parse_crossref_dict(obj)
        if entities is not None:
            (we, re, ce) = entities
            we.editgroup_id = editgroup_id
            re.editgroup_id = editgroup_id
            if ce is not None:
                ce.editgroup_id = editgroup_id
                container = self.api.create_container(ce)
                re.container_id = container.ident
                self._issnl_id_map[ce.issnl] = container.ident
            created = self.api.create_work(we)
            re.work_id = created.ident
            self.api.create_release(re)

    def create_batch(self, batch, editgroup_id=None):
        """Current work/release pairing disallows batch creation of releases.
        Could do batch work creation and then match against releases, but meh."""
        for row in batch:
            self.create_row(row, editgroup_id)
