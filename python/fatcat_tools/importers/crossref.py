
import sys
import json
import sqlite3
import datetime
import itertools
import subprocess
import fatcat_openapi_client
from .common import EntityImporter, clean


# The docs/guide should be the canonical home for these mappings; update there
# first
# Can get a list of Crossref types (with counts) via API:
# https://api.crossref.org/works?rows=0&facet=type-name:*
CROSSREF_TYPE_MAP = {
    'book': 'book',
    'book-chapter': 'chapter',
    'book-part': 'chapter',
    'book-section': 'chapter',
    'component': 'component',
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

# These are based, informally, on sorting the most popular licenses found in
# Crossref metadata. There were over 500 unique strings and only a few most
# popular are here; many were variants of the CC URLs. Would be useful to
# normalize CC licenses better.
# The current norm is to only add license slugs that are at least partially OA.
LICENSE_SLUG_MAP = {
    "//creativecommons.org/publicdomain/mark/1.0": "CC-0",
    "//creativecommons.org/publicdomain/mark/1.0/": "CC-0",
    "//creativecommons.org/publicdomain/mark/1.0/deed.de": "CC-0",
    "//creativecommons.org/publicdomain/mark/1.0/deed.de": "CC-0",
    "//creativecommons.org/publicdomain/zero/1.0/": "CC-0",
    "//creativecommons.org/publicdomain/zero/1.0/legalcode": "CC-0",
    "//creativecommons.org/share-your-work/public-domain/cc0/": "CC-0",
    "//creativecommons.org/licenses/by/2.0/": "CC-BY",
    "//creativecommons.org/licenses/by/3.0/": "CC-BY",
    "//creativecommons.org/licenses/by/4.0/": "CC-BY",
    "//creativecommons.org/licenses/by-sa/3.0/": "CC-BY-SA",
    "//creativecommons.org/licenses/by-sa/4.0/": "CC-BY-SA",
    "//creativecommons.org/licenses/by-nd/3.0/": "CC-BY-ND",
    "//creativecommons.org/licenses/by-nd/4.0/": "CC-BY-ND",
    "//creativecommons.org/licenses/by-nc/3.0/": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc/4.0/": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc-sa/3.0/": "CC-BY-NC-SA",
    "//creativecommons.org/licenses/by-nc-sa/4.0/": "CC-BY-NC-SA",
    "//creativecommons.org/licenses/by-nc-nd/3.0/": "CC-BY-NC-ND",
    "//creativecommons.org/licenses/by-nc-nd/4.0/": "CC-BY-NC-ND",
    "//spdx.org/licenses/CC0-1.0.json": "CC-0",
    "//spdx.org/licenses/CC-BY-1.0.json": "CC-BY",
    "//spdx.org/licenses/CC-BY-4.0.json": "CC-BY",
    "//spdx.org/licenses/CC-BY-NC-4.0.json": "CC-BY-NC",
    "//spdx.org/licenses/CC-BY-SA-3.0.json": "CC-BY-SA",
    "//spdx.org/licenses/CC-BY-SA-4.0.json": "CC-BY-SA",
    "//spdx.org/licenses/MIT.json": "MIT",
    "//spdx.org/licenses/OGL-Canada-2.0.json": "OGL-Canada",
    "//www.elsevier.com/open-access/userlicense/1.0/": "ELSEVIER-USER-1.0",
    "//www.karger.com/Services/SiteLicenses": "KARGER",
    "//pubs.acs.org/page/policy/authorchoice_termsofuse.html": "ACS-CHOICE",
    "//pubs.acs.org/page/policy/authorchoice_ccby_termsofuse.html": "CC-BY",
    "//www.biologists.com/user-licence-1-1/": "BIOLOGISTS-USER",
    "//www.biologists.com/user-licence-1-1": "BIOLOGISTS-USER",
    "//www.apa.org/pubs/journals/resources/open-access.aspx": "APA",
    "//www.ametsoc.org/PUBSReuseLicenses": "AMETSOC",
    # //onlinelibrary.wiley.com/termsAndConditions doesn't seem like a license
    # //www.springer.com/tdm doesn't seem like a license
    # //iopscience.iop.org/page/copyright is closed
    # //www.acm.org/publications/policies/copyright_policy#Background is closed
    # //rsc.li/journals-terms-of-use is closed for vor (am open)
    # //www.ieee.org/publications_standards/publications/rights/ieeecopyrightform.pdf is 404 (!)
    "//arxiv.org/licenses/nonexclusive-distrib/1.0/": "ARXIV-1.0",
}

def lookup_license_slug(raw):
    if not raw:
        return None
    raw = raw.strip().replace('http://', '//').replace('https://', '//')
    if 'creativecommons.org' in raw.lower():
        raw = raw.lower()
        raw = raw.replace('/legalcode', '/').replace('/uk', '')
        if not raw.endswith('/'):
            raw = raw + '/'
    return LICENSE_SLUG_MAP.get(raw)

def test_lookup_license_slug():

    assert lookup_license_slug("https://creativecommons.org/licenses/by-nc/3.0/") == "CC-BY-NC"
    assert lookup_license_slug("http://creativecommons.org/licenses/by/2.0/uk/legalcode") == "CC-BY"
    assert lookup_license_slug("https://creativecommons.org/publicdomain/zero/1.0/legalcode") == "CC-0"
    assert lookup_license_slug("http://creativecommons.org/licenses/by/4.0") == "CC-BY"
    assert lookup_license_slug("https://creativecommons.org/licenses/by-nc-sa/4.0/") == "CC-BY-NC-SA"
    assert lookup_license_slug("https://www.ametsoc.org/PUBSReuseLicenses") == "AMETSOC"
    assert lookup_license_slug("https://www.amec.org/PUBSReuseLicenses") is None
    assert lookup_license_slug("") is None
    assert lookup_license_slug(None) is None

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
            editgroup_extra=eg_extra,
            **kwargs)

        self.create_containers = kwargs.get('create_containers', True)
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
        return CONTAINER_TYPE_MAP.get(crossref_type)

    def want(self, obj):
        if not obj.get('title'):
            self.counts['skip-blank-title'] += 1
            return False

        # these are pre-registered DOIs before the actual record is ready
        # title is a list of titles
        if obj.get('title')[0].strip().lower() in [
                "OUP accepted manuscript".lower(),
            ]:
            self.counts['skip-stub-title'] += 1
            return False

        # do most of these checks in-line below
        return True

    def parse_record(self, obj):
        """
        obj is a python dict (parsed from json).
        returns a ReleaseEntity
        """

        # Ways to be out of scope (provisionally)
        # journal-issue and journal-volume map to None, but allowed for now
        if obj.get('type') in (None, 'journal', 'proceedings',
                'standard-series', 'report-series', 'book-series', 'book-set',
                'book-track', 'proceedings-series'):
            self.counts['skip-release-type'] += 1
            return None

        # Do require the 'title' keys to exist, as release entities do
        if (not 'title' in obj) or (not obj['title']):
            self.counts['skip-blank-title'] += 1
            return None

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
                    # TODO: can end up empty
                    raw_name = am.get('name') or am.get('given')
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
                        # note: affiliation => more_affiliations
                        extra['more_affiliations'] = [clean(a['name']) for a in am.get('affiliation')[1:]]
                if am.get('sequence') and am.get('sequence') != "additional":
                    extra['seq'] = clean(am.get('sequence'))
                if not extra:
                    extra = None
                assert ctype in ("author", "editor", "translator")
                raw_name = clean(raw_name)
                contribs.append(fatcat_openapi_client.ReleaseContrib(
                    creator_id=creator_id,
                    index=index,
                    raw_name=raw_name,
                    given_name=clean(am.get('given')),
                    surname=clean(am.get('family')),
                    raw_affiliation=clean(raw_affiliation),
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
        publisher = clean(obj.get('publisher'))

        container_name = obj.get('container-title')
        if container_name:
            container_name = clean(container_name[0], force_xml=True)
        if not container_name:
            container_name = None
        if (container_id is None and self.create_containers and (issnl is not None)
                and container_name):
            ce = fatcat_openapi_client.ContainerEntity(
                issnl=issnl,
                publisher=publisher,
                container_type=self.map_container_type(release_type),
                name=container_name)
            ce_edit = self.create_container(ce)
            container_id = ce_edit.ident
            self._issnl_id_map[issnl] = container_id

        # license slug
        license_slug = None
        license_extra = []
        for l in obj.get('license', []):
            if l['content-version'] not in ('vor', 'unspecified'):
                continue
            slug = lookup_license_slug(l['URL'])
            if slug:
                license_slug = slug
            if 'start' in l:
                l['start'] = l['start']['date-time']
            license_extra.append(l)

        # references
        refs = []
        for i, rm in enumerate(obj.get('reference', [])):
            try:
                year = int(rm.get('year'))
                # TODO: will need to update/config in the future!
                # NOTE: are there crossref works with year < 100?
                if year > 2025 or year < 100:
                    year = None
            except:
                year = None
            ref_extra = dict()
            key = rm.get('key')
            if key and key.startswith(obj['DOI'].upper()):
                key = key.replace(obj['DOI'].upper() + "-", '')
                key = key.replace(obj['DOI'].upper(), '')
            ref_container_name = rm.get('volume-title')
            if not ref_container_name:
                ref_container_name = rm.get('journal-title')
            elif rm.get('journal-title'):
                ref_extra['journal-title'] = rm['journal-title']
            if rm.get('DOI'):
                ref_extra['doi'] = rm.get('DOI').lower()
            author = clean(rm.get('author'))
            if author:
                ref_extra['authors'] = [author]
            for k in ('editor', 'edition', 'authority', 'version', 'genre',
                    'url', 'event', 'issue', 'volume', 'date', 'accessed_date',
                    'issued', 'page', 'medium', 'collection_title', 'chapter_number',
                    'unstructured', 'series-title', 'volume-title'):
                if clean(rm.get(k)):
                    ref_extra[k] = clean(rm[k])
            if not ref_extra:
                ref_extra = None
            refs.append(fatcat_openapi_client.ReleaseRef(
                index=i,
                # doing lookups would be a second import pass
                target_release_id=None,
                key=key,
                year=year,
                container_name=clean(ref_container_name),
                title=clean(rm.get('article-title')),
                locator=clean(rm.get('first-page')),
                # TODO: just dump JSON somewhere here?
                extra=ref_extra))

        # abstracts
        abstracts = []
        abstract = clean(obj.get('abstract'))
        if abstract and len(abstract) > 10:
            abstracts.append(fatcat_openapi_client.ReleaseAbstract(
                mimetype="application/xml+jats",
                content=abstract))

        # extra fields
        extra = dict()
        extra_crossref = dict()
        # top-level extra keys
        if not container_id:
            if obj.get('container-title'):
                extra['container_name'] = container_name
        for key in ('group-title'):
            val = obj.get(key)
            if val:
                if type(val) == list:
                    val = val[0]
                if type(val) == str:
                    val = clean(val)
                    if val:
                        extra[key] = clean(val)
                else:
                    extra[key] = val
        # crossref-nested extra keys
        for key in ('subject', 'type', 'alternative-id', 'archive', 'funder'):
            val = obj.get(key)
            if val:
                if type(val) == str:
                    extra_crossref[key] = clean(val)
                else:
                    extra_crossref[key] = val
        if license_extra:
            extra_crossref['license'] = license_extra

        if len(obj['title']) > 1:
            aliases = [clean(t) for t in obj['title'][1:]]
            aliases = [t for t in aliases if t]
            if aliases:
                extra['aliases'] = aliases

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
            release_stage = "published"
        else:
            # unknown
            release_stage = None

        # external identifiers
        extids = self.lookup_ext_ids(doi=obj['DOI'].lower())

        # filter out unreasonably huge releases
        if len(abstracts) > 100:
            self.counts['skip-huge-abstracts'] += 1
            return None
        if len(contribs) > 2000:
            self.counts['skip-huge-contribs'] += 1
            return None
        if len(refs) > 5000:
            self.counts['skip-huge-refs'] += 1
            return None

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


        original_title = None
        if obj.get('original-title'):
            original_title = clean(obj.get('original-title')[0], force_xml=True)

        title = None
        if obj.get('title'):
            title = clean(obj.get('title')[0], force_xml=True)
            if not title or len(title) <= 1:
                # title can't be just a single character
                self.counts['skip-blank-title'] += 1
                return None

        subtitle = None
        if obj.get('subtitle'):
            subtitle = clean(obj.get('subtitle')[0], force_xml=True)
            if not subtitle or len(subtitle) <= 1:
                # subtitle can't be just a single character
                subtitle = None

        if extra_crossref:
            extra['crossref'] = extra_crossref
        if not extra:
            extra = None

        re = fatcat_openapi_client.ReleaseEntity(
            work_id=None,
            container_id=container_id,
            title=title,
            subtitle=subtitle,
            original_title=original_title,
            release_type=release_type,
            release_stage=release_stage,
            release_date=release_date,
            release_year=release_year,
            publisher=publisher,
            ext_ids=fatcat_openapi_client.ReleaseExtIds(
                doi=obj['DOI'].lower(),
                pmid=extids['pmid'],
                pmcid=extids['pmcid'],
                wikidata_qid=extids['wikidata_qid'],
                isbn13=isbn13,
                core=extids['core_id'],
                arxiv=extids['arxiv_id'],
                jstor=extids['jstor_id'],
            ),
            volume=clean(obj.get('volume')),
            issue=clean(obj.get('issue')),
            pages=clean(obj.get('page')),
            language=clean(obj.get('language')),
            license_slug=license_slug,
            extra=extra,
            abstracts=abstracts,
            contribs=contribs,
            refs=refs,
        )
        return re

    def try_update(self, re):

        # lookup existing DOI (don't need to try other ext idents for crossref)
        existing = None
        try:
            existing = self.api.lookup_release(doi=re.ext_ids.doi)
        except fatcat_openapi_client.rest.ApiException as err:
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
        self.api.create_release_auto_batch(fatcat_openapi_client.ReleaseAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))

