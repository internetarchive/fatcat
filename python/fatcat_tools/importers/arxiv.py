import datetime
import json
import re
import sys
from typing import Any, Dict, List, Optional

import fatcat_openapi_client
from bs4 import BeautifulSoup
from fatcat_openapi_client import ApiClient, ReleaseEntity
from pylatexenc.latex2text import LatexNodes2Text

from fatcat_tools.normal import clean_doi

from .common import EntityImporter
from .crossref import lookup_license_slug

latex2text = LatexNodes2Text()


def latex_to_text(raw: str) -> str:
    # hack: handle a single special mangled title
    if raw.startswith("%CRTFASTGEEPWR"):
        return raw.strip()
    try:
        return latex2text.latex_to_text(raw).strip()
    except AttributeError:
        return raw.strip()
    except IndexError:
        return raw.strip()


def test_latex_to_text() -> None:
    s = "%CRTFASTGEEPWR: a SAS macro for power of the generalized estimating equations of multi-period cluster randomized trials with application to stepped wedge designs"
    assert latex_to_text(s) == s


def parse_arxiv_authors(raw: str) -> List[str]:
    if not raw:
        return []
    raw = raw.replace("*", "")
    if "(" in raw:
        raw = re.sub(r"\(.*\)", "", raw)
    authors = raw.split(", ")
    if authors:
        last = authors[-1].split(" and ")
        if len(last) == 2:
            authors[-1] = last[0]
            authors.append(last[1])
        if authors[-1].startswith("and "):
            authors[-1] = authors[-1][4:]
    authors = [latex_to_text(a).strip() for a in authors]
    authors = [a for a in authors if a]
    return authors


def test_parse_arxiv_authors() -> None:

    assert parse_arxiv_authors(
        "Raphael Chetrite, Shamik Gupta, Izaak Neri and \\'Edgar Rold\\'an"
    ) == [
        "Raphael Chetrite",
        "Shamik Gupta",
        "Izaak Neri",
        "Édgar Roldán",
    ]
    assert parse_arxiv_authors("Izaak Neri and \\'Edgar Rold\\'an") == [
        "Izaak Neri",
        "Édgar Roldán",
    ]
    assert parse_arxiv_authors("Izaak Neri, and \\'Edgar Rold\\'an") == [
        "Izaak Neri",
        "Édgar Roldán",
    ]
    assert parse_arxiv_authors("Izaak Neri, et al.") == [
        "Izaak Neri",
        "et al.",
    ]
    assert parse_arxiv_authors("Raphael Chetrite Shamik Gupta") == [
        "Raphael Chetrite Shamik Gupta",
    ]

    assert parse_arxiv_authors(
        "B. P. Lanyon, T. J. Weinhold, N. K. Langford, M. Barbieri, D. F. V.  James*, A. Gilchrist, and A. G. White (University of Queensland, *University of Toronto)"
    ) == [
        "B. P. Lanyon",
        "T. J. Weinhold",
        "N. K. Langford",
        "M. Barbieri",
        "D. F. V.  James",
        "A. Gilchrist",
        "A. G. White",
    ]


class ArxivRawImporter(EntityImporter):
    """
    Converts arxiv.org "arXivRaw" OAI-PMH XML records to fatcat release entities

    TODO: arxiv_id lookup in API (rust) with no version specified should select
          the "most recent" version; can be a simple sort?
    """

    def __init__(self, api: ApiClient, **kwargs) -> None:

        eg_desc = kwargs.get(
            "editgroup_description",
            "Automated import of arxiv metadata via arXivRaw OAI-PMH feed",
        )
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.ArxivRawImporter")
        # lower batch size, because multiple versions per entry (guessing 2-3 on average?)
        batch_size = kwargs.get("edit_batch_size", 50)
        super().__init__(
            api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            batch_size=batch_size,
            **kwargs
        )
        self._test_override = False

    # TODO: record is really a beautiful soup element, but setting to 'Any' to
    # make initial type annotations simple
    def parse_record(self, record: Any) -> Optional[List[ReleaseEntity]]:

        if not record:
            return None
        metadata = record.arXivRaw
        if not metadata:
            return None
        extra: Dict[str, Any] = dict()
        extra_arxiv: Dict[str, Any] = dict()

        # don't know!
        release_type = "article"

        base_id = metadata.id.string
        doi = None
        if metadata.doi and metadata.doi.string:
            doi = clean_doi(metadata.doi.string.lower().split()[0].strip())
            if doi and not (doi.startswith("10.") and "/" in doi and doi.split("/")[1]):
                sys.stderr.write("BOGUS DOI: {}\n".format(doi))
                doi = None
        title = latex_to_text(metadata.title.get_text().replace("\n", " "))
        authors = parse_arxiv_authors(metadata.authors.get_text().replace("\n", " "))
        contribs = [
            fatcat_openapi_client.ReleaseContrib(index=i, raw_name=a, role="author")
            for i, a in enumerate(authors)
        ]

        lang: Optional[str] = "en"  # the vast majority in english
        if metadata.comments and metadata.comments.get_text():
            comments = metadata.comments.get_text().replace("\n", " ").strip()
            extra_arxiv["comments"] = comments
            if "in french" in comments.lower():
                lang = "fr"
            elif "in spanish" in comments.lower():
                lang = "es"
            elif "in portuguese" in comments.lower():
                lang = "pt"
            elif "in hindi" in comments.lower():
                lang = "hi"
            elif "in japanese" in comments.lower():
                lang = "ja"
            elif "in german" in comments.lower():
                lang = "de"
            elif "simplified chinese" in comments.lower():
                lang = "zh"
            elif "in russian" in comments.lower():
                lang = "ru"
            # more languages?

        number = None
        if metadata.find("journal-ref") and metadata.find("journal-ref").get_text():
            journal_ref = metadata.find("journal-ref").get_text().replace("\n", " ").strip()
            extra_arxiv["journal_ref"] = journal_ref
            if "conf." in journal_ref.lower() or "proc." in journal_ref.lower():
                release_type = "paper-conference"
        if metadata.find("report-no") and metadata.find("report-no").string:
            number = metadata.find("report-no").string.strip()
            # at least some people plop extra metadata in here. hrmf!
            if "ISSN " in number or "ISBN " in number or len(number.split()) > 2:
                extra_arxiv["report-no"] = number
                number = None
            else:
                release_type = "report"
        if metadata.find("acm-class") and metadata.find("acm-class").string:
            extra_arxiv["acm_class"] = metadata.find("acm-class").string.strip()
        if metadata.categories and metadata.categories.get_text():
            extra_arxiv["categories"] = metadata.categories.get_text().split()
        license_slug = None
        if metadata.license and metadata.license.get_text():
            license_slug = lookup_license_slug(metadata.license.get_text())
        abstracts = None
        if metadata.abstract:
            # TODO: test for this multi-abstract code path
            abstracts = []
            abst = metadata.abstract.get_text().strip()
            orig = None
            if "-----" in abst:
                both = abst.split("-----")
                abst = both[0].strip()
                orig = both[1].strip()
            if "$" in abst or "{" in abst:
                mime = "application/x-latex"
                abst_plain = latex_to_text(abst)
                abstracts.append(
                    fatcat_openapi_client.ReleaseAbstract(
                        content=abst_plain, mimetype="text/plain", lang="en"
                    )
                )
            else:
                mime = "text/plain"
            abstracts.append(
                fatcat_openapi_client.ReleaseAbstract(content=abst, mimetype=mime, lang="en")
            )
            if orig:
                abstracts.append(
                    fatcat_openapi_client.ReleaseAbstract(content=orig, mimetype=mime)
                )
                # indicates that fulltext probably isn't english either
                if lang == "en":
                    lang = None

        # extra:
        #   withdrawn_date
        #   translation_of
        #   subtitle
        #   aliases
        #   container_name
        #   group-title
        #   arxiv: comments, categories, etc
        extra_arxiv["base_id"] = base_id
        extra["superceded"] = True
        extra["arxiv"] = extra_arxiv

        versions = []
        for version in metadata.find_all("version"):
            arxiv_id = base_id + version["version"]
            release_date = version.date.string.strip()
            release_date = datetime.datetime.strptime(
                release_date, "%a, %d %b %Y %H:%M:%S %Z"
            ).date()
            # TODO: source_type?
            versions.append(
                ReleaseEntity(
                    work_id=None,
                    title=title,
                    # original_title
                    version=version["version"],
                    release_type=release_type,
                    release_stage="submitted",
                    release_date=release_date.isoformat(),
                    release_year=release_date.year,
                    ext_ids=fatcat_openapi_client.ReleaseExtIds(
                        arxiv=arxiv_id,
                    ),
                    number=number,
                    language=lang,
                    license_slug=license_slug,
                    abstracts=abstracts,
                    contribs=contribs,
                    extra=extra.copy(),
                )
            )
        # TODO: assert that versions are actually in order?
        assert versions

        versions[-1].extra.pop("superceded")

        # only apply DOI to most recent version (HACK)
        if doi:
            versions[-1].ext_ids.doi = doi
            if len(versions) > 1:
                versions[-1].release_stage = "accepted"
        return versions

    def try_update(self, versions: List[ReleaseEntity]) -> bool:
        """
        This is pretty complex! There is no batch/bezerk mode for arxiv importer.

        For each version, do a lookup by full arxiv_id, and store work/release
        id results.

        If a version has a DOI, also do a doi lookup and store that result. If
        there is an existing release with both matching, set that as the
        existing work. If they don't match, use the full arxiv_id match and
        move on (maybe log or at least count the error?). If it's a
        one/or/other case, update the existing release (and mark version as
        existing).

        If there was any existing release, take its work_id.

        Iterate back through versions. If it didn't already exist, insert it
        with any existing work_id. If there wasn't an existing work_id, lookup
        the new release (by rev from edit?) and use that for the rest.

        Do not pass any versions on for batch insert.
        """

        # first do lookups
        any_work_id = None
        for v in versions:
            v._existing_work_id = None
            v._updated = False
            existing = None
            existing_doi = None
            try:
                existing = self.api.lookup_release(arxiv=v.ext_ids.arxiv)
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status != 404:
                    raise err

            if existing:
                v._existing_work_id = existing.work_id
                any_work_id = existing.work_id

            if v.ext_ids.doi:
                try:
                    existing_doi = self.api.lookup_release(doi=v.ext_ids.doi)
                except fatcat_openapi_client.rest.ApiException as err:
                    if err.status != 404:
                        raise err
            if existing_doi:
                if existing and existing.ident == existing_doi.ident:
                    # great, they match and have idents, nothing to do
                    pass
                elif existing and existing.ident != existing_doi.ident:
                    # could be that a new arxiv version was created (update?),
                    # or that VOR has no arxiv version (or catalog is borked or
                    # something else)
                    # stick with arxiv_id match as existing, but don't set DOI;
                    # don't update anything
                    v.ext_ids.doi = None
                    pass
                else:
                    assert not existing
                    # there's a pre-existing DOI release we should group under,
                    # but we don't know if we're the version-of-record or what,
                    # so just group but don't update existing DOI release
                    v.ext_ids.doi = None
                    any_work_id = any_work_id or existing_doi.work_id

        last_edit = None
        for v in versions:
            if v._existing_work_id:
                if not v._updated:
                    self.counts["exists"] += 1
                continue
            if not any_work_id and last_edit:
                # fetch the last inserted release from this group
                r = self.api.get_release_revision(last_edit.revision)
                assert r.work_id
                any_work_id = r.work_id
            v.work_id = any_work_id
            last_edit = self.api.create_release(self.get_editgroup_id(), v)
            self.counts["insert"] += 1

        return False

    def insert_batch(self, batch_batch: List[ReleaseEntity]) -> None:
        # there is no batch/bezerk mode for arxiv importer, except for testing
        if self._test_override:
            for batch in batch_batch:
                self.api.create_release_auto_batch(
                    fatcat_openapi_client.ReleaseAutoBatch(
                        editgroup=fatcat_openapi_client.Editgroup(
                            description=self.editgroup_description, extra=self.editgroup_extra
                        ),
                        entity_list=batch,
                    )
                )
                self.counts["insert"] += len(batch) - 1
        else:
            raise NotImplementedError()

    def parse_file(self, handle: Any) -> None:

        # 1. open with beautiful soup
        soup = BeautifulSoup(handle, "xml")

        # 2. iterate over articles, call parse_article on each
        for article in soup.find_all("record"):
            resp = self.parse_record(article)
            print(json.dumps(resp))
            # sys.exit(-1)
