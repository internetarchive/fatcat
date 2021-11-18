import datetime
import json
import sys
import warnings
from typing import Any, Dict, List, Optional, Sequence

import fatcat_openapi_client
from bs4 import BeautifulSoup
from fatcat_openapi_client import DefaultApi, ReleaseEntity

from fatcat_tools.biblio_lookup_tables import LANG_MAP_MARC
from fatcat_tools.normal import clean_doi, clean_str

from .common import EntityImporter
from .crossref import CONTAINER_TYPE_MAP

# TODO: more entries?
JSTOR_CONTRIB_MAP = {
    "author": "author",
    "editor": "editor",
    "translator": "translator",
    "illustrator": "illustrator",
}

JSTOR_TYPE_MAP = {
    "book-review": "review-book",
    "editorial": "editorial",
    "misc": "stub",
    "news": "article",
    "research-article": "article-journal",
}


class JstorImporter(EntityImporter):
    """
    Importer for JSTOR bulk XML metadata (eg, from their Early Journals
    Collection)
    """

    def __init__(self, api: DefaultApi, issn_map_file: Sequence, **kwargs) -> None:

        eg_desc = kwargs.get("editgroup_description", "Automated import of JSTOR XML metadata")
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.JstorImporter")
        super().__init__(
            api,
            issn_map_file=issn_map_file,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs
        )

        self.create_containers = kwargs.get("create_containers", True)

        self.read_issn_map_file(issn_map_file)

    def map_container_type(self, crossref_type: Optional[str]) -> Optional[str]:
        if not crossref_type:
            return None
        return CONTAINER_TYPE_MAP.get(crossref_type)

    def want(self, raw_record: Any) -> bool:
        return True

    # TODO: mypy annotations partially skipped on this function ('Any' instead of
    # 'BeautifulSoup') for now because XML parsing annotations are large and
    # complex
    def parse_record(self, article: Any) -> Optional[ReleaseEntity]:

        journal_meta = article.front.find("journal-meta")
        article_meta = article.front.find("article-meta")

        extra: Dict[str, Any] = dict()
        extra_jstor: Dict[str, Any] = dict()

        release_type = JSTOR_TYPE_MAP.get(article["article-type"])
        title = article_meta.find("article-title")
        if title and title.get_text():
            title = title.get_text().replace("\n", " ").strip()
        elif title and not title.get_text():
            title = None

        if (
            not title
            and release_type
            and release_type.startswith("review")
            and article_meta.product.source
        ):
            title = "Review: {}".format(
                article_meta.product.source.replace("\n", " ").get_text()
            )

        if not title:
            return None

        if title.endswith("."):
            title = title[:-1]

        if "[Abstract]" in title:
            # TODO: strip the "[Abstract]" bit?
            release_type = "abstract"
        elif "[Editorial" in title:
            release_type = "editorial"
        elif "[Letter" in title:
            release_type = "letter"
        elif "[Poem" in title or "[Photograph" in title:
            release_type = None

        if title.startswith("[") and title.endswith("]"):
            # strip brackets if that is all that is there (eg, translation or non-english)
            title = title[1:-1]

        # JSTOR journal-id
        journal_ids = [j.string for j in journal_meta.find_all("journal-id")]
        if journal_ids:
            extra_jstor["journal_ids"] = journal_ids

        journal_title = journal_meta.find("journal-title").get_text().replace("\n", " ")
        publisher = journal_meta.find("publisher-name").get_text().replace("\n", " ")
        issn = journal_meta.find("issn")
        if issn:
            issn = issn.string
            if len(issn) == 8:
                issn = "{}-{}".format(issn[0:4], issn[4:8])
            else:
                assert len(issn) == 9

        issnl = self.issn2issnl(issn)
        container_id = None
        if issnl:
            container_id = self.lookup_issnl(issnl)

        # create container if it doesn't exist
        if (
            container_id is None
            and self.create_containers
            and (issnl is not None)
            and journal_title
        ):
            ce = fatcat_openapi_client.ContainerEntity(
                issnl=issnl,
                publisher=publisher,
                container_type=self.map_container_type(release_type),
                name=clean_str(journal_title, force_xml=True),
            )
            ce_edit = self.create_container(ce)
            container_id = ce_edit.ident
            self._issnl_id_map[issnl] = container_id

        doi = article_meta.find("article-id", {"pub-id-type": "doi"})
        if doi:
            doi = clean_doi(doi.string.lower())
        else:
            doi = None

        jstor_id = article_meta.find("article-id", {"pub-id-type": "jstor"})
        if jstor_id:
            jstor_id = jstor_id.string.strip()
        if not jstor_id and doi:
            assert doi.startswith("10.2307/")
            jstor_id = doi.replace("10.2307/", "")
        assert jstor_id and int(jstor_id)

        contribs = []
        cgroup = article_meta.find("contrib-group")
        if cgroup:
            for c in cgroup.find_all("contrib"):
                given = c.find("given-names")
                if given:
                    given = clean_str(given.get_text().replace("\n", " "))
                surname = c.find("surname")
                if surname:
                    surname = clean_str(surname.get_text().replace("\n", " "))
                raw_name = c.find("string-name")
                if raw_name:
                    raw_name = clean_str(raw_name.get_text().replace("\n", " "))

                if not raw_name:
                    if given and surname:
                        raw_name = "{} {}".format(given, surname)
                    elif surname:
                        raw_name = surname

                role = JSTOR_CONTRIB_MAP.get(c.get("contrib-type", "author"))
                if not role and c.get("contrib-type"):
                    sys.stderr.write("NOT IN JSTOR_CONTRIB_MAP: {}\n".format(c["contrib-type"]))
                contribs.append(
                    fatcat_openapi_client.ReleaseContrib(
                        role=role,
                        raw_name=raw_name,
                        given_name=given,
                        surname=surname,
                    )
                )

        for i, contrib in enumerate(contribs):
            if contrib.raw_name != "et al.":
                contrib.index = i

        release_year = None
        release_date = None
        pub_date = article_meta.find("pub-date")
        if pub_date and pub_date.year:
            release_year = int(pub_date.year.string)
            if pub_date.month and pub_date.day:
                release_date = datetime.date(
                    release_year, int(pub_date.month.string), int(pub_date.day.string)
                )
                if release_date.day == 1 and release_date.month == 1:
                    # suspect jan 1st dates get set by JSTOR when actual
                    # date not known (citation needed), so drop them
                    release_date = None

        volume = None
        if article_meta.volume:
            volume = article_meta.volume.string or None

        issue = None
        if article_meta.issue:
            issue = article_meta.issue.string or None

        pages = None
        if article_meta.find("page-range"):
            pages = article_meta.find("page-range").string
        elif article_meta.fpage:
            pages = article_meta.fpage.string

        language = None
        cm = article_meta.find("custom-meta")
        if cm.find("meta-name").string == "lang":
            language = cm.find("meta-value").string.split()[0]
            language = LANG_MAP_MARC.get(language)
            if not language:
                warnings.warn("MISSING MARC LANG: {}".format(cm.find("meta-value").string))

        # JSTOR issue-id
        if article_meta.find("issue-id"):
            issue_id = clean_str(article_meta.find("issue-id").string)
            if issue_id:
                extra_jstor["issue_id"] = issue_id

        # everything in JSTOR is published
        release_stage = "published"

        # extra:
        #   withdrawn_date
        #   translation_of
        #   subtitle
        #   aliases
        #   container_name
        #   group-title
        #   pubmed: retraction refs
        if extra_jstor:
            extra["jstor"] = extra_jstor

        re = fatcat_openapi_client.ReleaseEntity(
            # work_id
            title=title,
            # original_title
            release_type=release_type,
            release_stage=release_stage,
            release_date=release_date,
            release_year=release_year,
            ext_ids=fatcat_openapi_client.ReleaseExtIds(
                doi=doi,
                jstor=jstor_id,
            ),
            volume=volume,
            issue=issue,
            pages=pages,
            publisher=publisher,
            language=language,
            # license_slug
            # content, mimetype, lang
            # abstracts=abstracts,
            contribs=contribs,
            # key, year, container_name, title, locator
            # extra: volume, authors, issue, publisher, identifiers
            # refs=refs,
            #   name, type, publisher, issnl
            #   extra: issnp, issne, original_name, languages, country
            container_id=container_id,
            extra=extra or None,
        )
        return re

    def try_update(self, re: ReleaseEntity) -> bool:

        # first, lookup existing by JSTOR id (which much be defined)
        existing = None
        try:
            existing = self.api.lookup_release(jstor=re.ext_ids.jstor)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        # then try DOI lookup if there is one (try JSTOR prefix+jstor_id if
        # there isn't a DOI set)
        if not existing:
            doi = re.ext_ids.doi
            if not doi:
                doi = "10.2307/{}".format(re.ext_ids.jstor)
            try:
                existing = self.api.lookup_release(doi=doi)
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status != 404:
                    raise err

        if existing and existing.ext_ids.jstor:
            # don't update if it already has JSTOR ID
            self.counts["exists"] += 1
            return False
        elif existing:
            # but do update if only DOI was set
            existing.ext_ids.jstor = re.ext_ids.jstor
            existing.extra["jstor"] = re.extra["jstor"]
            # better release_type detection, and some other fields
            # TODO: don't do this over-writing in the future? assuming here
            # this is a one-time batch import over/extending bootstrap crossref
            # metadata
            existing.release_type = re.release_type
            existing.publisher = re.publisher
            existing.contribs = re.contribs
            existing.language = re.language
            self.api.update_release(self.get_editgroup_id(), existing.ident, existing)
            self.counts["update"] += 1
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
        for article in soup.find_all("article"):
            resp = self.parse_record(article)
            print(json.dumps(resp))
            # sys.exit(-1)
