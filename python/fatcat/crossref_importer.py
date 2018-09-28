
import sys
import json
import sqlite3
import datetime
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
        row = [str(cell or '') or None for cell in row]
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

        # This work is out of scope if it doesn't have authors and a title
        if (not 'author' in obj) or (not 'title' in obj):
            return None

        # Other ways to be out of scope (provisionally)
        if ((not 'type' in obj) or (not 'container-title' in obj) or
                len(obj['container-title']) < 1):
            return None

        # contribs
        def do_contribs(obj_list, ctype):
            contribs = []
            for i, am in enumerate(obj_list):
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
                extra = dict()
                if ctype == "author":
                    index = i
                else:
                    index = None
                if am.get('affiliation'):
                    # note: affiliation => affiliations
                    extra['affiliations'] = am.get('affiliation')
                if am.get('sequence') and am.get('sequence') != "additional":
                    extra['sequence'] = am.get('sequence')
                if not extra:
                    extra = None
                contribs.append(fatcat_client.ReleaseContrib(
                    creator_id=creator_id,
                    index=index,
                    raw_name=raw_name,
                    role=ctype,
                    extra=extra))
            return contribs
        contribs = do_contribs(obj['author'], "author")
        contribs.extend(do_contribs(obj.get('editor', []), "editor"))
        contribs.extend(do_contribs(obj.get('translator', []), "translator"))

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
                # NOTE: will need to update/config in the future!
                # NOTE: are there crossref works with year < 100?
                if year > 2025 or year < 100:
                    year = None
            except:
                year = None
            extra = rm.copy()
            if rm.get('DOI'):
                extra['doi'] = rm.get('DOI').lower()
            key = rm.get('key')
            if key and key.startswith(obj['DOI'].upper()):
                key = key.replace(obj['DOI'].upper() + "-", '')
                key = key.replace(obj['DOI'].upper(), '')
            container_name = rm.get('volume-title')
            if not container_name:
                container_name = rm.get('journal-title')
            extra.pop('DOI', None)
            extra.pop('key', None)
            extra.pop('year', None)
            extra.pop('volume-name', None)
            extra.pop('journal-title', None)
            extra.pop('title', None)
            extra.pop('first-page', None)
            extra.pop('doi-asserted-by', None)
            if extra:
                extra = dict(crossref=extra)
            else:
                extra = None
            refs.append(fatcat_client.ReleaseRef(
                index=i,
                # doing lookups would be a second import pass
                target_release_id=None,
                key=key,
                year=year,
                container_name=container_name,
                title=rm.get('title'),
                locator=rm.get('first-page'),
                # TODO: just dump JSON somewhere here?
                extra=extra))

        # abstracts
        abstracts = []
        if obj.get('abstract') != None:
            abstracts.append(fatcat_client.ReleaseEntityAbstracts(
                mimetype="application/xml+jats",
                content=obj.get('abstract')))

        # extra fields
        extra = dict()
        for key in ('subject', 'type', 'license', 'alternative-id',
                'container-title', 'original-title', 'subtitle', 'archive',
                'funder', 'group-title'):
            # TODO: unpack "container-title" array
            val = obj.get(key)
            if val:
                extra[key] = val
        if 'license' in extra and extra['license']:
            for i in range(len(extra['license'])):
                if 'start' in extra['license'][i]:
                    extra['license'][i]['start'] = extra['license'][i]['start']['date-time']
        if len(obj['title']) > 1:
            extra['other-titles'] = obj['title'][1:]
        # TODO: this should be top-level
        extra['is_kept'] = len(obj.get('archive', [])) > 0

        # ISBN
        isbn13 = None
        for raw in obj.get('ISBN', []):
            # TODO: convert if not ISBN-13 format
            if len(raw) == 17:
                isbn13 = raw
                break

        # release status
        if obj['type'] in ('journal-article', 'conference-proceeding', 'book',
                'dissertation', 'book-chapter'):
            release_status = "published"
        else:
            # unknown
            release_status = None

        # external identifiers
        extids = self.lookup_ext_ids(doi=obj['DOI'].lower())

        # TODO: filter out huge releases; we'll get them later (and fix bug in
        # fatcatd)
        if max(len(contribs), len(refs), len(abstracts)) > 750:
            return None

        # release date parsing is amazingly complex
        release_date = obj['issued']['date-parts'][0]
        if not release_date or not release_date[0]:
            # got some NoneType, even though at least year is supposed to be set
            release_date = None
        elif len(release_date) == 3:
            release_date = datetime.datetime(year=release_date[0], month=release_date[1], day=release_date[2])
        else:
            # only the year is actually required; mangle to first day for date
            # (TODO: something better?)
            release_date = datetime.datetime(year=release_date[0], month=1, day=1)
        # convert to string ISO datetime format (if not null)
        if release_date:
            release_date = release_date.isoformat() + "Z"

        re = fatcat_client.ReleaseEntity(
            work_id=None,
            title=obj['title'][0],
            contribs=contribs,
            refs=refs,
            container_id=container_id,
            publisher=publisher,
            release_type=obj['type'],
            release_status=release_status,
            doi=obj['DOI'].lower(),
            isbn13=isbn13,
            core_id=extids['core_id'],
            pmid=extids['pmid'],
            pmcid=extids['pmcid'],
            wikidata_qid=extids['wikidata_qid'],
            release_date=release_date,
            issue=obj.get('issue'),
            volume=obj.get('volume'),
            pages=obj.get('page'),
            abstracts=abstracts,
            extra=dict(crossref=extra))
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
            self.insert_count = self.insert_count + 1

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
                    ce_eg = self.api.create_editgroup(
                        fatcat_client.Editgroup(editor_id='aaaaaaaaaaaabkvkaaaaaaaaae'))
                    container = self.api.create_container(ce, editgroup=ce_eg.id)
                    self.api.accept_editgroup(ce_eg.id)
                    re.container_id = container.ident
                    self._issnl_id_map[ce.issnl] = container.ident
                release_batch.append(re)
        self.api.create_release_batch(release_batch, autoaccept="true", editgroup=editgroup)
        self.insert_count = self.insert_count + len(release_batch)
