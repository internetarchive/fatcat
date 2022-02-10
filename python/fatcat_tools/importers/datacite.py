"""
Prototype importer for datacite.org data.

Example input document: https://api.datacite.org/dois/10.7916/d8-f93n-rk51

Datacite being an aggregator, the data is heterogeneous and exposes a couple of
problems in content and structure. A few fields have their own parsing
functions (parse_datacite_...), which may help testing.
"""

import collections
import datetime
import json
import re
import sys
from typing import Any, Dict, List, Optional, Sequence, Set, Tuple

import dateparser
import fatcat_openapi_client
import langdetect
import pycountry
from fatcat_openapi_client import ApiClient, ReleaseContrib, ReleaseEntity

from fatcat_tools.biblio_lookup_tables import DATACITE_TYPE_MAP
from fatcat_tools.normal import clean_doi, clean_str, lookup_license_slug
from fatcat_tools.transforms import entity_to_dict

from .common import MAX_ABSTRACT_LENGTH, EntityImporter

# https://guide.fatcat.wiki/entity_container.html#container_type-vocabulary
DATACITE_CONTAINER_TYPE_MAP: Dict[str, str] = {
    "Journal": "journal",
    "Series": "journal",
    "Book Series": "book-series",
}

# DATACITE_UNKNOWN_MARKERS via https://support.datacite.org/docs/schema-values-unknown-information-v43.
DATACITE_UNKNOWN_MARKERS: List[str] = [
    "(:unac)",  # temporarily inaccessible
    "(:unal)",  # unallowed, suppressed intentionally
    "(:unap)",  # not applicable, makes no sense
    "(:unas)",  # value unassigned (e.g., Untitled)
    "(:unav)",  # value unavailable, possibly unknown
    "(:unkn)",  # known to be unknown (e.g., Anonymous, Inconnue)
    "(:none)",  # never had a value, never will
    "(:null)",  # explicitly and meaningfully empty
    "(:tba)",  # to be assigned or announced later
    "(:etal)",  # too numerous to list (et alia)
]

# UNKNOWN_MARKERS joins official datacite markers with a generic tokens marking
# unknown values.
UNKNOWN_MARKERS: Set[str] = set(DATACITE_UNKNOWN_MARKERS).union(
    set(
        (
            "NA",
            "NN",
            "n.a.",
            "[s.n.]",
            "Unknown",
        )
    )
)

# UNKNOWN_MARKERS_LOWER are lowercase version of UNKNOWN blocklist.
UNKNOWN_MARKERS_LOWER = set((v.lower() for v in UNKNOWN_MARKERS))

# Any "min" number of "tokens" will signal "spam", https://fatcat.wiki/release/rzcpjwukobd4pj36ipla22cnoi
DATACITE_TITLE_SPAM_WORDGROUPS: List[Dict[str, Any]] = [
    {
        "tokens": (
            "full",
            "movies",
            "movie",
            "watch",
            "streaming",
            "online",
            "free",
            "hd",
            "download",
            "english",
            "subtitle",
            "bluray",
        ),
        "min": 4,
    }
]


class DataciteImporter(EntityImporter):
    """
    Importer for datacite records.
    """

    def __init__(
        self,
        api: ApiClient,
        issn_map_file: Sequence,
        debug: bool = False,
        insert_log_file: bool = None,
        **kwargs
    ) -> None:

        eg_desc = kwargs.get(
            "editgroup_description",
            "Automated import of Datacite DOI metadata, harvested from REST API",
        )
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.DataciteImporter")
        super().__init__(
            api,
            issn_map_file=issn_map_file,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs
        )

        self.create_containers = kwargs.get("create_containers", True)
        self.read_issn_map_file(issn_map_file)
        self.debug = debug
        self.insert_log_file = insert_log_file
        self.this_year = datetime.datetime.now().year

        print("datacite with debug={}".format(self.debug), file=sys.stderr)

    def parse_record(self, obj: Dict[str, Any]) -> Optional[ReleaseEntity]:
        """
        Mapping datacite JSON to ReleaseEntity.
        """
        if not obj or not isinstance(obj, dict):
            return None
        if "attributes" not in obj:
            return None

        attributes = obj["attributes"]
        doi = clean_doi(attributes.get("doi", "").lower())

        if not doi:
            print("skipping record without a DOI", file=sys.stderr)
            return None

        if not str.isascii(doi):
            print("[{}] skipping non-ascii doi for now".format(doi))
            return None

        creators = attributes.get("creators", []) or []
        contributors = attributes.get("contributors", []) or []  # Much fewer than creators.

        contribs = self.parse_datacite_creators(creators, doi=doi)

        # Beside creators, we have contributors in datacite. Sample:
        # ContactPerson, DataCollector, DataCurator, DataManager, Distributor,
        # Editor, Funder, HostingInstitution, Other, Producer, ProjectLeader,
        # ProjectMember, RelatedPerson, ResearchGroup, Researcher,
        # RightsHolder, Sponsor, Supervisor
        #
        # Datacite schema:
        # https://schema.datacite.org/meta/kernel-4.3/doc/DataCite-MetadataKernel_v4.3.pdf#page=32
        # -- could be used as a form of controlled vocab?
        #
        # Currently (07/2020) in release_contrib:
        #
        # select count(*), role from release_contrib group by role;
        #    count   |    role
        # -----------+------------
        #  500269665 | author
        #    4386563 | editor
        #      17871 | translator
        #   10870584 |
        # (4 rows)
        #
        # Related: https://guide.fatcat.wiki/entity_release.html -- role
        # (string, of a set): the type of contribution, from a controlled
        # vocabulary. TODO: vocabulary needs review.
        contribs_extra_contributors = self.parse_datacite_creators(
            contributors, set_index=False, doi=doi
        )

        # Unfortunately, creators and contributors might overlap, refs GH59.
        for cc in contribs_extra_contributors:
            if contributor_list_contains_contributor(contribs, cc):
                continue
            contribs.append(cc)

        # Title, may come with "attributes.titles[].titleType", like
        # "AlternativeTitle", "Other", "Subtitle", "TranslatedTitle"
        titles = attributes.get("titles", []) or []
        title, original_language_title, subtitle = parse_datacite_titles(titles)

        if title is None:
            print("[{}] skipping record w/o title: {}".format(doi, obj), file=sys.stderr)
            return False

        title = clean_str(title)
        if not title:
            print("[{}] skipping record w/o title: {}".format(doi, obj), file=sys.stderr)
            return False

        # check for blocklisted "spam", e.g. "FULL MOVIE"
        for rule in DATACITE_TITLE_SPAM_WORDGROUPS:
            seen = set()
            token_list: List[str] = rule.get("tokens") or []
            for token in token_list:
                if token in title.lower():
                    seen.add(token)
            if len(seen) >= rule["min"]:
                print("[{}] skipping spammy title: {}".format(doi, obj), file=sys.stderr)
                return False

        if not subtitle:
            subtitle = None
        else:
            subtitle = clean_str(subtitle)

        # Dates. A few internal dates (registered, created, updated) and
        # published (0..2554). We try to work with typed date list, in
        # "attributes.dates[].dateType", values: "Accepted", "Available"
        # "Collected", "Copyrighted", "Created", "Issued", "Submitted",
        # "Updated", "Valid".
        release_date, release_month, release_year = parse_datacite_dates(
            attributes.get("dates", [])
        )

        # block bogus far-future years/dates
        if release_year is not None and (
            release_year > (self.this_year + 5) or release_year < 1000
        ):
            release_date = None
            release_month = None
            release_year = None

        # Some records do not use the "dates" field (e.g. micropub), but:
        # "attributes.published" or "attributes.publicationYear"
        if not any((release_date, release_month, release_year)):
            release_date, release_month, release_year = parse_single_date(
                attributes.get("publicationYear")
            )
            if not any((release_date, release_month, release_year)):
                release_date, release_month, release_year = parse_single_date(
                    attributes.get("published")
                )

        if not any((release_date, release_month, release_year)):
            print("[{}] record w/o date: {}".format(doi, obj), file=sys.stderr)

        # Start with clear stages, e.g. published. TODO(martin): we could
        # probably infer a bit more from the relations, e.g.
        # "IsPreviousVersionOf" or "IsNewVersionOf".
        release_stage: Optional[str] = "published"

        # TODO(martin): If 'state' is not 'findable' or 'isActive' is not true,
        # we might want something else than 'published'. See also:
        # https://support.datacite.org/docs/doi-states.

        # Publisher. A few NA values. A few bogus values.
        publisher = attributes.get("publisher")

        if publisher in UNKNOWN_MARKERS | set(("Unpublished", "Unknown")):
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
            publisher = clean_str(publisher)

        # Container. For the moment, only ISSN as container.
        container_id = None
        container_name = None

        container = attributes.get("container", {}) or {}
        if container.get("type") in DATACITE_CONTAINER_TYPE_MAP.keys():
            container_type = DATACITE_CONTAINER_TYPE_MAP.get(container["type"])
            if container.get("identifier") and container.get("identifierType") == "ISSN":
                issn = container.get("identifier")
                if issn and len(issn) == 8:
                    issn = issn[:4] + "-" + issn[4:]
                    issnl = self.issn2issnl(issn)
                else:
                    issnl = None
                if issnl is not None:
                    container_id = self.lookup_issnl(issnl)

                    if container_id is None and container.get("title"):
                        container_name = container.get("title")
                        if isinstance(container_name, list):
                            if len(container_name) > 0:
                                print(
                                    "[{}] too many container titles: {}".format(
                                        doi, len(container_name)
                                    )
                                )
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
                    container_name = container.get("title")
                    if isinstance(container_name, list):
                        if len(container_name) > 0:
                            print(
                                "[{}] too many container titles: {}".format(
                                    doi, len(container_name)
                                )
                            )
                            container_name = container_name[0]

        # Exception: https://www.micropublication.org/, see: !MR24.
        if container_id is None and container_name is None:
            if publisher and publisher.lower().startswith("micropublication"):
                container_name = publisher

        # Volume and issue.
        volume = container.get("volume")
        issue = container.get("issue")

        if volume:
            volume = clean_str(volume)

        if issue:
            issue = clean_str(issue)

        # Pages.
        pages = None

        first_page = container.get("firstPage")
        last_page = container.get("lastPage")

        if first_page and last_page:
            try:
                _ = int(first_page) < int(last_page)
                pages = "{}-{}".format(first_page, last_page)
            except ValueError as err:  # noqa: F841
                # TODO(martin): This is more debug than info.
                # print('[{}] {}'.format(doi, err), file=sys.stderr)
                pass

        if not pages and first_page:
            pages = first_page

        # License.
        license_slug = None
        license_extra = []

        for lic in attributes.get("rightsList", []):
            slug = datacite_lookup_license_slug(lic.get("rightsUri"))
            if slug:
                license_slug = slug
            license_extra.append(lic)

        release_type = self.datacite_release_type(doi, attributes)

        # Language values are varied ("ger", "es", "English", "ENG", "en-us",
        # "other", ...). Try to crush it with langcodes: "It may sound to you
        # like langcodes solves a pretty boring problem. At one level, that's
        # right. Sometimes you have a boring problem, and it's great when a
        # library solves it for you." -- TODO(martin): We need more of these.
        language = None

        value = attributes.get("language", "") or ""
        try:
            language = pycountry.languages.lookup(value).alpha_2
        except (LookupError, AttributeError) as err:  # noqa: F841
            pass
            # TODO(martin): Print this on debug level, only.
            # print('[{}] language lookup miss for {}: {}'.format(doi, value, err), file=sys.stderr)

        # Abstracts appear in "attributes.descriptions[].descriptionType", some
        # of the observed values: "Methods", "TechnicalInfo",
        # "SeriesInformation", "Other", "TableOfContents", "Abstract". The
        # "Other" fields might contain references or related articles (with
        # DOI). TODO(martin): maybe try to parse out some of those refs.
        abstracts = []
        descs = attributes.get("descriptions", []) or []
        for desc in descs:
            if not desc.get("descriptionType") == "Abstract":
                continue

            # Description maybe a string, int or list.
            text = desc.get("description", "")
            if not text:
                continue
            if isinstance(text, int):
                text = "{}".format(text)
            if isinstance(text, list):
                try:
                    text = "\n".join(text)
                except TypeError:
                    continue  # Bail out, if it is not a list of strings.

            # Limit length.
            if len(text) < 10:
                continue
            if len(text) > MAX_ABSTRACT_LENGTH:
                text = text[:MAX_ABSTRACT_LENGTH] + " [...]"

            # Detect language. This is fuzzy and may be removed, if too unreliable.
            lang = None
            try:
                lang = langdetect.detect(text)
            except (langdetect.lang_detect_exception.LangDetectException, TypeError) as err:
                print(
                    "[{}] language detection failed with {} on {}".format(doi, err, text),
                    file=sys.stderr,
                )
            abstract_text = clean_str(text)
            if not abstract_text:
                continue
            abstracts.append(
                fatcat_openapi_client.ReleaseAbstract(
                    mimetype="text/plain",
                    content=abstract_text,
                    lang=lang,
                )
            )

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

        relIds = attributes.get("relatedIdentifiers", []) or []
        for rel in relIds:
            if not rel.get("relationType", "") in ("References", "Cites"):
                continue
            ref_extra = dict()
            if rel.get("relatedIdentifierType", "") == "DOI":
                ref_extra["doi"] = rel.get("relatedIdentifier")
            refs.append(
                fatcat_openapi_client.ReleaseRef(
                    index=ref_index,
                    extra=ref_extra or None,
                )
            )
            ref_index += 1

        # More specific release_type via 'Reviews' relationsship.
        for rel in relIds:
            if rel.get("relatedIdentifierType", "") != "Reviews":
                continue
            release_type = "review"

        # Extra information.
        extra_datacite: Dict[str, Any] = dict()

        if license_extra:
            extra_datacite["license"] = license_extra
        if attributes.get("subjects"):
            # these subjects with schemeUri are too much metadata, which
            # doesn't compress. filter them out.
            extra_subjects = [
                subj for subj in attributes["subjects"] if not subj.get("schemeUri")
            ]
            if extra_subjects:
                extra_datacite["subjects"] = extra_subjects

        # Include version information.
        metadata_version = attributes.get("metadataVersion") or ""

        if metadata_version:
            extra_datacite["metadataVersion"] = metadata_version

        # Include resource types.
        types = attributes.get("types") or {}
        resource_type = types.get("resourceType", "") or ""
        resource_type_general = types.get("resourceTypeGeneral", "") or ""

        if resource_type and resource_type.lower() not in UNKNOWN_MARKERS_LOWER:
            extra_datacite["resourceType"] = resource_type
        if resource_type_general and resource_type_general.lower() not in UNKNOWN_MARKERS_LOWER:
            extra_datacite["resourceTypeGeneral"] = resource_type_general

        # Include certain relations from relatedIdentifiers. Keeping the
        # original structure of data here, which is a list of dicts, with
        # relation type, identifier and identifier type (mostly).
        relations = []
        for rel in relIds:
            if rel.get("relationType") in (
                "IsPartOf",
                "Reviews",
                "Continues",
                "IsVariantFormOf",
                "IsSupplementTo",
                "HasVersion",
                "IsMetadataFor",
                "IsNewVersionOf",
                "IsIdenticalTo",
                "IsVersionOf",
                "IsDerivedFrom",
                "IsSourceOf",
            ):
                relations.append(rel)

        # TODO: could use many of these relations to do release/work grouping

        if relations:
            extra_datacite["relations"] = relations

        extra: Dict[str, Any] = dict()

        # "1.0.0", "v1.305.2019", "Final", "v1.0.0", "v0.3.0", "1", "0.19.0",
        # "3.1", "v1.1", "{version}", "4.0", "10329", "11672", "11555",
        # "v1.4.5", "2", "V1", "v3.0", "v0", "v0.6", "11124", "v1.0-beta", "1st
        # Edition", "20191024", "v2.0.0", "v0.9.3", "10149", "2.0", null,
        # "v0.1.1", "3.0", "1.0", "3", "v1.12.2", "20191018", "v0.3.1", "v1.0",
        # "10161", "10010691", "10780", # "Presentación"
        version = attributes.get("version") or None

        # top-level extra keys
        if not container_id and container_name:
            extra["container_name"] = container_name

        # Always include datacite key, even if value is empty (dict).
        extra["datacite"] = extra_datacite

        # Preparation for a schema update.
        if release_month:
            extra["release_month"] = release_month

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
            version=version,
        )
        re = self.biblio_hacks(re)
        return re

    @staticmethod
    def datacite_release_type(doi: str, attributes: Dict[str, Any]) -> Optional[str]:
        """
        Release type. Try to determine the release type from a variety of types
        supplied in datacite. The "attributes.types.resourceType" is
        uncontrolled (170000+ unique values, from "null", "Dataset" to "Jupyter
        Notebook" and "Macroseismic Data Points" or "2 days of IP flows in
        2009") citeproc may be the closest, but not always supplied.  Order
        lookup roughly by completeness of mapping.
        """

        release_type = None
        if not attributes.get("types"):
            return None
        types = attributes["types"]

        for typeType in ("citeproc", "ris", "schemaOrg", "bibtex", "resourceTypeGeneral"):
            value = types.get(typeType)
            release_type = DATACITE_TYPE_MAP.get(typeType, {}).get(value)
            if release_type is not None:
                break

        # special case: figshare "collections" which group other entities
        if doi.startswith("10.6084/") or doi.startswith("10.25384"):
            if types.get("resourceType") == "Collection":
                release_type = "stub"

        if release_type is None:
            print("[{}] no mapped type: {}".format(doi, types), file=sys.stderr)

        return release_type

    @staticmethod
    def biblio_hacks(re: ReleaseEntity) -> ReleaseEntity:
        """
        This function handles known special cases. For example,
        publisher-specific or platform-specific workarounds.
        """

        # only runs on datacite entities with a DOI
        assert re.ext_ids.doi

        # release_type exception: Global Biodiversity Information Facility
        # publishes highly interesting datasets, but titles are mostly the same
        # ("GBIF Occurrence Download" or "Occurrence Download"); set
        # release_type to "stub" (CSL/FC).
        if re.title == "GBIF Occurrence Download" and re.ext_ids.doi.startswith("10.15468/dl."):
            re.release_type = "stub"

        # release_type exception: lots of "Experimental Crystal Structure Determination"
        # publisher: "Cambridge Crystallographic Data Centre"
        if re.ext_ids.doi.startswith("10.5517/"):
            re.release_type = "entry"

        # Supplement files, e.g. "Additional file 1: ASE constructs in questionnaire."
        if re.title.lower().startswith("additional file") and re.release_type in (
            "article",
            "article-journal",
        ):
            re.release_type = "component"

        # figshare
        if re.ext_ids.doi.startswith("10.6084/") or re.ext_ids.doi.startswith("10.25384"):
            # set version if DOI ends with versioned suffix
            doi_suffix = re.ext_ids.doi.split(".")[-1]
            if doi_suffix and doi_suffix.startswith("v") and doi_suffix[1:].isdigit():
                re.version = doi_suffix
            # "Figure 123 from " -> component
            # "Table S1. ;Figure S1;Figure S2. ;Figure S3. ;Figure S4. from Use of organic exudates from two polar diatoms by bacterial isolates from the Arctic ocean"
            if " from " in re.title and re.release_type not in ("stub", "graphic"):
                if re.title.startswith("Figure "):
                    re.release_type = "component"
                elif re.title.startswith("Table "):
                    re.release_type = "component"

        # figshare.com
        if (
            re.ext_ids.doi.startswith("10.6084/m9.figshare.")
            and re.extra.get("container_name") is None
        ):
            re.extra["container_name"] = "figshare.com"

        # Columbia Institutional Repository includes full bibliographic
        # metadata, which results in incorrect container_id matches. But this
        # DOI prefix also publishes actual journals!
        if (
            re.ext_ids.doi.startswith("10.7916/")
            and "-" in re.ext_ids.doi
            and re.publisher == "Columbia University"
            and re.extra
            and re.extra.get("datacite")
        ):
            for relation in re.extra["datacite"].get("relations", []):
                if relation.get("relationType") == "IsVariantFormOf":
                    re.container_id = None
                    if re.release_stage in ("published", None):
                        re.release_stage = "submitted"

        # several institutional and other repositories (including "RWTH" and
        # "DESY") also results in incorrect container_id matches.
        # This probably doesn't filter out enough, but is a start.
        IR_DOI_PREFIXES = [
            "10.15495/epub_ubt_",
            "10.18154/rwth-20",
            "10.3204/pubdb-",
            "10.3204/phppubdb-",
            "10.26204/kluedo/",
        ]
        for prefix in IR_DOI_PREFIXES and re.extra and re.extra.get("datacite"):
            if re.ext_ids.doi.startswith(prefix):
                for relation in re.extra["datacite"].get("relations", []):
                    if relation.get("relationType") == "IsVariantFormOf":
                        re.container_id = None

        return re

    def try_update(self, re: ReleaseEntity) -> bool:
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

        # eventually we'll want to support "updates", but for now just skip if
        # entity already exists
        if existing:
            self.counts["exists"] += 1
            return False

        return True

    def insert_batch(self, batch: List[ReleaseEntity]) -> None:
        print("inserting batch ({})".format(len(batch)), file=sys.stderr)
        if self.insert_log_file:
            with open(self.insert_log_file, "a") as f:
                for doc in batch:
                    json.dump(entity_to_dict(doc, api_client=None), f)
                    f.write("\n")
        self.api.create_release_auto_batch(
            fatcat_openapi_client.ReleaseAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description, extra=self.editgroup_extra
                ),
                entity_list=batch,
            )
        )

    def parse_datacite_creators(
        self,
        creators: List[Dict[str, Any]],
        role: str = "author",
        set_index: bool = True,
        doi: Optional[str] = None,
    ) -> List[ReleaseContrib]:
        """
        Parses a list of creators into a list of ReleaseContrib objects. Set
        set_index to False, if the index contrib field should be left blank.
        The doi parameter is only used for debugging.
        """
        # Contributors. Many nameIdentifierSchemes, we do not use (yet):
        # "attributes.creators[].nameIdentifiers[].nameIdentifierScheme":
        # ["LCNA", "GND", "email", "NAF", "OSF", "RRID", "ORCID",
        # "SCOPUS", "NRCPID", "schema.org", "GRID", "MGDS", "VIAF", "JACoW-ID"].
        contribs: List[ReleaseContrib] = []

        # Names, that should be ignored right away.
        name_blocklist = set(("Occdownload Gbif.Org",))

        i: Optional[int] = 0
        for c in creators:
            if not set_index:
                i = None
            nameType = c.get("nameType", "") or ""
            if nameType in ("", "Personal"):
                creator_id = None
                for nid in c.get("nameIdentifiers", []) or []:
                    if not isinstance(nid, dict):
                        # see: fatcat-workers/issues/44035/
                        print(
                            "unexpected nameIdentifiers, expected list of dicts, got: {}".format(
                                nid
                            ),
                            file=sys.stderr,
                        )
                        continue
                    name_scheme = nid.get("nameIdentifierScheme", "") or ""
                    if not name_scheme.lower() == "orcid":
                        continue
                    orcid = nid.get("nameIdentifier") or ""
                    orcid = orcid.replace("https://orcid.org/", "")
                    if not orcid:
                        continue
                    creator_id = self.lookup_orcid(orcid)
                    # TODO(martin): If creator_id is None, should we create creators?

                # If there are multiple affiliation strings, use the first one.
                affiliations = c.get("affiliation", []) or []
                raw_affiliation = None
                if len(affiliations) == 0:
                    raw_affiliation = None
                else:
                    raw_affiliation = clean_str(affiliations[0])

                name = c.get("name")
                given_name = c.get("givenName")
                surname = c.get("familyName")

                if name:
                    name = clean_str(name)
                if not any((name, given_name, surname)):
                    continue
                if not name:
                    name = "{} {}".format(given_name or "", surname or "").strip()
                if name in name_blocklist:
                    continue
                if name.lower() in UNKNOWN_MARKERS_LOWER:
                    continue
                # Unpack name, if we have an index form (e.g. 'Razis, Panos A') into 'Panos A razis'.
                if name:
                    name = index_form_to_display_name(name)

                if given_name:
                    given_name = clean_str(given_name)
                surname = clean_str(surname)

                # Perform a final assertion that name does not reduce to zero
                # (e.g. whitespace only name).
                if name:
                    name = name.strip()
                if not name:
                    continue

                if raw_affiliation == "":
                    continue

                extra = None

                # "DataManager", "DataCurator", "ContactPerson", "Distributor",
                # "RegistrationAgency", "Sponsor", "Researcher",
                # "RelatedPerson", "ProjectLeader", "Editor", "Other",
                # "ProjectMember", "Funder", "RightsHolder", "DataCollector",
                # "Supervisor", "Producer", "HostingInstitution", "ResearchGroup"
                contributorType = c.get("contributorType", "") or ""

                if contributorType:
                    extra = {"type": contributorType}

                rc = fatcat_openapi_client.ReleaseContrib(
                    creator_id=creator_id,
                    index=i,
                    raw_name=name,
                    given_name=given_name,
                    surname=surname,
                    role=role,
                    raw_affiliation=raw_affiliation,
                    extra=extra,
                )
                # Filter out duplicates early.
                if not contributor_list_contains_contributor(contribs, rc):
                    contribs.append(rc)
                    if i is not None:
                        i += 1
            elif nameType == "Organizational":
                name = c.get("name", "") or ""
                if name in UNKNOWN_MARKERS:
                    continue
                if len(name) < 3:
                    continue
                extra = {"organization": name}
                contribs.append(fatcat_openapi_client.ReleaseContrib(index=i, extra=extra))
                if i is not None:
                    i += 1
            else:
                print("[{}] unknown name type: {}".format(doi, nameType), file=sys.stderr)

        return contribs


def contributor_list_contains_contributor(
    contributor_list: ReleaseContrib, contributor: ReleaseContrib
) -> bool:
    """
    Given a list of contributors, determine, whether contrib is in that list.
    """
    for cc in contributor_list:
        if cc.raw_name != contributor.raw_name:
            continue
        cc_role = cc.role or "author"
        contributor_role = contributor.role or "author"
        if cc_role != contributor_role:
            continue
        return True
    return False


def datacite_lookup_license_slug(raw: Optional[str]) -> Optional[str]:
    """
    Resolve a variety of strings into a some pseudo-canonical form, e.g.
    CC-BY-ND, CC-0, MIT and so on.
    TODO(martin): reuse from or combine with crossref, maybe.
    """
    if not raw:
        return None

    if "creativecommons.org/publicdomain/zero" in raw:
        return "CC-0"
    if raw.lower().endswith("/cc0"):
        return "CC-0"

    if "creativecommons" in raw:
        # https://creativecommons.org/publicdomain/mark/1.0/deed.de
        if "creativecommons.org/publicdomain" in raw:
            return "CC-PUBLICDOMAIN"
        if "creativecommons.org/share-your-work/public-domain/cc0" in raw:
            return "CC-0"
        # https://creativecommons.org/licenses/by/4.0/deed.es_ES
        raw = raw.lower()
        match = re.search(
            r"creativecommons.org/licen[sc]es/(?P<name>[a-z-]+)", raw, re.IGNORECASE
        )
        if not match:
            print("missed potential license: {}".format(raw), file=sys.stderr)
            return None
        name = match.groupdict().get("name")
        if not name:
            return None
        if not name.startswith("cc"):
            name = "cc-{}".format(name)
        return name.upper()

    if "opensource.org" in raw:
        # https://opensource.org/licenses/alphabetical, e.g. opensource.org/licenses/EUPL-1.2
        match = re.search(r"opensource.org/licenses/(?P<name>[^/]+)", raw, re.IGNORECASE)
        if not match:
            print("missed potential license: {}".format(raw), file=sys.stderr)
            return None
        name = match.groupdict().get("name")
        if not name:
            return None
        if len(name) > 11:
            return None
        return name.upper()

    if "gnu.org" in raw:
        # http://www.gnu.org/copyleft/gpl, https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html
        match = re.search(
            r"/(?P<name>fdl(-[0-9.]*[0-9]+)?|gpl(-[0-9.]*[0-9]+)?|lgpl(-[0-9.]*[0-9]+)|aglp(-[0-9.]*[0-9]+)?)",
            raw,
            re.IGNORECASE,
        )
        if not match:
            print("missed potential license: {}".format(raw), file=sys.stderr)
            return None
        name = match.groupdict().get("name")
        if not name:
            return None
        if len(name) > 8:
            return None
        return name.upper()

    if "spdx.org" in raw:
        if "spdx.org/licenses/CC0" in raw:
            return "CC-0"
        # https://spdx.org/licenses/CC-BY-NC-ND-4.0.html
        match = re.search(r"spdx.org/licenses/(?P<name>[a-z0-9-]+)", raw, re.IGNORECASE)
        if not match:
            print("missed potential license: {}".format(raw), file=sys.stderr)
            return None
        name = match.groupdict().get("name")
        if not name:
            return None
        if len(name) > 36:
            return None
        # cleanup version and extensions
        name = re.sub("(-[0-9])?[.]?[0-9]?(.json|.html)?", "", name.lower())
        return name.upper()

    if "rightsstatements.org" in raw:
        # http://rightsstatements.org/vocab/InC/1.0/
        match = re.search(r"rightsstatements.org/(vocab|page)/(?P<name>[^/]*)", raw)
        if not match:
            print("missed potential license: {}".format(raw), file=sys.stderr)
            return None
        name = match.groupdict().get("name")
        if not name:
            return None
        if len(name) > 9:
            return None
        return "RS-{}".format(name.upper())

    # Fallback to generic license lookup
    return lookup_license_slug(raw)


def find_original_language_title(
    item: Dict[str, Any], min_length: int = 4, max_questionmarks: int = 3
) -> Optional[str]:
    """
    Perform a few checks before returning a potential original language title.

    Example input: {'title': 'Some title', 'original_language_title': 'Some title'}
    """
    if "original_language_title" not in item:
        return None
    title = item.get("title")
    if not title:
        return None
    original_language_title = item.get("original_language_title")
    if isinstance(original_language_title, str) and title != original_language_title:
        if len(original_language_title) < min_length:
            return None
        if original_language_title.count("?") > max_questionmarks:
            return None
        return original_language_title
    if isinstance(original_language_title, dict):
        content = original_language_title.get("__content__", "") or ""
        if content and content != title and not content.count("?") > max_questionmarks:
            return content
    return None


def parse_datacite_titles(
    titles: List[Dict[str, Any]]
) -> Tuple[Optional[str], Optional[str], Optional[str]]:
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
        title = titles[0].get("title", "") or ""
        title = title.strip()
        if not title:
            title = None
        return title, original_language_title, subtitle
    else:
        for entry in titles:
            if not title and ("titleType" not in entry or not entry.get("titleType")):
                title = (entry.get("title") or "").strip()
            if not subtitle and entry.get("titleType") == "Subtitle":
                subtitle = entry.get("title", "").strip()
            if not original_language_title:
                original_language_title = find_original_language_title(entry)

    return title, original_language_title, subtitle


def parse_single_date(
    value: Optional[str],
) -> Tuple[Optional[datetime.date], Optional[int], Optional[int]]:
    """
    Given a single string containing a date in arbitrary format, try to return
    tuple (date: datetime.date, month: int, year: int).
    """
    if not value:
        return None, None, None
    if isinstance(value, int):
        value = str(value)
    parser = dateparser.date.DateDataParser()
    try:
        # Results in a dict with keys: date_obj, period, locale.
        parse_result = parser.get_date_data(value)
        # A datetime object, later we need a date, only.
        result = parse_result["date_obj"]
        if result is not None:
            if parse_result["period"] == "year":
                return None, None, result.year
            elif parse_result["period"] == "month":
                return None, result.month, result.year
            else:
                return result.date(), result.month, result.year
    except TypeError as err:
        print("{} date parsing failed with: {}".format(value, err), file=sys.stderr)

    return None, None, None


def parse_datacite_dates(
    dates: List[Dict[str, Any]],
) -> Tuple[Optional[datetime.date], Optional[int], Optional[int]]:
    """
    Given a list of date fields (under .dates), return tuple, (release_date,
    release_month, release_year).
    """
    release_date, release_month, release_year = None, None, None

    if not dates:
        return release_date, release_month, release_year

    if not isinstance(dates, list):
        raise ValueError("expected a list of date items")

    # Observed values: "Available", "Submitted", "Valid", "Issued", "Accepted",
    # "Collected", "Updated", "Copyrighted", "Created"
    # Ignored for now: "Collected", "Issued"
    date_type_prio = (
        "Valid",
        "Available",
        "Accepted",
        "Submitted",
        "Copyrighted",
        "Created",
        "Updated",
    )

    # We need to note the granularity, since a string like "2019" would be
    # parsed into "2019-01-01", even though the month is unknown. Use 3
    # granularity types: 'y', 'm', 'd'.
    Pattern = collections.namedtuple("Pattern", "layout granularity")

    # Before using (expensive) dateparser, try a few common patterns.
    common_patterns = (
        Pattern("%Y-%m-%d", "d"),
        Pattern("%Y-%m", "m"),
        Pattern("%Y-%m-%dT%H:%M:%SZ", "d"),
        Pattern("%Y-%m-%dT%H:%M:%S", "d"),
        Pattern("%Y", "y"),
    )

    def parse_item(
        item: Dict[str, Any]
    ) -> Tuple[Optional[datetime.date], Optional[int], Optional[int]]:
        result, value = None, str(item.get("date", "")) or ""
        release_date: Optional[datetime.date] = None
        release_month: Optional[int] = None
        release_year: Optional[int] = None

        for layout, granularity in common_patterns:
            try:
                result = datetime.datetime.strptime(value, layout)
            except ValueError:
                continue
            else:
                if granularity == "y":
                    # TODO: year_only = True
                    pass
                break

        if result is None:
            print("fallback for {}".format(value), file=sys.stderr)
            release_date, release_month, release_year = parse_single_date(value)

        # XXX: result has not been updated since the above line
        if result is None:
            # Unparsable date.
            return release_date, release_month, release_year

        if granularity != "y":
            release_date = result.date()
        release_year = result.year
        if granularity in ("m", "d"):
            release_month = result.month

        return release_date, release_month, release_year

    today = datetime.date.today()

    for prio in date_type_prio:
        for item in dates:
            if not item.get("dateType") == prio:
                continue

            release_date, release_month, release_year = parse_item(item)
            if release_date is None and release_year is None:
                continue

            if release_year is not None and (
                release_year < 1000 or release_year > today.year + 5
            ):
                # Skip possibly bogus dates.
                release_year = None
                continue
            break
        else:
            continue
        break

    if release_date is None and release_year is None:
        for item in dates:
            release_date, release_month, release_year = parse_item(item)
            if release_year or release_date:
                break

    return release_date, release_month, release_year


def index_form_to_display_name(s: str) -> str:
    """
    Try to convert an index form name, like 'Razis, Panos A' into display_name,
    e.g. 'Panos A Razis'.
    """
    if "," not in s:
        return s
    skip_on_chars = ["(", ")", "*"]
    for char in skip_on_chars:
        if char in s:
            return s
    if s.count(",") > 1:
        # "Dr. Hina, Dr. Muhammad Usman Shahid, Dr. Muhammad Zeeshan Khan"
        return s

    # Not names, but sprinkled in fields where authors live.
    stopwords = [
        s.lower()
        for s in (
            "Archive",
            "Collection",
            "Coordinator",
            "Department",
            "Germany",
            "International",
            "National",
            "Netherlands",
            "Office",
            "Organisation",
            "Organization",
            "Service",
            "Services",
            "United States",
            "University",
            "Verein",
            "Volkshochschule",
        )
    ]
    lower = s.lower()
    for stop in stopwords:
        if stop in lower:
            return s

    a, b = s.split(",")
    return "{} {}".format(b.strip(), a.strip())
