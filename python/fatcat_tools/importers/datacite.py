"""
WIP: Importer for datacite.org data.

Example doc at: https://gist.github.com/miku/5610a2d64e3fee82d16f5d3f3a295fc8
"""

from .common import EntityImporter
import dateparser
import datetime
import fatcat_openapi_client
import json
import sys

# https://guide.fatcat.wiki/entity_container.html#container_type-vocabulary
CONTAINER_TYPE_MAP = {
    'Journal': 'journal',
    'Series': 'journal',
    'Book Series': 'book-series',
}

# TODO(martin): merge this with other maps, maybe.
LICENSE_SLUG_MAP = {
    "//creativecommons.org/licenses/by/2.0": "CC-BY",
    "//creativecommons.org/licenses/by/2.0/uk/legalcode": "CC-BY",
    "//creativecommons.org/licenses/by/3.0": "CC-BY",
    "//creativecommons.org/licenses/by/3.0/us": "CC-BY",
    "//creativecommons.org/licenses/by/4.0": "CC-BY",
    "//creativecommons.org/licenses/by/4.0/deed.de": "CC-BY",
    "//creativecommons.org/licenses/by/4.0/deed.en_US": "CC-BY",
    "//creativecommons.org/licenses/by/4.0/legalcode": "CC-BY",
    "//creativecommons.org/licenses/by-nc/2.0": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc/3.0": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc/4.0": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc/4.0/legalcode": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc-nd/3.0": "CC-BY-NC-ND",
    "//creativecommons.org/licenses/by-nc-nd/3.0/gr": "CC-BY-NC-ND",
    "//creativecommons.org/licenses/by-nc-nd/4.0": "CC-BY-NC-ND",
    "//creativecommons.org/licenses/by-nc-nd/4.0": "CC-BY-ND",
    "//creativecommons.org/licenses/by-nc-nd/4.0/legalcode": "CC-BY-ND",
    "//creativecommons.org/licenses/by-nc-sa/4.0": "CC-BY-NC-SA",
    "//creativecommons.org/licenses/by-nc-sa/4.0": "CC-BY-SA",
    "//creativecommons.org/licenses/by-nd/4.0": "CC-BY-ND",
    "//creativecommons.org/licenses/by-sa/3.0/de": "CC-BY-SA",
    "//creativecommons.org/licenses/by-sa/3.0/gr": "CC-BY-SA",
    "//creativecommons.org/licenses/by-sa/4.0": "CC-BY-SA",
    "//creativecommons.org/licenses/by-sa/4.0/legalcode": "CC-BY-SA",
    "//creativecommons.org/licenses/CC-BY/4.0": "CC-BY",
    "//creativecommons.org/licenses/publicdomain/zero/1.0": "CC-0",
    "//creativecommons.org/publicdomain/zero/1.0": "CC-0",
    "//creativecommons.org/publicdomain/zero/1.0": "CC-0",
    "//creativecommons.org/publicdomain/zero/1.0/legalcode": "CC-0",
    "//opensource.org/licenses/MIT": "MIT",
    "//www.elsevier.com/open-access/userlicense/1.0": "ELSEVIER-USER-1.0",
    "//www.gnu.org/licenses/gpl-3.0.en.html": "GPLv3",
    "//www.gnu.org/licenses/old-licenses/gpl-2.0.en.html": "GPLv2",
    "//www.karger.com/Services/SiteLicenses": "KARGER",
    "//www.opensource.org/licenses/Apache-2.0": "Apache-2.0",
    "//www.opensource.org/licenses/BSD-3-Clause": "BSD-3-Clause",
    "//www.opensource.org/licenses/EUPL-1.1": "EUPL-1.1", # redirects to EUPL-1.2
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
}

class DataciteImporter(EntityImporter):
    """
    Importer for datacite records. TODO(martin): Do we need issn_map_file?
    """

    def __init__(self, api, issn_map_file, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Automated import of Datacite DOI metadata, harvested from REST API")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.DataciteImporter')
        super().__init__(api,
            issn_map_file=issn_map_file,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

        self.create_containers = kwargs.get('create_containers', True)
        self.read_issn_map_file(issn_map_file)

    def parse_record(self, obj):
        """
        TODO(martin): Map datacite to RE.

        WIP, notes:

        * Many subjects, should they end up in extra?
        * attributes.creators and attributes.contributors

        $ jq '.attributes.creators[]?.nameType?' datacite.500k | sort | uniq -c | sort -nr
        3963663 "Personal"
        289795 null
        8892 "Organizational"

        Shall we use issued, available?

          {
            "date": "2011-11-18",
            "dateType": "Accepted"
          },
          {
            "date": "2011-11-18",
            "dateType": "Available"
          },
          {
            "date": "2011-11-07",
            "dateType": "Copyrighted"
          },
          {
            "date": "2011-11-18",
            "dateType": "Issued"
          },
          {
            "date": "2011-11-07",
            "dateType": "Issued"
          }

        TODO(martin): Quick analysis of dates and stages.
        """

        if 'attributes' not in obj:
            return None

        attributes = obj['attributes']

        # > Contributors
        #
        #  "attributes.creators[].contributorType": [
        #    "author"
        #  ],
        #  "attributes.creators[].nameIdentifiers[].nameIdentifierScheme": [
        #    "LCNA",
        #    "GND",
        #    "email",
        #    "NAF",
        #    "OSF",
        #    "RRID",
        #    "ORCID",
        #    "SCOPUS",
        #    "NRCPID",
        #    "schema.org",
        #    "GRID",
        #    "MGDS",
        #    "VIAF",
        #    "JACoW-ID"
        #  ],
        #
        #    "https://orcid.org/0000-0002-9902-738X",
        #    "http://jacow.org/JACoW-00001280",
        #    "Wiebe_Peter",
        #    "https://osf.io/https://osf.io/kjfuy/",
        #    "http://www.viaf.org176549220",
        #    "2239",
        #    "Jeffries_Martin",
        #    "https://orcid.org/0000-0002-1493-6630",
        #    "0000-0002-6233-612X",
        #
        # "creators": [
        #   {
        #     "name": "Bögli, Hans",
        #     "nameType": "Personal",
        #     "givenName": "Hans",
        #     "familyName": "Bögli",
        #     "affiliation": []
        #   }
        # ],

        contribs = []

        for i, c in enumerate(attributes['creators']):
            if not c.get('nameType') == 'Personal':
                continue
            creator_id = None
            for nid in c.get('nameIdentifiers', []):
                if not nid.get('nameIdentifierScheme').lower() == "orcid":
                    continue
                orcid = nid.get('nameIdentifier', '').replace('https://orcid.org/', '')
                if not orcid:
                    continue
                creator_id = self.lookup_orcid(orcid)
                # If creator_id is None, should we create creators?
            contribs.append(fatcat_openapi_client.ReleaseContrib(
                creator_id=creator_id,
                index=i,
                raw_name=c.get('name'),
                given_name=c.get('givenName'),
                surname=c.get('familyName'),
            ))

        # > Title
        #
        #   "attributes.titles[].titleType": [
        #     "AlternativeTitle",
        #     "Other",
        #     "Subtitle",
        #     null,
        #     "TranslatedTitle"
        #   ],
        title, subtitle = None, None

        for entry in attributes.get('titles', []):
            if not title and 'titleType' not in entry:
                title = entry.get('title').strip()
            if entry.get('titleType') == 'Subtitle':
                subtitle = entry.get('title').strip()

        # > Dates
        #
        #  "attributes.dates[].dateType": [
        #    "Accepted",
        #    "Available"
        #    "Collected",
        #    "Copyrighted",
        #    "Created",
        #    "Issued",
        #    "Submitted",
        #    "Updated",
        #    "Valid",
        #  ],
        #
        # Different documents have different dates defined. Choose the topmost
        # available from prio list.
        date_type_prio = (
            'Valid',
            'Issued',
            'Available',
            'Accepted',
            'Submitted',
            'Copyrighted',
            'Collected',
            'Created',
            'Updated',
        )

        release_year, release_date = None, None
        for prio in date_type_prio:
            dates = attributes.get('dates', []) or [] # Never be None.
            for item in dates:
                if not item.get('dateType') == prio:
                    continue
                result = dateparser.parse(item.get('date'))
                if result is None:
                    # Unparsable date.
                    continue
                release_date = result
                release_year = result.year
                if 1000 < release_year < datetime.date.today().year + 5:
                    # Skip possibly bogus dates.
                    continue
                break
            else:
                continue
            break

        # > Publisher
        #
        # A few NA values. A few bogus values.
        #
        publisher = attributes.get('publisher')

        if publisher in ('(:unav)', 'Unknown', 'n.a.', '[s.n.]', '(:unap)'):
            publisher = None
        if publisher is not None and len(publisher) > 80:
            # Arbitrary magic value, TODO(martin): better heuristic.
            # Example: "ETH-Bibliothek Zürich, Bildarchiv / Fotograf: Feller,
            # Elisabeth, Empfänger, Unbekannt, Fotograf / Fel_041033-RE / Unbekannt,
            # Nutzungsrechte müssen durch den Nutzer abgeklärt werden",
            # TODO(martin): log misses.
            publisher = None

        # > Container
        #
        # For the moment, only ISSN as container.
        #
        #    "container": {
        #      "type": "Journal",
        #      "issue": "8",
        #      "title": "Angewandte Chemie International Edition",
        #      "volume": "57",
        #      "lastPage": "2080",
        #      "firstPage": "2077",
        #      "identifier": "14337851",
        #      "identifierType": "ISSN"
        #    },
        #
        # "attributes.container.type": [
        #   "DataRepository",
        #   "Journal",
        #   "Series",
        #   "Book Series"
        # ],
        #
        #  "attributes.container.identifierType": [
        #    "Handle",
        #    "ISBN",
        #    "LISSN",
        #    "DOI",
        #    "EISSN",
        #    "URL",
        #    "ISSN"
        #  ],

        container_id = None
        container = attributes.get('container', {}) or {}
        if container.get('type') in CONTAINER_TYPE_MAP.keys():
            container_type = CONTAINER_TYPE_MAP.get(container['type'])
            if container.get('identifier') and container.get('identifierType') == 'ISSN':
                issn = container.get('identifier')
                if len(issn) == 8:
                    issn = issn[:4] + "-" + issn[4:]
                issnl = self.issn2issnl(issn)
                container_id = self.lookup_issnl(issnl)

                if container_id is None and container.get('title'):
                    ce = fatcat_openapi_client.ContainerEntity(
                        issnl=issnl,
                        container_type=container_type,
                        name=container.get('title'),
                    )
                    ce_edit = self.create_container(ce)
                    container_id = ce_edit.ident
                    self._issnl_id_map[issnl] = container_id

        # > License
        #
        # attributes.rightsList[].rightsUri
        # attributes.rightsList[].rights
        # attributes.rightsList[].lang
        #

        license_slug = None
        license_extra = []
        for l in attributes.get('rightsList', []):
            slug = lookup_license_slug(l.get('rightsUri'))
            if slug:
                license_slug = slug
            license_extra.append(l)

        # > Release type.
        #
        # Datacite has some fine granular typing (e.g. "Supplementary
        # Collection of Datasets", "Taxonomic treatment", "blog_entry", ...
        #
        # Additional, coarse: resourceTypeGeneral
        #
        #  "attributes.types.resourceTypeGeneral": [
        #    "Image",
        #    "Dataset",
        #    "PhysicalObject",
        #    "Collection",
        #    "Text",
        #    "Sound",
        #    "InteractiveResource",
        #    "Event",
        #    "Software",
        #    "Other",
        #    "Workflow",
        #    "Audiovisual"
        #  ],

        # > Extra information.
        extra, extra_datacite = dict(), dict()
        if license_extra:
            extra_datacite['license'] = license_extra

        if extra_datacite:
            extra['datacite'] = extra_datacite

        # https://guide.fatcat.wiki/entity_release.html
        re = fatcat_openapi_client.ReleaseEntity(
            work_id=None,
            container_id=container_id,
            release_type=None,
            release_stage=None,
            title=title, # attributes.titles, various titleType
            subtitle=subtitle,
            original_title=title, # AlternativeTitle?
            release_year=release_year, # publicationYear
            release_date=release_date, # date issues/available?
            publisher=publisher, # attributes.publisher
            ext_ids=fatcat_openapi_client.ReleaseExtIds(
                doi=attributes.get('doi'), # attributes.doi,
                # Can we add handle.net link?
            ),
            contribs=contribs,
            volume=None,
            issue=None,
            pages=None,
            language=None,
            abstracts=None,
            refs=None,
            extra=extra,
            license_slug=license_slug,
        )
        return re

    def try_update(self, re, debug=True):
        if debug is True:
            # print(type(re))
            print(json.dumps(re.to_dict(), default=extended_encoder))
            return
        return False

    def insert_batch(self, batch):
        # Debugging.
        for item in batch:
            print(item)
        return

        # Orig.
        self.api.create_release_auto_batch(fatcat_openapi_client.ReleaseAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))

def extended_encoder(value):
    """
    Can be used with json.dumps(value, default=extended_encoder) to serialize
    value not serializable by default. https://docs.python.org/3/library/json.html#basic-usage
    """
    if isinstance(value, (datetime.datetime, datetime.date)):
        return value.isoformat()
    if isinstance(value, set):
        return list(value)

def lookup_license_slug(raw):
    """
    TODO(martin): reuse from crossref, maybe.
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
