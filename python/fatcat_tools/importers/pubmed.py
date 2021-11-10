import datetime
import json
import sys
import warnings
from typing import Any, Dict, List, Optional, Sequence

import fatcat_openapi_client
from bs4 import BeautifulSoup
from fatcat_openapi_client import ApiClient, ReleaseEntity

from fatcat_tools.biblio_lookup_tables import (
    COUNTRY_NAME_MAP,
    LANG_MAP_MARC,
    MONTH_ABBR_MAP,
    PUBMED_RELEASE_TYPE_MAP,
)
from fatcat_tools.normal import clean_doi, clean_issn, clean_pmcid, clean_pmid, clean_str

from .common import EntityImporter


class PubmedImporter(EntityImporter):
    """
    Importer for PubMed/MEDLINE XML metadata.

    If lookup_refs is true, will do identifer-based lookups for all references.

    TODO: MEDLINE doesn't include PMC/OA license; could include in importer?
    """

    def __init__(
        self, api: ApiClient, issn_map_file: Sequence, lookup_refs: bool = True, **kwargs
    ):

        eg_desc = kwargs.get(
            "editgroup_description", "Automated import of PubMed/MEDLINE XML metadata"
        )
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.PubmedImporter")
        super().__init__(
            api,
            issn_map_file=issn_map_file,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs
        )

        self.lookup_refs = lookup_refs
        self.create_containers = kwargs.get("create_containers", True)
        self.read_issn_map_file(issn_map_file)

    def want(self, raw_record: BeautifulSoup) -> bool:
        return True

    # TODO: mypy annotations partially skipped on this function ('Any' instead of
    # 'BeautifulSoup') for now because XML parsing annotations are large and
    # complex
    def parse_record(self, a: Any) -> ReleaseEntity:

        medline = a.MedlineCitation
        # PubmedData isn't required by DTD, but seems to always be present
        pubmed = a.PubmedData
        extra = dict()
        extra_pubmed = dict()

        identifiers = pubmed.ArticleIdList
        pmid = medline.PMID.string.strip()
        doi = identifiers.find("ArticleId", IdType="doi")
        if doi and doi.string:
            doi = clean_doi(doi.string)
        else:
            doi = None

        pmcid = identifiers.find("ArticleId", IdType="pmc")
        if pmcid:
            pmcid = clean_pmcid(pmcid.string.strip().upper())

        release_type = None
        pub_types = []
        for pub_type in medline.Article.PublicationTypeList.find_all("PublicationType"):
            pub_types.append(pub_type.string)
            if pub_type.string in PUBMED_RELEASE_TYPE_MAP:
                release_type = PUBMED_RELEASE_TYPE_MAP[pub_type.string]
                break
        if pub_types:
            extra_pubmed["pub_types"] = pub_types
        if medline.Article.PublicationTypeList.find(string="Retraction of Publication"):
            release_type = "retraction"
            retraction_of = medline.find("CommentsCorrections", RefType="RetractionOf")
            if retraction_of:
                if retraction_of.RefSource:
                    extra_pubmed["retraction_of_raw"] = retraction_of.RefSource.string
                if retraction_of.PMID:
                    extra_pubmed["retraction_of_pmid"] = retraction_of.PMID.string

        # everything in medline is published
        release_stage = "published"
        if medline.Article.PublicationTypeList.find(string="Corrected and Republished Article"):
            release_stage = "updated"
        if medline.Article.PublicationTypeList.find(string="Retraction of Publication"):
            release_stage = "retraction"

        withdrawn_status = None
        if medline.Article.PublicationTypeList.find(string="Retracted Publication"):
            withdrawn_status = "retracted"
        elif medline.find("CommentsCorrections", RefType="ExpressionOfConcernIn"):
            withdrawn_status = "concern"

        pages = medline.find("MedlinePgn")
        if pages:
            pages = pages.string

        title = medline.Article.ArticleTitle.get_text()  # always present
        if title:
            title = title.replace("\n", " ")
            if title.endswith("."):
                title = title[:-1]
            # this hides some "special" titles, but the vast majority are
            # translations; translations don't always include the original_title
            if title.startswith("[") and title.endswith("]"):
                title = title[1:-1]
        else:
            # will filter out later
            title = None

        original_title = medline.Article.find("VernacularTitle", recurse=False)
        if original_title:
            original_title = original_title.get_text() or None
            original_title = original_title.replace("\n", " ")
            if original_title and original_title.endswith("."):
                original_title = original_title[:-1]

        if original_title and not title:
            # if we only have an "original" title, but not translated/english
            # title, sub in the original title so the entity can be created
            title = original_title
            original_title = None

        # TODO: happening in alpha order, not handling multi-language well.
        language = medline.Article.Language
        if language:
            language = language.get_text()
            if language in ("und", "un"):
                # "undetermined"
                language = None
            else:
                language = LANG_MAP_MARC.get(language)
                if not language and not (medline.Article.Language.get_text() in LANG_MAP_MARC):
                    warnings.warn(
                        "MISSING MARC LANG: {}".format(medline.Article.Language.string)
                    )

        ### Journal/Issue Metadata
        # MedlineJournalInfo is always present
        issnl = None
        container_id = None
        container_name = None
        container_extra = dict()
        mji = medline.MedlineJournalInfo
        if mji.find("Country"):
            country_name = mji.Country.string.strip()
            country_code = COUNTRY_NAME_MAP.get(country_name)
            if country_code:
                container_extra["country"] = country_code
            elif country_name:
                container_extra["country_name"] = country_name
        if mji.find("ISSNLinking"):
            issnl = mji.ISSNLinking.string

        journal = medline.Article.Journal
        issnp = journal.find("ISSN", IssnType="Print")
        if issnp:
            issnp = clean_issn(issnp.string)
        else:
            issnp = None

        if not issnl and issnp:
            issnl = self.issn2issnl(issnp)
        else:
            issnl = None

        if issnl:
            container_id = self.lookup_issnl(issnl)

        pub_date = medline.Article.find("ArticleDate")
        if not pub_date:
            pub_date = journal.PubDate
        if not pub_date:
            pub_date = journal.JournalIssue.PubDate
        release_date: Optional[str] = None
        release_year: Optional[int] = None
        if pub_date.Year:
            release_year = int(pub_date.Year.string)
            if pub_date.find("Day") and pub_date.find("Month"):
                try:
                    release_date_date = datetime.date(
                        release_year,
                        MONTH_ABBR_MAP[pub_date.Month.string],
                        int(pub_date.Day.string),
                    )
                    release_date = release_date_date.isoformat()
                except ValueError as ve:
                    print("bad date, skipping: {}".format(ve), file=sys.stderr)
                    release_date = None
        elif pub_date.MedlineDate:
            medline_date = pub_date.MedlineDate.string.strip()
            if len(medline_date) >= 4 and medline_date[:4].isdigit():
                release_year = int(medline_date[:4])
                if release_year < 1300 or release_year > 2040:
                    print(
                        "bad medline year, skipping: {}".format(release_year), file=sys.stderr
                    )
                    release_year = None
            else:
                print(
                    "unparsable medline date, skipping: {}".format(medline_date),
                    file=sys.stderr,
                )

        if journal.find("Title"):
            container_name = journal.Title.get_text()

        if (
            container_id is None
            and self.create_containers
            and (issnl is not None)
            and container_name
        ):
            # name, type, publisher, issnl
            # extra: original_name, languages, country
            ce = fatcat_openapi_client.ContainerEntity(
                name=container_name,
                container_type="journal",
                # NOTE: publisher not included
                issnl=issnl,
                issnp=issnp,
                extra=(container_extra or None),
            )
            ce_edit = self.create_container(ce)
            container_id = ce_edit.ident
            self._issnl_id_map[issnl] = container_id

        ji = journal.JournalIssue
        volume = None
        if ji.find("Volume"):
            volume = ji.Volume.string
        issue = None
        if ji.find("Issue"):
            issue = ji.Issue.string

        ### Abstracts
        # "All abstracts are in English"
        abstracts = []
        primary_abstract = medline.find("Abstract")
        if primary_abstract and primary_abstract.AbstractText.get("NlmCategory"):
            joined = "\n".join(
                [m.get_text() for m in primary_abstract.find_all("AbstractText")]
            )
            abst = fatcat_openapi_client.ReleaseAbstract(
                content=joined,
                mimetype="text/plain",
                lang="en",
            )
            if abst.content:
                abstracts.append(abst)
        elif primary_abstract:
            for abstract in primary_abstract.find_all("AbstractText"):
                abst = fatcat_openapi_client.ReleaseAbstract(
                    content=abstract.get_text().strip(),
                    mimetype="text/plain",
                    lang="en",
                )
                if abst.content:
                    abstracts.append(abst)
                if abstract.find("math"):
                    abst = fatcat_openapi_client.ReleaseAbstract(
                        # strip the <AbstractText> tags
                        content=str(abstract)[14:-15],
                        mimetype="application/mathml+xml",
                        lang="en",
                    )
                    if abst.content:
                        abstracts.append(abst)
        other_abstracts = medline.find_all("OtherAbstract")
        for other in other_abstracts:
            lang: Optional[str] = "en"
            if other.get("Language"):
                lang = LANG_MAP_MARC.get(other["Language"])
            abst = fatcat_openapi_client.ReleaseAbstract(
                content=other.AbstractText.get_text().strip(),
                mimetype="text/plain",
                lang=lang,
            )
            if abst.content:
                abstracts.append(abst)

        ### Contribs
        contribs = []
        if medline.AuthorList:
            for author in medline.AuthorList.find_all("Author"):
                creator_id = None
                given_name = None
                surname = None
                raw_name = None
                if author.ForeName:
                    given_name = author.ForeName.get_text().replace("\n", " ")
                if author.LastName:
                    surname = author.LastName.get_text().replace("\n", " ")
                if given_name and surname:
                    raw_name = "{} {}".format(given_name, surname)
                elif surname:
                    raw_name = surname
                if not raw_name and author.CollectiveName and author.CollectiveName.get_text():
                    raw_name = author.CollectiveName.get_text().replace("\n", " ")
                contrib_extra = dict()
                orcid = author.find("Identifier", Source="ORCID")
                if orcid:
                    # needs re-formatting from, eg, "0000000179841889"
                    orcid = orcid.string
                    if orcid.startswith("http://orcid.org/"):
                        orcid = orcid.replace("http://orcid.org/", "")
                    elif orcid.startswith("https://orcid.org/"):
                        orcid = orcid.replace("https://orcid.org/", "")
                    elif "-" not in orcid:
                        orcid = "{}-{}-{}-{}".format(
                            orcid[0:4],
                            orcid[4:8],
                            orcid[8:12],
                            orcid[12:16],
                        )
                    creator_id = self.lookup_orcid(orcid)
                    contrib_extra["orcid"] = orcid
                affiliations = author.find_all("Affiliation")
                raw_affiliation = None
                if affiliations:
                    raw_affiliation = affiliations[0].get_text().replace("\n", " ")
                    if len(affiliations) > 1:
                        contrib_extra["more_affiliations"] = [
                            ra.get_text().replace("\n", " ") for ra in affiliations[1:]
                        ]
                if author.find("EqualContrib"):
                    # TODO: schema for this?
                    contrib_extra["equal"] = True
                contribs.append(
                    fatcat_openapi_client.ReleaseContrib(
                        raw_name=raw_name,
                        given_name=given_name,
                        surname=surname,
                        role="author",
                        raw_affiliation=raw_affiliation,
                        creator_id=creator_id,
                        extra=contrib_extra,
                    )
                )

            if medline.AuthorList["CompleteYN"] == "N":
                contribs.append(fatcat_openapi_client.ReleaseContrib(raw_name="et al."))

        for i, contrib in enumerate(contribs):
            if contrib.raw_name != "et al.":
                contrib.index = i

        ### References
        refs = []
        if pubmed.ReferenceList:
            # note that Reference always exists within a ReferenceList, but
            # that there may be multiple ReferenceList (eg, sometimes one per
            # Reference)
            for ref in pubmed.find_all("Reference"):
                ref_extra: Dict[str, Any] = dict()
                ref_doi = ref.find("ArticleId", IdType="doi")
                if ref_doi:
                    ref_doi = clean_doi(ref_doi.string)
                ref_pmid = ref.find("ArticleId", IdType="pubmed")
                if ref_pmid:
                    ref_pmid = clean_pmid(ref_pmid.string)
                ref_release_id = None
                if ref_doi:
                    ref_extra["doi"] = ref_doi
                    if self.lookup_refs:
                        ref_release_id = self.lookup_doi(ref_doi)
                if ref_pmid:
                    ref_extra["pmid"] = ref_pmid
                    if self.lookup_refs:
                        ref_release_id = self.lookup_pmid(ref_pmid)
                ref_raw = ref.Citation
                if ref_raw:
                    ref_extra["unstructured"] = ref_raw.get_text()
                refs.append(
                    fatcat_openapi_client.ReleaseRef(
                        target_release_id=ref_release_id,
                        extra=ref_extra or None,
                    )
                )

        # extra:
        #   translation_of
        #   aliases
        #   container_name
        #   group-title
        #   pubmed: retraction refs
        if extra_pubmed:
            extra["pubmed"] = extra_pubmed

        title = clean_str(title)
        if not title:
            return None

        re = fatcat_openapi_client.ReleaseEntity(
            work_id=None,
            title=title,
            original_title=clean_str(original_title),
            release_type=release_type,
            release_stage=release_stage,
            release_date=release_date,
            release_year=release_year,
            withdrawn_status=withdrawn_status,
            ext_ids=fatcat_openapi_client.ReleaseExtIds(
                doi=doi,
                pmid=pmid,
                pmcid=pmcid,
                # isbn13     # never in Article
            ),
            volume=volume,
            issue=issue,
            pages=pages,
            # publisher  # not included?
            language=language,
            # license_slug   # not in MEDLINE
            abstracts=abstracts or None,
            contribs=contribs or None,
            refs=refs or None,
            container_id=container_id,
            extra=extra or None,
        )
        return re

    def try_update(self, re: ReleaseEntity) -> bool:

        # first, lookup existing by PMID (which must be defined)
        existing = None
        try:
            existing = self.api.lookup_release(pmid=re.ext_ids.pmid)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        # then try DOI lookup if there is one
        if not existing and re.ext_ids.doi:
            try:
                existing = self.api.lookup_release(doi=re.ext_ids.doi)
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status != 404:
                    raise err
            if existing and existing.ext_ids.pmid and existing.ext_ids.pmid != re.ext_ids.pmid:
                warn_str = "PMID/DOI mismatch: release {}, pmid {} != {}".format(
                    existing.ident, existing.ext_ids.pmid, re.ext_ids.pmid
                )
                warnings.warn(warn_str)
                self.counts["warn-pmid-doi-mismatch"] += 1
                # don't clobber DOI, but do group together
                re.ext_ids.doi = None
                re.work_id = existing.work_id

        if existing and not self.do_updates:
            self.counts["exists"] += 1
            return False

        if existing and existing.ext_ids.pmid and (existing.refs or not re.refs):
            # TODO: any other reasons to do an update?
            # don't update if it already has PMID
            self.counts["exists"] += 1
            return False
        elif existing:
            # but do update if only DOI was set
            existing.ext_ids.doi = existing.ext_ids.doi or re.ext_ids.doi
            existing.ext_ids.pmid = existing.ext_ids.pmid or re.ext_ids.pmid
            existing.ext_ids.pmcid = existing.ext_ids.pmcid or re.ext_ids.pmcid

            existing.container_id = existing.container_id or re.container_id
            existing.refs = existing.refs or re.refs
            existing.abstracts = existing.abstracts or re.abstracts
            existing.extra["pubmed"] = re.extra["pubmed"]

            # fix stub titles
            if existing.title in [
                "OUP accepted manuscript",
            ]:
                existing.title = re.title

            existing.original_title = existing.original_title or re.original_title
            existing.release_type = existing.release_type or re.release_type
            existing.release_stage = existing.release_stage or re.release_stage
            existing.release_date = existing.release_date or re.release_date
            existing.release_year = existing.release_year or re.release_year
            existing.withdrawn_status = existing.withdrawn_status or re.withdrawn_status
            existing.volume = existing.volume or re.volume
            existing.issue = existing.issue or re.issue
            existing.pages = existing.pages or re.pages
            existing.language = existing.language or re.language

            # update subtitle in-place first
            if not existing.subtitle and existing.extra.get("subtitle"):
                subtitle = existing.extra.pop("subtitle")
                if type(subtitle) == list:
                    subtitle = subtitle[0]
                if subtitle:
                    existing.subtitle = subtitle
            if not existing.subtitle:
                existing.subtitle = re.subtitle

            try:
                self.api.update_release(self.get_editgroup_id(), existing.ident, existing)
                self.counts["update"] += 1
            except fatcat_openapi_client.rest.ApiException as err:
                # there is a code path where we try to update the same release
                # twice in a row; if that happens, just skip
                # NOTE: API behavior might change in the future?
                if "release_edit_editgroup_id_ident_id_key" in err.body:
                    self.counts["skip-update-conflict"] += 1
                    return False
                else:
                    raise err
            finally:
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

    def parse_file(self, handle: Any) -> None:

        # 1. open with beautiful soup
        soup = BeautifulSoup(handle, "xml")

        # 2. iterate over articles, call parse_article on each
        for article in soup.find_all("PubmedArticle"):
            resp = self.parse_record(article)
            print(json.dumps(resp))
            # sys.exit(-1)
