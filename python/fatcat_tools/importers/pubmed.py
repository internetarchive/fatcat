
import sys
import json
import sqlite3
import datetime
import warnings
from bs4 import BeautifulSoup
from bs4.element import NavigableString

import fatcat_client
from .common import EntityImporter, clean, LANG_MAP_MARC

# from: https://www.ncbi.nlm.nih.gov/books/NBK3827/table/pubmedhelp.T.publication_types/?report=objectonly
PUBMED_RELEASE_TYPE_MAP = {
    #Adaptive Clinical Trial
    "Address": "speech",
    "Autobiography": "book",
    #Bibliography
    "Biography": "book",
    #Case Reports
    "Classical Article": "article-journal",
    #Clinical Conference
    #Clinical Study
    #Clinical Trial
    #Clinical Trial, Phase I
    #Clinical Trial, Phase II
    #Clinical Trial, Phase III
    #Clinical Trial, Phase IV
    #Clinical Trial Protocol
    #Clinical Trial, Veterinary
    #Collected Works
    #Comparative Study
    #Congress
    #Consensus Development Conference
    #Consensus Development Conference, NIH
    #Controlled Clinical Trial
    "Dataset": "dataset",
    #Dictionary
    #Directory
    #Duplicate Publication
    "Editorial": "editorial",
    #English Abstract   # doesn't indicate that this is abstract-only
    #Equivalence Trial
    #Evaluation Studies
    #Expression of Concern
    #Festschrift
    #Government Document
    #Guideline
    "Historical Article": "article-journal",
    #Interactive Tutorial
    "Interview": "interview",
    "Introductory Journal Article": "article-journal",
    "Journal Article": "article-journal",
    "Lecture": "speech",
    "Legal Case": "legal_case",
    "Legislation": "legislation",
    "Letter": "letter",
    #Meta-Analysis
    #Multicenter Study
    #News
    "Newspaper Article": "article-newspaper",
    #Observational Study
    #Observational Study, Veterinary
    #Overall
    #Patient Education Handout
    #Periodical Index
    #Personal Narrative
    #Portrait
    #Practice Guideline
    #Pragmatic Clinical Trial
    #Publication Components
    #Publication Formats
    #Publication Type Category
    #Randomized Controlled Trial
    #Research Support, American Recovery and Reinvestment Act
    #Research Support, N.I.H., Extramural
    #Research Support, N.I.H., Intramural
    #Research Support, Non-U.S. Gov't Research Support, U.S. Gov't, Non-P.H.S.
    #Research Support, U.S. Gov't, P.H.S.
    #Review     # in the "literature review" sense, not "product review"
    #Scientific Integrity Review
    #Study Characteristics
    #Support of Research
    #Systematic Review
    "Technical Report": "report",
    #Twin Study
    #Validation Studies
    #Video-Audio Media
    #Webcasts
}

MONTH_ABBR_MAP = {
    "Jan":  1, "01":  1,
    "Feb":  2, "02":  2,
    "Mar":  3, "03":  3,
    "Apr":  4, "04":  4,
    "May":  5, "05":  5,
    "Jun":  6, "06":  6,
    "Jul":  7, "07":  7,
    "Aug":  8, "08":  8,
    "Sep":  9, "09":  9,
    "Oct": 10, "10": 10,
    "Nov": 11, "11": 11,
    "Dec": 12, "12": 12,
}


class PubmedImporter(EntityImporter):
    """
    Importer for PubMed/MEDLINE XML metadata.

    If lookup_refs is true, will do identifer-based lookups for all references.
    
    TODO: MEDLINE doesn't include PMC/OA license; could include in importer?
    """

    def __init__(self, api, issn_map_file, lookup_refs=False, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Automated import of PubMed/MEDLINE XML metadata")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.PubmedImporter')
        super().__init__(api,
            issn_map_file=issn_map_file,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

        self.lookup_refs = lookup_refs
        extid_map_file = kwargs.get('extid_map_file')
        self.extid_map_db = None
        if extid_map_file:
            db_uri = "file:{}?mode=ro".format(extid_map_file)
            print("Using external ID map: {}".format(db_uri))
            self.extid_map_db = sqlite3.connect(db_uri, uri=True)
        else:
            print("Not using external ID map")

        self.create_containers = kwargs.get('create_containers', True)
        self.read_issn_map_file(issn_map_file)

    def lookup_ext_ids(self, pmid):
        if self.extid_map_db is None:
            return dict(doi=None, core_id=None, pmid=None, pmcid=None,
                wikidata_qid=None, arxiv_id=None, jstor_id=None)
        row = self.extid_map_db.execute("SELECT core, doi, pmcid, wikidata FROM ids WHERE pmid=? LIMIT 1",
            [pmid]).fetchone()
        if row is None:
            return dict(doi=None, core_id=None, pmid=None, pmcid=None,
                wikidata_qid=None, arxiv_id=None, jstor_id=None)
        row = [str(cell or '') or None for cell in row]
        return dict(
            core_id=row[0],
            doi=row[1],
            pmcid=row[2],
            wikidata_qid=row[3],
            # TODO:
            arxiv_id=None,
            jstor_id=None,
        )

    def want(self, obj):
        return True

    def parse_record(self, a):

        medline = a.MedlineCitation
        # PubmedData isn't required by DTD, but seems to always be present
        pubmed = a.PubmedData
        extra = dict()
        extra_pubmed = dict()

        identifiers = pubmed.ArticleIdList
        pmid = medline.PMID.string.strip()
        doi = identifiers.find("ArticleId", IdType="doi")
        if doi:
            doi = doi.string.lower()

        pmcid = identifiers.find("ArticleId", IdType="pmc")
        if pmcid:
            pmcid = pmcid.string

        release_type = None
        pub_types = []
        for pub_type in medline.Article.PublicationTypeList.find_all("PublicationType"):
            pub_types.append(pub_type.string)
            if pub_type.string in PUBMED_RELEASE_TYPE_MAP:
                release_type = PUBMED_RELEASE_TYPE_MAP[pub_type.string]
                break
        if pub_types:
            extra_pubmed['pub_types'] = pub_types
        if medline.Article.PublicationTypeList.find(string="Retraction of Publication"):
            release_type = "retraction"
            retraction_of = medline.find("CommentsCorrections", RefType="RetractionOf")
            if retraction_of:
                extra_pubmed['retraction_of_raw'] = retraction_of.RefSource.string
                extra_pubmed['retraction_of_pmid'] = retraction_of.PMID.string

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

        pages = medline.find('MedlinePgn')
        if pages:
            pages = pages.string

        title = medline.Article.ArticleTitle.string # always present
        if title:
            if title.endswith('.'):
                title = title[:-1]
            # this hides some "special" titles, but the vast majority are
            # translations; translations don't always include the original_title
            if title.startswith('[') and title.endswith(']'):
                title = title[1:-1]
        else:
            # will filter out later
            title = None

        original_title = medline.Article.find("VernacularTitle", recurse=False)
        if original_title:
            original_title = original_title.string or None
            if original_title and original_title.endswith('.'):
                original_title = original_title[:-1]

        # TODO: happening in alpha order, not handling multi-language well.
        language = medline.Article.Language
        if language:
            language = language.string
            if language in ("und", "un"):
                # "undetermined"
                language = None
            else:
                language = LANG_MAP_MARC.get(language)
                if not language:
                    warnings.warn("MISSING MARC LANG: {}".format(medline.Article.Language.string))

        ### Journal/Issue Metadata
        # MedlineJournalInfo is always present
        issnl = None
        container_id = None
        container_name = None
        container_extra = dict()
        mji = medline.MedlineJournalInfo
        if mji.find("Country"):
            container_extra['country_name'] = mji.Country.string
        if mji.find("ISSNLinking"):
            issnl = mji.ISSNLinking.string

        journal = medline.Article.Journal
        issnp = journal.find("ISSN", IssnType="Print")
        if issnp:
            container_extra['issnp'] = issnp.string
        if not issnl:
            issnll = self.issn2issnl(issnp)

        if issnl:
            container_id = self.lookup_issnl(issnl)

        pub_date = medline.Article.find('ArticleDate')
        if not pub_date:
            pub_date = journal.PubDate
        release_date = None
        release_year = None
        if pub_date.Year:
            release_year = int(pub_date.Year.string)
            if pub_date.find("Day") and pub_date.find("Month"):
                release_date = datetime.date(
                    release_year,
                    MONTH_ABBR_MAP[pub_date.Month.string],
                    int(pub_date.Day.string))
                release_date = release_date.isoformat()

        if journal.find("Title"):
            container_name = journal.Title.string

        if (container_id is None and self.create_containers and (issnl is not None)
                and container_name):
            # name, type, publisher, issnl
            # extra: issnp, issne, original_name, languages, country
            ce = fatcat_client.ContainerEntity(
                name=container_name,
                container_type='journal',
                #NOTE: publisher not included
                issnl=issnl,
                extra=(container_extra or None))
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
        if primary_abstract and primary_abstract.AbstractText.get('NlmCategory'):
            joined = "\n".join([m.get_text() for m in primary_abstract.find_all("AbstractText")])
            abstracts.append(fatcat_client.ReleaseAbstract(
                content=joined,
                mimetype="text/plain",
                lang="en",
            ))
        elif primary_abstract:
            for abstract in primary_abstract.find_all("AbstractText"):
                abstracts.append(fatcat_client.ReleaseAbstract(
                    content=abstract.get_text().strip(),
                    mimetype="text/plain",
                    lang="en",
                ))
                if abstract.find('math'):
                    abstracts.append(fatcat_client.ReleaseAbstract(
                        # strip the <AbstractText> tags
                        content=str(abstract)[14:-15],
                        mimetype="application/mathml+xml",
                        lang="en",
                    ))
        other_abstracts = medline.find_all("OtherAbstract")
        for other in other_abstracts:
            lang = "en"
            if other.get('Language'):
                lang = LANG_MAP_MARC.get(other['Language'])
            abstracts.append(fatcat_client.ReleaseAbstract(
                content=other.AbstractText.get_text().strip(),
                mimetype="text/plain",
                lang=lang,
            ))
        if not abstracts:
            abstracts = None

        ### Contribs
        contribs = []
        if medline.AuthorList:
            for author in medline.AuthorList.find_all("Author"):
                creator_id = None
                given_name = None
                surname = None
                raw_name = None
                if author.ForeName:
                    given_name = author.ForeName.string
                if author.LastName:
                    surname = author.LastName.string
                if given_name and surname:
                    raw_name = "{} {}".format(given_name, surname)
                elif surname:
                    raw_name = surname
                contrib_extra = dict()
                orcid = author.find("Identifier", Source="ORCID")
                if orcid:
                    # needs re-formatting from, eg, "0000000179841889"
                    orcid = orcid.string
                    if orcid.startswith("http://orcid.org/"):
                        orcid = orcid.replace("http://orcid.org/", "")
                    elif orcid.startswith("https://orcid.org/"):
                        orcid = orcid.replace("https://orcid.org/", "")
                    elif not '-' in orcid:
                        orcid = "{}-{}-{}-{}".format(
                            orcid[0:4],
                            orcid[4:8],
                            orcid[8:12],
                            orcid[12:16],
                        )
                    creator_id = self.lookup_orcid(orcid)
                    contrib_extra['orcid'] = orcid
                affiliations = author.find_all("Affiliation")
                raw_affiliation = None
                if affiliations:
                    raw_affiliation = affiliations[0].string
                    if len(affiliations) > 1:
                        contrib_extra['more_affiliations'] = [ra.string for ra in affiliations[1:]]
                if author.find("EqualContrib"):
                    # TODO: schema for this?
                    contrib_extra['equal'] = True
                contribs.append(fatcat_client.ReleaseContrib(
                    raw_name=raw_name,
                    given_name=given_name,
                    surname=surname,
                    role="author",
                    raw_affiliation=raw_affiliation,
                    creator_id=creator_id,
                    extra=contrib_extra,
                ))

            if medline.AuthorList['CompleteYN'] == 'N':
                contribs.append(fatcat_client.ReleaseContrib(raw_name="et al."))
        if not contribs:
            contribs = None

        ### References
        refs = []
        if pubmed.ReferenceList:
            for ref in pubmed.ReferenceList.find_all('Reference'):
                ref_extra = dict()
                ref_pmid = ref.find("ArticleId", IdType="pubmed")
                ref_doi = ref.find("ArticleId", IdType="doi")
                ref_release_id = None
                if ref_pmid:
                    ref_pmid = ref_pmid.string.strip()
                    ref_extra['pmid'] = ref_pmid
                    if self.lookup_refs:
                        ref_release_id = self.lookup_pmid(ref_pmid)
                if ref_doi:
                    ref_doi = ref_doi.string.lower().strip()
                    ref_extra['doi'] = ref_doi
                    if self.lookup_refs:
                        ref_release_id = self.lookup_doi(ref_doi)
                ref_raw = ref.Citation
                if ref_raw:
                    ref_extra['unstructured'] = ref_raw.string
                if not ref_extra:
                    ref_extra = None
                refs.append(fatcat_client.ReleaseRef(
                    target_release_id=ref_release_id,
                    extra=ref_extra,
                ))
        if not refs:
            refs = None

        # extra:
        #   translation_of
        #   subtitle
        #   aliases
        #   container_name
        #   group-title
        #   pubmed: retraction refs
        if extra_pubmed:
            extra['pubmed'] = extra_pubmed
        if not extra:
            extra = None

        title = clean(title)
        if not title:
            return None

        re = fatcat_client.ReleaseEntity(
            work_id=None,
            title=title,
            original_title=clean(original_title),
            release_type=release_type,
            release_stage=release_stage,
            release_date=release_date,
            release_year=release_year,
            withdrawn_status=withdrawn_status,
            ext_ids=fatcat_client.ReleaseExtIds(
                doi=doi,
                pmid=pmid,
                pmcid=pmcid,
                #isbn13     # never in Article
            ),
            volume=volume,
            issue=issue,
            pages=pages,
            #publisher  # not included?
            language=language,
            #license_slug   # not in MEDLINE
            abstracts=abstracts,
            contribs=contribs,
            refs=refs,
            container_id=container_id,
            extra=extra,
        )
        return re

    def try_update(self, re):

        # first, lookup existing by PMID (which must be defined)
        existing = None
        try:
            existing = self.api.lookup_release(pmid=re.ext_ids.pmid)
        except fatcat_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        # then try DOI lookup if there is one
        if not existing and re.ext_ids.doi:
            try:
                existing = self.api.lookup_release(doi=re.ext_ids.doi)
            except fatcat_client.rest.ApiException as err:
                if err.status != 404:
                    raise err
            if existing and existing.ext_ids.pmid and existing.ext_ids.pmid != re.ext_ids.pmid:
                warnings.warn("PMID/DOI mismatch: release {}, pmid {} != {}".format(
                    existing.ident, existing.ext_ids.pmid, re.ext_ids.pmid))
                self.counts['exists-pmid-doi-mismatch'] += 1
                return False

        if existing and existing.ext_ids.pmid and (existing.refs or not re.refs):
            # TODO: any other reasons to do an update?
            # don't update if it already has PMID
            self.counts['exists'] += 1
            return False
        elif existing:
            # but do update if only DOI was set
            existing.ext_ids.doi = existing.ext_ids.doi or re.ext_ids.doi
            existing.ext_ids.pmid = existing.ext_ids.pmid or re.ext_ids.pmid
            existing.ext_ids.pmcid = existing.ext_ids.pmcid or re.ext_ids.pmcid
            existing.refs = existing.refs or re.refs
            existing.extra['pubmed'] = re.extra['pubmed']
            self.api.update_release(self.get_editgroup_id(), existing.ident, existing)
            self.counts['update'] += 1
            return False

        return True

    def insert_batch(self, batch):
        self.api.create_release_auto_batch(fatcat_client.ReleaseAutoBatch(
            editgroup=fatcat_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))

    def parse_file(self, handle):

        # 1. open with beautiful soup
        soup = BeautifulSoup(handle, "xml")

        # 2. iterate over articles, call parse_article on each
        for article in soup.find_all("PubmedArticle"):
            resp = self.parse_record(article)
            print(json.dumps(resp))
            #sys.exit(-1)

if __name__=='__main__':
    parser = PubmedImporter(None, None)
    parser.parse_file(open(sys.argv[1]))
