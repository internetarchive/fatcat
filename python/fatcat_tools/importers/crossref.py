
import sys
import json
import sqlite3
import datetime
import itertools
import subprocess
import fatcat_client
from .common import EntityImporter


# The docs/guide should be the cannonical home for these mappings; update there
# first
CROSSREF_TYPE_MAP = {
    'book': 'book',
    'book-chapter': 'chapter',
    'book-part': 'chapter',
    'book-section': 'chapter',
    'component': None,
    'dataset': 'dataset',
    'dissertation': 'thesis',
    'edited-book': 'book',
    'journal-article': 'article-journal',
    'monograph': 'book',
    'other': None,
    'peer-review': 'peer_review',
    'posted-content': 'post',
    'proceedings-article': 'paper-conference',
    'reference-book': 'book',
    'reference-entry': 'entry',
    'report': 'report',
    'standard': 'standard',
}

CONTAINER_TYPE_MAP = {
    'article-journal': 'journal',
    'paper-conference': 'conference',
    'book': 'book-series',
}

# TODO:
LICENSE_SLUG_MAP = {
    "http://creativecommons.org/licenses/by/3.0/": "CC-BY",
    "http://creativecommons.org/licenses/by/4.0/": "CC-BY",
    "http://creativecommons.org/licenses/by-sa/3.0/": "CC-BY-SA",
    "http://creativecommons.org/licenses/by-sa/4.0/": "CC-BY-SA",
    "http://creativecommons.org/licenses/by-nd/3.0/": "CC-BY-ND",
    "http://creativecommons.org/licenses/by-nd/4.0/": "CC-BY-ND",
    "http://creativecommons.org/licenses/by-nc/3.0/": "CC-BY-NC",
    "http://creativecommons.org/licenses/by-nc/4.0/": "CC-BY-NC",
    "http://creativecommons.org/licenses/by-nc-sa/3.0/": "CC-BY-NC-SA",
    "http://creativecommons.org/licenses/by-nc-sa/4.0/": "CC-BY-NC-SA",
    "http://creativecommons.org/licenses/by-nc-nd/3.0/": "CC-BY-NC-ND",
    "http://creativecommons.org/licenses/by-nc-nd/4.0/": "CC-BY-NC-ND",
    "http://www.elsevier.com/open-access/userlicense/1.0/": "ELSEVIER-USER-1.0",
    # http://onlinelibrary.wiley.com/termsAndConditions doesn't seem like a license
    # http://www.springer.com/tdm doesn't seem like a license
}

class CrossrefImporter(EntityImporter):
    """
    Importer for Crossref metadata.

    Can use a local sqlite3 file for faster "external identifier" lookups

    See https://github.com/CrossRef/rest-api-doc for JSON schema notes
    """

    def __init__(self, api, issn_map_file, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Automated import of Crossref DOI metadata, harvested from REST API")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.CrossrefImporter')
        super().__init__(api,
            issn_map_file=issn_map_file,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra)

        self.create_containers = kwargs.get('create_containers')
        extid_map_file = kwargs.get('extid_map_file')
        self.extid_map_db = None
        if extid_map_file:
            db_uri = "file:{}?mode=ro".format(extid_map_file)
            print("Using external ID map: {}".format(db_uri))
            self.extid_map_db = sqlite3.connect(db_uri, uri=True)
        else:
            print("Not using external ID map")

        self.read_issn_map_file(issn_map_file)

    def lookup_ext_ids(self, doi):
        if self.extid_map_db is None:
            return dict(core_id=None, pmid=None, pmcid=None, wikidata_qid=None, arxiv_id=None, jstor_id=None)
        row = self.extid_map_db.execute("SELECT core, pmid, pmcid, wikidata FROM ids WHERE doi=? LIMIT 1",
            [doi.lower()]).fetchone()
        if row is None:
            return dict(core_id=None, pmid=None, pmcid=None, wikidata_qid=None, arxiv_id=None, jstor_id=None)
        row = [str(cell or '') or None for cell in row]
        return dict(
            core_id=row[0],
            pmid=row[1],
            pmcid=row[2],
            wikidata_qid=row[3],
            # TODO:
            arxiv_id=None,
            jstor_id=None,
        )

    def map_release_type(self, crossref_type):
        return CROSSREF_TYPE_MAP.get(crossref_type)

    def map_container_type(self, crossref_type):
        return CONTAINER_TYPE_MAP.get(release_type)

    def want(self, obj):

        # Ways to be out of scope (provisionally)
        # journal-issue and journal-volume map to None, but allowed for now
        if obj.get('type') in (None, 'journal', 'proceedings',
                'standard-series', 'report-series', 'book-series', 'book-set',
                'book-track', 'proceedings-series'):
            return False

        # Do require the 'title' keys to exsit, as release entities do
        if (not 'title' in obj) or (not obj['title']):
            return False

        # Can't handle such large lists yet
        authors = len(obj.get('author', []))
        abstracts = len(obj.get('abstract', []))
        refs = len(obj.get('reference', []))
        if max(authors, abstracts, refs) > 750:
            return False

        return True

    def parse_record(self, obj):
        """
        obj is a python dict (parsed from json).
        returns a ReleaseEntity
        """

        release_type = self.map_release_type(obj['type'])

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
                raw_affiliation = None
                if am.get('affiliation'):
                    if len(am.get('affiliation')) > 0:
                        raw_affiliation = am.get('affiliation')[0]['name']
                    if len(am.get('affiliation')) > 1:
                        # note: affiliation => affiliations
                        extra['affiliations'] = [a['name'] for a in am.get('affiliation')[1:]]
                if am.get('sequence') and am.get('sequence') != "additional":
                    extra['sequence'] = am.get('sequence')
                if not extra:
                    extra = None
                assert ctype in ("author", "editor", "translator")
                contribs.append(fatcat_client.ReleaseContrib(
                    creator_id=creator_id,
                    index=index,
                    raw_name=raw_name,
                    raw_affiliation=raw_affiliation,
                    role=ctype,
                    extra=extra))
            return contribs
        contribs = do_contribs(obj.get('author', []), "author")
        contribs.extend(do_contribs(obj.get('editor', []), "editor"))
        contribs.extend(do_contribs(obj.get('translator', []), "translator"))

        # container
        issn = obj.get('ISSN', [None])[0]
        issnl = self.issn2issnl(issn)
        container_id = None
        if issnl:
            container_id = self.lookup_issnl(issnl)
        publisher = obj.get('publisher')

        if (container_id is None and self.create_containers and (issnl is not None)
            and obj.get('container-title') and len(obj['container-title']) > 0):
            ce = fatcat_client.ContainerEntity(
                issnl=issnl,
                publisher=publisher,
                container_type=self.map_container_type(release_type),
                name=obj['container-title'][0])
            ce_edit = self.create_container(ce)
            container_id = ce_edit.ident

        # license slug
        license_slug = None
        for l in obj.get('license', []):
            if l['content-version'] not in ('vor', 'unspecified'):
                continue
            slug = LICENSE_SLUG_MAP.get(l['URL'])
            if slug:
                license_slug = slug
                break

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
            ref_locator = rm.get('first-page')
            ref_title = rm.get('title')
            if extra.get('DOI'):
                extra['doi'] = extra['DOI']
            extra.pop('DOI', None)
            extra.pop('key', None)
            extra.pop('year', None)
            extra.pop('volume-title', None)
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
                title=ref_title,
                locator=ref_locator,
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
        assert max(len(contribs), len(refs), len(abstracts)) <= 750

        # release date parsing is amazingly complex
        raw_date = obj['issued']['date-parts'][0]
        if not raw_date or not raw_date[0]:
            # got some NoneType, even though at least year is supposed to be set
            release_year = None
            release_date = None
        elif len(raw_date) == 3:
            release_year = raw_date[0]
            release_date = datetime.date(year=raw_date[0], month=raw_date[1], day=raw_date[2])
        else:
            # sometimes only the year is included, not the full date
            release_year = raw_date[0]
            release_date = None

        re = fatcat_client.ReleaseEntity(
            work_id=None,
            container_id=container_id,
            title=obj.get('title', [None])[0],
            original_title=obj.get('original-title', [None])[0],
            release_type=release_type,
            release_status=release_status,
            release_date=release_date,
            release_year=release_year,
            publisher=publisher,
            doi=obj['DOI'].lower(),
            pmid=extids['pmid'],
            pmcid=extids['pmcid'],
            wikidata_qid=extids['wikidata_qid'],
            isbn13=isbn13,
            core_id=extids['core_id'],
            arxiv_id=extids['arxiv_id'],
            jstor_id=extids['jstor_id'],
            volume=obj.get('volume'),
            issue=obj.get('issue'),
            pages=obj.get('page'),
            language=None,  # crossref doesn't supply language info
            license_slug=license_slug,
            extra=dict(crossref=extra),
            abstracts=abstracts,
            contribs=contribs,
            refs=refs,
        )
        return re

    def try_update(self, re):

        # lookup existing DOI (don't need to try other ext idents for crossref)
        existing = None
        try:
            existing = self.api.lookup_release(doi=re.doi)
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
        self.api.create_release_batch(batch,
            autoaccept=True,
            description=self.editgroup_description,
            extra=json.dumps(self.editgroup_extra))

