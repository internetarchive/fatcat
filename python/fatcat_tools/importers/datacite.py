"""
Prototype importer for datacite.org data.

Example input document at: https://gist.github.com/miku/5610a2d64e3fee82d16f5d3f3a295fc8.

Datacite being an aggregator, the data is varied and exposes a couple of
problems in content and structure. A few fields habe their own parsing
functions (parse_datacite_...), which can be tested more easily.
"""

from .common import EntityImporter, clean
import dateparser
import datetime
import fatcat_openapi_client
import hashlib
import json
import pycountry
import langdetect
import sqlite3
import sys
from fatcat_tools.transforms import entity_to_dict


# Cutoff length for abstracts.
MAX_ABSTRACT_LENGTH = 2048

# https://guide.fatcat.wiki/entity_container.html#container_type-vocabulary
CONTAINER_TYPE_MAP = {
    'Journal': 'journal',
    'Series': 'journal',
    'Book Series': 'book-series',
}

# The docs/guide should be the cannonical home for these mappings; update there
# first.  Map various datacite type types to CSL-ish types. None means TODO or
# remove.
DATACITE_TYPE_MAP = {
    'ris': {
        'THES': 'thesis',
        'SOUND': None,
        'CHAP': 'chapter',
        'FIGURE': None,
        'RPRT': 'report',
        'JOUR': 'article-journal',
        'MPCT': None,
        'GEN': None,
        'BOOK': 'book',
        'DATA': 'dataset',
        'COMP': None,
    },
    'schemaOrg': {
        'Dataset': 'dataset',
        'Book': 'book',
        'ScholarlyArticle': 'article',
        'ImageObject': 'graphic',
        'Collection': None,
        'MediaObject': None,
        'Event': None,
        'SoftwareSourceCode': None,
        'Chapter': 'chapter',
        'CreativeWork': None,
        'PublicationIssue': 'article',
        'AudioObject': None,
        'Thesis': 'thesis',
    },
    'citeproc': {
        'article': 'article',
        'article-journal': 'article-journal',
        'article-magazine': 'article-magazine',
        'article-newspaper': 'article-newspaper',
        'bill': 'bill',
        'book': 'book',
        'broadcast': 'broadcast',
        'chapter': 'chapter',
        'dataset': 'dataset',
        'entry-dictionary': 'entry-dictionary',
        'entry-encyclopedia': 'entry-encyclopedia',
        'entry': 'entry',
        'figure': 'figure',
        'graphic': 'graphic',
        'interview': 'interview',
        'legal_case': 'legal_case',
        'legislation': 'legislation',
        'manuscript': 'manuscript',
        'map': 'map',
        'motion_picture': 'motion_picture',
        'musical_score': 'musical_score',
        'pamphlet': 'pamphlet',
        'paper-conference': 'paper-conference',
        'patent': 'patent',
        'personal_communication': 'personal_communication',
        'post': 'post',
        'post-weblog': 'post-weblog',
        'report': 'report',
        'review-book': 'review-book',
        'review': 'review',
        'song': 'song',
        'speech': 'speech',
        'thesis': 'thesis',
        'treaty': 'treaty',
        'webpage': 'webpage',
    },  # https://docs.citationstyles.org/en/master/specification.html#appendix-iii-types
    'bibtex': {
        'phdthesis': 'thesis',
        'inbook': 'chapter',
        'misc': None,
        'article': 'article-journal',
        'book': 'book',
    },
    'resourceTypeGeneral': {
        'Image': None,
        'Dataset': 'dataset',
        'PhysicalObject': None,
        'Collection': None,
        'Text': None,
        'Sound': None,
        'InteractiveResource': None,
        'Event': None,
        'Software': None,
        'Other': None,
        'Workflow': None,
        'Audiovisual': None,
    }
}

# TODO(martin): merge this with other maps, maybe.
LICENSE_SLUG_MAP = {
    "//creativecommons.org/licenses/by/2.0/": "CC-BY",
    "//creativecommons.org/licenses/by/2.0/uk/legalcode": "CC-BY",
    "//creativecommons.org/licenses/by/3.0/": "CC-BY",
    "//creativecommons.org/licenses/by/3.0/us": "CC-BY",
    "//creativecommons.org/licenses/by/4.0/": "CC-BY",
    "//creativecommons.org/licenses/by/4.0/deed.de/": "CC-BY",
    "//creativecommons.org/licenses/by/4.0/deed.en_US/": "CC-BY",
    "//creativecommons.org/licenses/by/4.0/legalcode/": "CC-BY",
    "//creativecommons.org/licenses/by-nc/2.0/": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc/3.0/": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc/4.0/": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc/4.0/legalcode": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc-nd/3.0/": "CC-BY-NC-ND",
    "//creativecommons.org/licenses/by-nc-nd/3.0/gr": "CC-BY-NC-ND",
    "//creativecommons.org/licenses/by-nc-nd/4.0/": "CC-BY-ND",
    "//creativecommons.org/licenses/by-nc-nd/4.0/legalcode": "CC-BY-ND",
    "//creativecommons.org/licenses/by-nc-sa/4.0/": "CC-BY-NC-SA",
    "//creativecommons.org/licenses/by-nd/4.0/": "CC-BY-ND",
    "//creativecommons.org/licenses/by-sa/3.0/de": "CC-BY-SA",
    "//creativecommons.org/licenses/by-sa/3.0/gr": "CC-BY-SA",
    "//creativecommons.org/licenses/by-sa/4.0/": "CC-BY-SA",
    "//creativecommons.org/licenses/by-sa/4.0/legalcode": "CC-BY-SA",
    "//creativecommons.org/licenses/CC-BY/4.0/": "CC-BY",
    "//creativecommons.org/licenses/publicdomain/zero/1.0/": "CC-0",
    "//creativecommons.org/publicdomain/zero/1.0/": "CC-0",
    "//creativecommons.org/publicdomain/zero/1.0/legalcode": "CC-0",
    "//opensource.org/licenses/MIT": "MIT",
    "//www.elsevier.com/open-access/userlicense/1.0": "ELSEVIER-USER-1.0",
    "//www.gnu.org/licenses/gpl-3.0.en.html": "GPLv3",
    "//www.gnu.org/licenses/old-licenses/gpl-2.0.en.html": "GPLv2",
    "//www.karger.com/Services/SiteLicenses": "KARGER",
    "//www.opensource.org/licenses/Apache-2.0": "Apache-2.0",
    "//www.opensource.org/licenses/BSD-3-Clause": "BSD-3-Clause",
    "//www.opensource.org/licenses/EUPL-1.1":
    "EUPL-1.1",  # redirects to EUPL-1.2
    "//www.opensource.org/licenses/MIT": "MIT",
    # "http://royalsocietypublishing.org/licence": "", # OA and "normal", https://royalsociety.org/journals/authors/licence-to-publish/
    # "http://rsc.li/journals-terms-of-use": "RSC",
    # "http://www.fu-berlin.de/sites/refubium/rechtliches/Nutzungsbedingungen": "", # 53 UrhG.
    # "http://www.nrcresearchpress.com/page/about/CorporateTextAndDataMining": "",
    # "http://www.springer.com/tdm": "",
    # "https://cds.unistra.fr/vizier-org/licences_vizier.html": "", # Maybe try to "SPN" those: https://web.archive.org/web/*/https://cds.unistra.fr/vizier-org/licences_vizier.html
    # "https://link.aps.org/licenses/aps-default-accepted-manuscript-license": "",
    # "https://oparu.uni-ulm.de/xmlui/license_opod_v1": "",
    # "https://publikationen.bibliothek.kit.edu/kitopen-lizenz": "",
    # "https://rightsstatements.org/page/InC/1.0?language=en": "",
    # "https://services.ceda.ac.uk/cedasite/register/info": "",
    # "https://wdc.dlr.de/ndmc/userfiles/file/NDMC-Data_Sharing_Principles.pdf": "", # 404
    # "https://www.cambridge.org/core/terms": "",
    # "https://www.elsevier.com/tdm/userlicense/1.0",
    # "info:eu-repo/semantics/closedAccess": "", # https://wiki.surfnet.nl/display/standards/info-eu-repo/#info-eu-repo-AccessRights
    # "info:eu-repo/semantics/embargoedAccess": "",
    # "info:eu-repo/semantics/openAccess": "",
    # Note: Some URLs pointing to licensing terms are not in WB yet (but would be nice).
}

# TODO(martin): drop this after 3.7 upgrade
try:
    isascii = str.isascii # new in 3.7, https://docs.python.org/3/library/stdtypes.html#str.isascii
except AttributeError:
    isascii = lambda s: len(s) == len(s.encode())


class DataciteImporter(EntityImporter):
    """
    Importer for datacite records.
    """
    def __init__(self,
                 api,
                 issn_map_file,
                 debug=False,
                 lang_detect=False,
                 insert_log_file=None,
                 **kwargs):

        eg_desc = kwargs.get(
            'editgroup_description',
            "Automated import of Datacite DOI metadata, harvested from REST API"
        )
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent',
                                         'fatcat_tools.DataciteImporter')
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
            print("Using external ID map: {}".format(db_uri), file=sys.stderr)
            self.extid_map_db = sqlite3.connect(db_uri, uri=True)
        else:
            print("Not using external ID map", file=sys.stderr)

        self.read_issn_map_file(issn_map_file)
        self.debug = debug
        self.lang_detect = lang_detect
        self.insert_log_file = insert_log_file

        print('datacite with debug={}, lang_detect={}'.format(
            self.debug, self.lang_detect),
              file=sys.stderr)

    def lookup_ext_ids(self, doi):
        """
        Return dictionary of identifiers refering to the same things as the given DOI.
        """
        if self.extid_map_db is None:
            return dict(core_id=None,
                        pmid=None,
                        pmcid=None,
                        wikidata_qid=None,
                        arxiv_id=None,
                        jstor_id=None)
        row = self.extid_map_db.execute(
            "SELECT core, pmid, pmcid, wikidata FROM ids WHERE doi=? LIMIT 1",
            [doi.lower()]).fetchone()
        if row is None:
            return dict(core_id=None,
                        pmid=None,
                        pmcid=None,
                        wikidata_qid=None,
                        arxiv_id=None,
                        jstor_id=None)
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

    def parse_record(self, obj):
        """
        Mapping datacite JSON to ReleaseEntity.
        """
        if not obj or not isinstance(obj, dict):
            return None
        if 'attributes' not in obj:
            return None

        attributes = obj['attributes']
        doi = clean_doi(attributes.get('doi', '').lower())

        if not isascii(doi):
            print('[{}] skipping non-ascii doi for now'.format(doi))
            return None

        # Contributors. Many nameIdentifierSchemes, we do not use (yet):
        # "attributes.creators[].nameIdentifiers[].nameIdentifierScheme":
        # ["LCNA", "GND", "email", "NAF", "OSF", "RRID", "ORCID",
        # "SCOPUS", "NRCPID", "schema.org", "GRID", "MGDS", "VIAF", "JACoW-ID"].
        contribs = []

        for i, c in enumerate(attributes['creators']):
            nameType = c.get('nameType', '') or ''
            if nameType == 'Personal' or nameType == '':
                creator_id = None
                for nid in c.get('nameIdentifiers', []):
                    name_scheme = nid.get('nameIdentifierScheme', '') or ''
                    if not name_scheme.lower() == "orcid":
                        continue
                    orcid = nid.get('nameIdentifier',
                                    '').replace('https://orcid.org/', '')
                    if not orcid:
                        continue
                    creator_id = self.lookup_orcid(orcid)
                    # TODO(martin): If creator_id is None, should we create creators?

                # If there are multiple affiliation strings, use the first one.
                affiliations = c.get('affiliation', []) or []
                raw_affiliation = None
                if len(affiliations) == 0:
                    raw_affiliation = None
                else:
                    raw_affiliation = clean(affiliations[0])

                name = c.get('name')
                given_name = c.get('givenName')
                surname = c.get('familyName')

                if name:
                    name = clean(name)

                if given_name:
                    given_name = clean(given_name)

                if surname:
                    surname = clean(surname)

                if not name:
                    continue

                if raw_affiliation is not None and not raw_affiliation:
                    continue

                contribs.append(
                    fatcat_openapi_client.ReleaseContrib(
                        creator_id=creator_id,
                        index=i,
                        raw_name=name,
                        given_name=given_name,
                        surname=surname,
                        role='author',
                        raw_affiliation=raw_affiliation,
                    ))
            elif nameType == 'Organizational':
                name = c.get('name', '') or ''
                if name == 'NN':
                    continue
                if len(name) < 3:
                    continue
                extra = {'organization': name}
                contribs.append(fatcat_openapi_client.ReleaseContrib(
                    index=i, extra=extra))
            else:
                print('[{}] unknown name type: {}'.format(doi, nameType), file=sys.stderr)

        # Title, may come with "attributes.titles[].titleType", like
        # "AlternativeTitle", "Other", "Subtitle", "TranslatedTitle"
        titles = attributes.get('titles', []) or []
        title, original_language_title, subtitle = parse_datacite_titles(
            titles)

        if title is None:
            print('[{}] skipping record w/o title: {}'.format(doi, obj), file=sys.stderr)
            return False

        title = clean(title)
        if not title:
            print('[{}] skipping record w/o title: {}'.format(doi, obj), file=sys.stderr)
            return False

        if not subtitle:
            subtitle = None
        else:
            subtitle = clean(subtitle)

        # Dates. A few internal dates (registered, created, updated) and
        # published (0..2554). We try to work with typed date list, in
        # "attributes.dates[].dateType", values: "Accepted", "Available"
        # "Collected", "Copyrighted", "Created", "Issued", "Submitted",
        # "Updated", "Valid".
        release_date, release_year = parse_datacite_dates(
            attributes.get('dates', []))

        # Start with clear stages, e.g. published. TODO(martin): we could
        # probably infer a bit more from the relations, e.g.
        # "IsPreviousVersionOf" or "IsNewVersionOf".
        release_stage = None
        if attributes.get(
                'state') == 'findable' or attributes.get('isActive') is True:
            release_stage = 'published'

        # Publisher. A few NA values. A few bogus values.
        publisher = attributes.get('publisher')

        if publisher in ('(:unav)', 'Unknown', 'n.a.', '[s.n.]', '(:unap)',
                         '(:none)', 'Unpublished'):
            publisher = None
            release_stage = None
        if publisher is not None and len(publisher) > 80:
            # Arbitrary magic value max length. TODO(martin): better heuristic,
            # but factored out; first we have to log misses. Example:
            # "ETH-Bibliothek Zürich, Bildarchiv / Fotograf: Feller,
            # Elisabeth, Empfänger, Unbekannt, Fotograf / Fel_041033-RE /
            # Unbekannt, Nutzungsrechte müssen durch den Nutzer abgeklärt
            # werden"
            publisher = None

        if publisher:
            publisher = clean(publisher)

        # Container. For the moment, only ISSN as container.
        container_id = None
        container_name = None

        container = attributes.get('container', {}) or {}
        if container.get('type') in CONTAINER_TYPE_MAP.keys():
            container_type = CONTAINER_TYPE_MAP.get(container['type'])
            if container.get('identifier') and container.get(
                    'identifierType') == 'ISSN':
                issn = container.get('identifier')
                if len(issn) == 8:
                    issn = issn[:4] + "-" + issn[4:]
                issnl = self.issn2issnl(issn)
                if issnl is not None:
                    container_id = self.lookup_issnl(issnl)

                    if container_id is None and container.get('title'):
                        container_name = container.get('title')
                        if isinstance(container_name, list):
                            if len(container_name) > 0:
                                print('[{}] too many container titles: {}'.format(doi,
                                    len(container_name)))
                                container_name = container_name[0]
                        assert isinstance(container_name, str)
                        ce = fatcat_openapi_client.ContainerEntity(
                            issnl=issnl,
                            container_type=container_type,
                            name=container_name,
                        )
                        ce_edit = self.create_container(ce)
                        container_id = ce_edit.ident
                        self._issnl_id_map[issnl] = container_id
                else:
                    # TODO(martin): factor this out into a testable function.
                    # TODO(martin): "container_name": "№1(1) (2018)" / 10.26087/inasan.2018.1.1.013
                    container_name = container.get('title')
                    if isinstance(container_name, list):
                        if len(container_name) > 0:
                            print('[{}] too many container titles: {}'.format(doi,
                                len(container_name)))
                            container_name = container_name[0]

        # Volume and issue.
        volume = container.get('volume')
        issue = container.get('issue')

        if volume:
            volume = clean(volume)

        if issue:
            issue = clean(issue)

        # Pages.
        pages = None

        first_page = container.get('firstPage')
        last_page = container.get('lastPage')

        if first_page and last_page:
            try:
                int(first_page) < int(last_page)
                pages = '{}-{}'.format(first_page, last_page)
            except ValueError as err:
                # TODO(martin): This is more debug than info.
                # print('[{}] {}'.format(doi, err), file=sys.stderr)
                pass

        if not pages and first_page:
            pages = first_page

        # License.
        license_slug = None
        license_extra = []

        for l in attributes.get('rightsList', []):
            slug = lookup_license_slug(l.get('rightsUri'))
            if slug:
                license_slug = slug
            license_extra.append(l)

        # Release type. Try to determine the release type from a variety of
        # types supplied in datacite. The "attributes.types.resourceType"
        # contains too many (176 in sample) things for now; citeproc may be the
        # closest, but not always supplied.
        for typeType in ('citeproc', 'resourceTypeGeneral', 'schemaOrg',
                         'bibtex', 'ris'):
            value = attributes.get('types', {}).get(typeType)
            release_type = DATACITE_TYPE_MAP.get(typeType, {}).get(value)
            if release_type is not None:
                break

        if release_type is None:
            print("[{}] no mapped type: {}".format(doi, value), file=sys.stderr)

        # Language values are varied ("ger", "es", "English", "ENG", "en-us",
        # "other", ...). Try to crush it with langcodes: "It may sound to you
        # like langcodes solves a pretty boring problem. At one level, that's
        # right. Sometimes you have a boring problem, and it's great when a
        # library solves it for you." -- TODO(martin): We need more of these.
        language = None

        value = attributes.get('language', '') or ''
        try:
            language = pycountry.languages.lookup(value).alpha_2
        except (LookupError, AttributeError) as err:
            pass
            # TODO(martin): Print this on debug level, only.
            # print('[{}] language lookup miss for {}: {}'.format(doi, value, err), file=sys.stderr)

        # Abstracts appear in "attributes.descriptions[].descriptionType", some
        # of the observed values: "Methods", "TechnicalInfo",
        # "SeriesInformation", "Other", "TableOfContents", "Abstract". The
        # "Other" fields might contain references or related articles (with
        # DOI). TODO(martin): maybe try to parse out some of those refs.
        abstracts = []
        descs = attributes.get('descriptions', []) or []
        for desc in descs:
            if not desc.get('descriptionType') == 'Abstract':
                continue
            if len(desc.get('description', '') or '') < 10:
                continue
            text = desc.get('description', '')
            if len(text) > MAX_ABSTRACT_LENGTH:
                text = text[:MAX_ABSTRACT_LENGTH] + " [...]"
            lang = None
            if self.lang_detect:
                try:
                    lang = langdetect.detect(text)
                except langdetect.lang_detect_exception.LangDetectException as err:
                    print('[{}] language detection failed: {}'.format(doi, err),
                          file=sys.stderr)
            abstracts.append(
                fatcat_openapi_client.ReleaseAbstract(
                    mimetype="text/plain",
                    content=text,
                    lang=lang,
                ))

        # References and relations. Datacite include many relation types in
        # "attributes.relatedIdentifiers[].relationType", e.g.
        # "IsPartOf", "IsPreviousVersionOf", "Continues", "IsVariantFormOf",
        # "IsSupplementTo", "Cites", "IsSupplementedBy", "IsDocumentedBy", "HasVersion",
        # "IsCitedBy", "IsMetadataFor", "IsNewVersionOf", "IsIdenticalTo", "HasPart",
        # "References", "Reviews", "HasMetadata", "IsContinuedBy", "IsVersionOf",
        # "IsDerivedFrom", "IsSourceOf".
        #
        # For the moment, we only care about References.
        refs, ref_index = [], 0

        relIds = attributes.get('relatedIdentifiers', []) or []
        for rel in relIds:
            if not rel.get('relationType', '') == 'References':
                continue
            ref_extra = dict()
            if rel.get('relatedIdentifierType', '') == 'DOI':
                ref_extra['doi'] = rel.get('relatedIdentifier')
            if not ref_extra:
                ref_extra = None
            refs.append(
                fatcat_openapi_client.ReleaseRef(
                    index=ref_index,
                    extra=ref_extra,
                ))
            ref_index += 1

        # Extra information.
        extra_datacite = dict()

        if license_extra:
            extra_datacite['license'] = license_extra
        if attributes.get('subjects'):
            extra_datacite['subjects'] = attributes['subjects']

        # Include certain relations from relatedIdentifiers. Keeping the
        # original structure of data here, which is a list of dicts, with
        # relation type, identifer and identifier type (mostly).
        relations = []
        for rel in relIds:
            if rel.get('relationType') in ('IsPartOf', 'Reviews', 'Continues',
                                           'IsVariantFormOf', 'IsSupplementTo',
                                           'HasVersion', 'IsMetadataFor',
                                           'IsNewVersionOf', 'IsIdenticalTo',
                                           'IsVersionOf', 'IsDerivedFrom',
                                           'IsSourceOf'):
                relations.append(rel)

        if relations:
            extra_datacite['relations'] = relations

        extra = dict()

        # top-level extra keys
        if not container_id and container_name:
            extra['container_name'] = container_name

        if extra_datacite:
            extra['datacite'] = extra_datacite

        extids = self.lookup_ext_ids(doi=doi)

        # Assemble release.
        re = fatcat_openapi_client.ReleaseEntity(
            work_id=None,
            container_id=container_id,
            release_type=release_type,
            release_stage=release_stage,
            title=title,
            subtitle=subtitle,
            original_title=original_language_title,
            release_year=release_year,
            release_date=release_date,
            publisher=publisher,
            ext_ids=fatcat_openapi_client.ReleaseExtIds(
                doi=doi,
                pmid=extids['pmid'],
                pmcid=extids['pmcid'],
                wikidata_qid=extids['wikidata_qid'],
                core=extids['core_id'],
                arxiv=extids['arxiv_id'],
                jstor=extids['jstor_id'],
            ),
            contribs=contribs,
            volume=volume,
            issue=issue,
            pages=pages,
            language=language,
            abstracts=abstracts,
            refs=refs,
            extra=extra,
            license_slug=license_slug,
        )
        return re

    def try_update(self, re):
        """
        When debug is true, write the RE to stdout, not to the database. Might
        hide schema mismatch bugs.
        """
        if self.debug is True:
            print(json.dumps(entity_to_dict(re, api_client=None)))
            return False

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
        print('inserting batch ({})'.format(len(batch)), file=sys.stderr)
        if self.insert_log_file:
            with open(self.insert_log_file, 'a') as f:
                for doc in batch:
                    json.dump(entity_to_dict(re, api_client=None), f)
                    f.write('\n')
        self.api.create_release_auto_batch(
            fatcat_openapi_client.ReleaseAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description,
                    extra=self.editgroup_extra),
                entity_list=batch))


def lookup_license_slug(raw):
    """
    TODO(martin): reuse from or combine with crossref, maybe.
    """
    if not raw:
        return None
    raw = raw.strip().replace('http://', '//').replace('https://', '//')
    if 'creativecommons.org' in raw.lower():
        raw = raw.lower()
        raw = raw.replace('/legalcode', '/').replace('/uk', '')
        if not raw.endswith('/'):
            raw = raw + '/'
    return LICENSE_SLUG_MAP.get(raw)


def find_original_language_title(item, min_length=4, max_questionmarks=3):
    """
    Perform a few checks before returning a potential original language title.

    Example input: {'title': 'Some title', 'original_language_title': 'Some title'}
    """
    if not 'original_language_title' in item:
        return None
    title = item.get('title')
    if not title:
        return None
    original_language_title = item.get('original_language_title')
    if isinstance(original_language_title,
                  str) and title != original_language_title:
        if len(original_language_title) < min_length:
            return None
        if original_language_title.count('?') > max_questionmarks:
            return None
        return original_language_title
    if isinstance(original_language_title, dict):
        content = original_language_title.get('__content__', '') or ''
        if content and content != title and not content.count(
                '?') > max_questionmarks:
            return content
    return None


def parse_datacite_titles(titles):
    """
    Given a list of title items from datacite, return 3-tuple (title,
    original_language_title, subtitle).

    Example input: [{"title": "Meeting Heterogeneity in Consumer Demand"}]
    """
    title, original_language_title, subtitle = None, None, None

    if titles is None:
        return title, original_language_title, subtitle
    if len(titles) == 0:
        return title, original_language_title, subtitle
    elif len(titles) == 1:
        original_language_title = find_original_language_title(titles[0])
        title = titles[0].get('title', '') or ''
        title = title.strip()
        if not title:
            title = None
        return title, original_language_title, subtitle
    else:
        for entry in titles:
            if not title and ('titleType' not in entry
                              or not entry.get('titleType')):
                title = entry.get('title').strip()
            if not subtitle and entry.get('titleType') == 'Subtitle':
                subtitle = entry.get('title', '').strip()
            if not original_language_title:
                original_language_title = find_original_language_title(entry)

    return title, original_language_title, subtitle


def parse_datacite_dates(dates):
    """
    Given a list of date fields (under .dates), return tuple, (release_date,
    release_year).
    """
    release_date, release_year = None, None

    if not dates:
        return release_date, release_year

    if not isinstance(dates, list):
        raise ValueError('expected a list of date items')

    # Observed values: "Available", "Submitted", "Valid", "Issued", "Accepted",
    # "Collected", "Updated", "Copyrighted", "Created"
    # Ignored for now: "Collected", "Issued"
    date_type_prio = (
        'Valid',
        'Available',
        'Accepted',
        'Submitted',
        'Copyrighted',
        'Created',
        'Updated',
    )

    # Before using (expensive) dateparser, try a few common patterns.
    common_patterns = ('%Y-%m-%d', '%Y-%m', '%Y-%m-%dT%H:%M:%SZ',
                       '%Y-%m-%dT%H:%M:%S', '%Y')

    def parse_item(item):
        result, value, year_only = None, item.get('date', ''), False
        release_date, release_year = None, None

        for pattern in common_patterns:
            try:
                result = datetime.datetime.strptime(value, pattern)
            except ValueError:
                continue
            else:
                if pattern == '%Y':
                    year_only = True
                break

        if result is None:
            print('fallback for {}'.format(value), file=sys.stderr)
            try:
                result = dateparser.parse(value)
            except TypeError as err:
                print("{} date parsing failed with: {}".format(value, err),
                      file=sys.stderr)
                return result_date, result_year

        if result is None:
            # Unparsable date.
            return release_date, release_year

        if not year_only:
            release_date = result.date()
        release_year = result.year

        return release_date, release_year

    for prio in date_type_prio:
        for item in dates:
            if not item.get('dateType') == prio:
                continue

            release_date, release_year = parse_item(item)
            if release_date is None and release_year is None:
                continue

            if release_year < 1000 or release_year > datetime.date.today(
            ).year + 5:
                # Skip possibly bogus dates.
                release_year = None
                continue
            break
        else:
            continue
        break

    if release_date is None and release_year is None:
        for item in dates:
            release_date, release_year = parse_item(item)
            if release_year or release_date:
                break

    return release_date, release_year

def clean_doi(doi):
    """
    10.25513/1812-3996.2017.1.34–42 // 8211, Hex 2013, Octal 20023
    See also: https://github.com/miku/throwaway-check-doi

    Replace unicode HYPHEN..HORIZONTAL BAR with HYPHEN-MINUS.
    """
    for c in ('\u2010', '\u2011', '\u2012', '\u2013', '\u2014', '\u2015'):
        doi = doi.replace(c, "-")
    return doi

