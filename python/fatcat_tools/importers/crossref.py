import datetime
from typing import Any, Dict, List, Optional, Sequence

import fatcat_openapi_client
from fatcat_openapi_client import DefaultApi, ReleaseContrib, ReleaseEntity

from fatcat_tools.biblio_lookup_tables import CONTAINER_TYPE_MAP
from fatcat_tools.normal import clean_doi, clean_str, lookup_license_slug

from .common import EntityImporter

# The docs/guide should be the canonical home for these mappings; update there
# first
# Can get a list of Crossref types (with counts) via API:
# https://api.crossref.org/works?rows=0&facet=type-name:*
CROSSREF_TYPE_MAP: Dict[str, Optional[str]] = {
    "book": "book",
    "book-chapter": "chapter",
    "book-part": "chapter",
    "book-section": "chapter",
    "component": "component",
    "dataset": "dataset",
    "dissertation": "thesis",
    "edited-book": "book",
    "journal-article": "article-journal",
    "monograph": "book",
    "other": None,
    "peer-review": "peer_review",
    "posted-content": "post",
    "proceedings-article": "paper-conference",
    "reference-book": "book",
    "reference-entry": "entry",
    "report": "report",
    "standard": "standard",
}


class CrossrefImporter(EntityImporter):
    """
    Importer for Crossref metadata.

    See https://github.com/CrossRef/rest-api-doc for JSON schema notes
    """

    def __init__(self, api: DefaultApi, issn_map_file: Sequence, **kwargs) -> None:

        eg_desc: Optional[str] = kwargs.get(
            "editgroup_description",
            "Automated import of Crossref DOI metadata, harvested from REST API",
        )
        eg_extra: Dict[str, Any] = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.CrossrefImporter")
        super().__init__(
            api,
            issn_map_file=issn_map_file,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs
        )

        self.create_containers: bool = kwargs.get("create_containers", True)
        self.read_issn_map_file(issn_map_file)

    def map_release_type(self, crossref_type: str) -> Optional[str]:
        return CROSSREF_TYPE_MAP.get(crossref_type)

    def map_container_type(self, crossref_type: Optional[str]) -> Optional[str]:
        if not crossref_type:
            return None
        return CONTAINER_TYPE_MAP.get(crossref_type)

    def want(self, obj: Dict[str, Any]) -> bool:
        if not obj.get("title"):
            self.counts["skip-blank-title"] += 1
            return False

        # these are pre-registered DOIs before the actual record is ready
        # title is a list of titles
        titles = obj.get("title")
        if titles is not None and titles[0].strip().lower() in [
            "OUP accepted manuscript".lower(),
        ]:
            self.counts["skip-stub-title"] += 1
            return False

        # do most of these checks in-line below
        return True

    def parse_record(self, obj: Dict[str, Any]) -> Optional[ReleaseEntity]:
        """
        obj is a python dict (parsed from json).
        returns a ReleaseEntity
        """

        # Ways to be out of scope (provisionally)
        # journal-issue and journal-volume map to None, but allowed for now
        if obj.get("type") in (
            None,
            "journal",
            "proceedings",
            "standard-series",
            "report-series",
            "book-series",
            "book-set",
            "book-track",
            "proceedings-series",
        ):
            self.counts["skip-release-type"] += 1
            return None

        # Do require the 'title' keys to exist, as release entities do
        if ("title" not in obj) or (not obj["title"]):
            self.counts["skip-blank-title"] += 1
            return None

        release_type = self.map_release_type(obj["type"])

        # contribs
        def do_contribs(obj_list: List[Dict[str, Any]], ctype: str) -> List[ReleaseContrib]:
            contribs = []
            for i, am in enumerate(obj_list):
                creator_id = None
                if "ORCID" in am.keys():
                    creator_id = self.lookup_orcid(am["ORCID"].split("/")[-1])
                # Sorry humans :(
                if am.get("given") and am.get("family"):
                    raw_name: Optional[str] = "{} {}".format(am["given"], am["family"])
                elif am.get("family"):
                    raw_name = am["family"]
                else:
                    # TODO: can end up empty
                    raw_name = am.get("name") or am.get("given")
                extra: Dict[str, Any] = dict()
                if ctype == "author":
                    index: Optional[int] = i
                else:
                    index = None
                raw_affiliation = None
                affiliation_list = am.get("affiliation") or []
                if affiliation_list and len(affiliation_list) > 0:
                    raw_affiliation = affiliation_list[0]["name"]
                    if len(affiliation_list) > 1:
                        # note: affiliation => more_affiliations
                        extra["more_affiliations"] = [
                            clean_str(a["name"]) for a in affiliation_list[1:]
                        ]
                if am.get("sequence") and am.get("sequence") != "additional":
                    extra["seq"] = clean_str(am.get("sequence"))
                assert ctype in ("author", "editor", "translator")
                raw_name = clean_str(raw_name)
                # TODO: what if 'raw_name' is None?
                contribs.append(
                    ReleaseContrib(
                        creator_id=creator_id,
                        index=index,
                        raw_name=raw_name,
                        given_name=clean_str(am.get("given")),
                        surname=clean_str(am.get("family")),
                        raw_affiliation=clean_str(raw_affiliation),
                        role=ctype,
                        extra=extra or None,
                    )
                )
            return contribs

        contribs = do_contribs(obj.get("author", []), "author")
        contribs.extend(do_contribs(obj.get("editor", []), "editor"))
        contribs.extend(do_contribs(obj.get("translator", []), "translator"))

        # container
        issn = obj.get("ISSN", [None])[0]
        issnl = self.issn2issnl(issn)
        container_id = None
        if issnl:
            container_id = self.lookup_issnl(issnl)
        publisher = clean_str(obj.get("publisher"))

        container_name = obj.get("container-title")
        if container_name:
            container_name = clean_str(container_name[0], force_xml=True)
        if not container_name:
            container_name = None
        if (
            container_id is None
            and self.create_containers
            and (issnl is not None)
            and container_name
        ):
            ce = fatcat_openapi_client.ContainerEntity(
                issnl=issnl,
                publisher=publisher,
                container_type=self.map_container_type(release_type),
                name=container_name,
            )
            ce_edit = self.create_container(ce)
            container_id = ce_edit.ident
            self._issnl_id_map[issnl] = container_id

        # license slug
        license_slug = None
        license_extra = []
        for lic in obj.get("license", []):
            if lic["content-version"] not in ("vor", "unspecified"):
                continue
            slug = lookup_license_slug(lic["URL"])
            if slug:
                license_slug = slug
            if "start" in lic:
                lic["start"] = lic["start"]["date-time"]
            license_extra.append(lic)

        # references
        refs = []
        for i, rm in enumerate(obj.get("reference", [])):
            try:
                year: Optional[int] = int(rm.get("year"))
                # TODO: will need to update/config in the future!
                # NOTE: are there crossref works with year < 100?
                if year is not None:
                    if year > 2025 or year < 100:
                        year = None
            except (TypeError, ValueError):
                year = None
            ref_extra: Dict[str, Any] = dict()
            key = rm.get("key")
            if key and key.startswith(obj["DOI"].upper()):
                key = key.replace(obj["DOI"].upper() + "-", "")
                key = key.replace(obj["DOI"].upper(), "")
            ref_container_name = rm.get("volume-title")
            if not ref_container_name:
                ref_container_name = rm.get("journal-title")
            elif rm.get("journal-title"):
                ref_extra["journal-title"] = rm["journal-title"]
            if rm.get("DOI"):
                ref_extra["doi"] = rm.get("DOI").lower()
            author = clean_str(rm.get("author"))
            if author:
                ref_extra["authors"] = [author]
            for k in (
                "editor",
                "edition",
                "authority",
                "version",
                "genre",
                "url",
                "event",
                "issue",
                "volume",
                "date",
                "accessed_date",
                "issued",
                "page",
                "medium",
                "collection_title",
                "chapter_number",
                "unstructured",
                "series-title",
                "volume-title",
            ):
                if clean_str(rm.get(k)):
                    ref_extra[k] = clean_str(rm[k])
            refs.append(
                fatcat_openapi_client.ReleaseRef(
                    index=i,
                    # doing lookups would be a second import pass
                    target_release_id=None,
                    key=key,
                    year=year,
                    container_name=clean_str(ref_container_name),
                    title=clean_str(rm.get("article-title")),
                    locator=clean_str(rm.get("first-page")),
                    # TODO: just dump JSON somewhere here?
                    extra=ref_extra or None,
                )
            )

        # abstracts
        abstracts = []
        abstract = clean_str(obj.get("abstract"))
        if abstract and len(abstract) > 10:
            abstracts.append(
                fatcat_openapi_client.ReleaseAbstract(
                    mimetype="application/xml+jats", content=abstract
                )
            )

        # extra fields
        extra: Dict[str, Any] = dict()
        extra_crossref: Dict[str, Any] = dict()
        # top-level extra keys
        if not container_id:
            if obj.get("container-title"):
                extra["container_name"] = container_name
        for key in "group-title":
            val = obj.get(key)
            if val:
                if type(val) == list:
                    val = val[0]
                if type(val) == str:
                    val = clean_str(val)
                    if val:
                        extra[key] = clean_str(val)
                else:
                    extra[key] = val
        # crossref-nested extra keys
        for key in ("subject", "type", "alternative-id", "archive", "funder"):
            val = obj.get(key)
            if val:
                if type(val) == str:
                    extra_crossref[key] = clean_str(val)
                else:
                    extra_crossref[key] = val
        if license_extra:
            extra_crossref["license"] = license_extra

        if len(obj["title"]) > 1:
            aliases = [clean_str(t) for t in obj["title"][1:]]
            aliases = [t for t in aliases if t]
            if aliases:
                extra["aliases"] = aliases

        # ISBN
        isbn13 = None
        for raw in obj.get("ISBN", []):
            # TODO: convert if not ISBN-13 format
            if len(raw) == 17:
                isbn13 = raw
                break

        # release status
        if obj["type"] in (
            "journal-article",
            "conference-proceeding",
            "book",
            "dissertation",
            "book-chapter",
        ):
            release_stage: Optional[str] = "published"
        else:
            # unknown
            release_stage = None

        # filter out unreasonably huge releases
        if len(abstracts) > 100:
            self.counts["skip-huge-abstracts"] += 1
            return None
        if len(contribs) > 2000:
            self.counts["skip-huge-contribs"] += 1
            return None
        if len(refs) > 5000:
            self.counts["skip-huge-refs"] += 1
            return None

        # release date parsing is amazingly complex
        raw_date = obj["issued"]["date-parts"][0]
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

        original_title: Optional[str] = None
        if obj.get("original-title"):
            ot = obj.get("original-title")
            if ot is not None:
                original_title = clean_str(ot[0], force_xml=True)

        title: Optional[str] = None
        if obj.get("title"):
            title = clean_str(obj["title"][0], force_xml=True)
            if not title or len(title) <= 1:
                # title can't be just a single character
                self.counts["skip-blank-title"] += 1
                return None

        doi = clean_doi(obj["DOI"].lower())
        if not doi:
            self.counts["skip-bad-doi"] += 1
            return None

        subtitle = None
        if obj.get("subtitle"):
            subtitle = clean_str(obj["subtitle"][0], force_xml=True)
            if not subtitle or len(subtitle) <= 1:
                # subtitle can't be just a single character
                subtitle = None

        if extra_crossref:
            extra["crossref"] = extra_crossref

        re = ReleaseEntity(
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
                doi=doi,
                isbn13=isbn13,
            ),
            volume=clean_str(obj.get("volume")),
            issue=clean_str(obj.get("issue")),
            pages=clean_str(obj.get("page")),
            language=clean_str(obj.get("language")),
            license_slug=license_slug,
            extra=extra or None,
            abstracts=abstracts or None,
            contribs=contribs or None,
            refs=refs or None,
        )
        return re

    def try_update(self, re: ReleaseEntity) -> bool:

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
        self.api.create_release_auto_batch(
            fatcat_openapi_client.ReleaseAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description, extra=self.editgroup_extra
                ),
                entity_list=batch,
            )
        )
