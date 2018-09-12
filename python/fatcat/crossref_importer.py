
import sys
import json
import sqlite3
import itertools
import fatcat_client
from fatcat.importer_common import FatcatImporter


class FatcatCrossrefImporter(FatcatImporter):

    def __init__(self, host_url, issn_map_file, extid_map_file=None, create_containers=True):
        super().__init__(host_url, issn_map_file)
        self.extid_map_db = None
        if extid_map_file:
            db_uri = "file:{}?mode=ro".format(extid_map_file)
            print("Using external ID map: {}".format(db_uri))
            self.extid_map_db = sqlite3.connect(db_uri, uri=True)
        else:
            print("Not using external ID map")
        self.create_containers = create_containers

    def lookup_ext_ids(self, doi):
        if self.extid_map_db is None:
            return dict(core_id=None, pmid=None, pmcid=None, wikidata_qid=None)
        row = self.extid_map_db.execute("SELECT core, pmid, pmcid, wikidata FROM ids WHERE doi=? LIMIT 1",
            [doi.lower()]).fetchone()
        if row is None:
            return dict(core_id=None, pmid=None, pmcid=None, wikidata_qid=None)
        row = [str((cell or None)) for cell in row]
        return dict(
            core_id=row[0],
            pmid=row[1],
            pmcid=row[2],
            wikidata_qid=row[3])

    def parse_crossref_dict(self, obj):
        """
        obj is a python dict (parsed from json).
        returns a ReleaseEntity
        """

        # This work is out of scope if it doesn't have authors or a title
        if (not 'author' in obj) or (not 'title' in obj):
            return None

        # Other ways to be out of scope (provisionally)
        if ((not 'type' in obj) or (not 'container-title' in obj) or
                len(obj['container-title']) < 1):
            return None

        # contribs
        contribs = []
        for i, am in enumerate(obj['author']):
            creator_id = None
            if 'ORCID' in am.keys():
                creator_id = self.lookup_orcid(am['ORCID'].split('/')[-1])
            # Sorry humans :(
            if am.get('given') and am.get('family'):
                raw_name = "{} {}".format(am['given'], am['family'])
            elif am.get('family'):
                raw_name = am['family']
            else:
                # TODO: defaults back to a pseudo-null value
                raw_name = am.get('given', '<blank>')
            contribs.append(fatcat_client.ReleaseContrib(
                creator_id=creator_id,
                index=i+1,
                raw_name=raw_name,
                role="author"))

        # container
        issn = obj.get('ISSN', [None])[0]
        issnl = self.issn2issnl(issn)
        container_id = None
        if issnl:
            container_id = self.lookup_issnl(issnl)
        publisher = obj.get('publisher')

        ce = None
        if container_id is None and self.create_containers and issnl != None:
            ce = fatcat_client.ContainerEntity(
                issnl=issnl,
                publisher=publisher,
                name=obj['container-title'][0])

        # references
        refs = []
        for i, rm in enumerate(obj.get('reference', [])):
            try:
                year = int(rm.get('year'))
                if year > 2025 or year < 1000:
                    # NOTE: will need to update/config in the future!
                    # NOTE: are there crossref works with year < 1000?
                    return None
            except:
                year = None
            extra = dict(crossref=rm)
            if rm.get('DOI') != None:
                extra['doi'] = rm.get('DOI').lower()
            refs.append(fatcat_client.ReleaseRef(
                index=i+1,
                # doing lookups would be a second import pass
                target_release_id=None,
                # unreliable for crossref: key=rm['key'].split('|')[-1],
                year=year,
                container_title=rm.get('volume-title'),
                title=rm.get('title'),
                locator=rm.get('first-page'),
                # TODO: just dump JSON somewhere here?
                extra=dict(crossref=rm)))

        # abstracts
        abstracts = []
        if obj.get('abstract') != None:
            abstracts.append(fatcat_client.ReleaseEntityAbstracts(
                mimetype="application/xml+jats",
                content=obj.get('abstract')))

        # release
        extra = dict(crossref={
            'links': obj.get('link', []),
            'subject': obj.get('subject'),
            'type': obj['type'],
            'license': obj.get('license', [dict(URL=None)])[0]['URL'] or None,
            'alternative-id': obj.get('alternative-id', [])})

        # external identifiers
        extids = self.lookup_ext_ids(doi=obj['DOI'].lower())

        re = fatcat_client.ReleaseEntity(
            work_id=None,
            title=obj['title'][0],
            contribs=contribs,
            refs=refs,
            container_id=container_id,
            release_type=obj['type'],
            doi=obj['DOI'].lower(),
            core_id=extids['core_id'],
            pmid=extids['pmid'],
            pmcid=extids['pmcid'],
            wikidata_qid=extids['wikidata_qid'],
            release_date=obj['created']['date-time'],
            issue=obj.get('issue'),
            volume=obj.get('volume'),
            pages=obj.get('page'),
            abstracts=abstracts,
            extra=extra)
        return (re, ce)

    def create_row(self, row, editgroup=None):
        if row is None:
            return
        obj = json.loads(row)
        entities = self.parse_crossref_dict(obj)
        if entities is not None:
            (re, ce) = entities
            if ce is not None:
                container = self.api.create_container(ce, editgroup=editgroup)
                re.container_id = container.ident
                self._issnl_id_map[ce.issnl] = container.ident
            self.api.create_release(re, editgroup=editgroup)

    def create_batch(self, batch, editgroup=None):
        """Current work/release pairing disallows batch creation of releases.
        Could do batch work creation and then match against releases, but meh."""
        release_batch = []
        for row in batch:
            if row is None:
                continue
            obj = json.loads(row)
            entities = self.parse_crossref_dict(obj)
            if entities is not None:
                (re, ce) = entities
                if ce is not None:
                    container = self.api.create_container(ce, editgroup=editgroup)
                    re.container_id = container.ident
                    self._issnl_id_map[ce.issnl] = container.ident
                release_batch.append(re)
        self.api.create_release_batch(release_batch, autoaccept="true", editgroup=editgroup)
