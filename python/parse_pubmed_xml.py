
import sys
import json
import datetime
from bs4 import BeautifulSoup
from bs4.element import NavigableString

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

class PubMedParser():
    """
    Converts PubMed/MEDLINE XML into in release entity (which can dump as JSON)

    TODO: MEDLINE doesn't include PMC/OA license; could include in importer?
    TODO: clean (ftfy) title, original title, etc
    """

    def __init__(self):
        pass

    def parse_file(self, handle):

        # 1. open with beautiful soup
        soup = BeautifulSoup(handle, "xml")

        # 2. iterate over articles, call parse_article on each
        for article in soup.find_all("PubmedArticle"):
            resp = self.parse_article(article)
            print(json.dumps(resp))
            #sys.exit(-1)

    def parse_article(self, a):

        medline = a.MedlineCitation
        # PubmedData isn't required by DTD, but seems to always be present
        pubmed = a.PubmedData
        extra = dict()
        extra_pubmed = dict()

        identifiers = pubmed.ArticleIdList
        doi = identifiers.find("ArticleId", IdType="doi")
        if doi:
            doi = doi.string.lower()

        pmcid = identifiers.find("ArticleId", IdType="pmc")
        if pmcid:
            pmcid = pmcid.string

        release_type = None
        for pub_type in medline.Article.PublicationTypeList.find_all("PublicationType"):
            if pub_type.string in PUBMED_RELEASE_TYPE_MAP:
                release_type = PUBMED_RELEASE_TYPE_MAP[pub_type.string]
            break
        if medline.Article.PublicationTypeList.find(string="Retraction of Publication"):
            release_type = "retraction"
            retraction_of = medline.find("CommentsCorrections", RefType="RetractionOf")
            if retraction_of:
                extra_pubmed['retraction_of_raw'] = retraction_of.RefSource.string
                extra_pubmed['retraction_of_pmid'] = retraction_of.PMID.string

        # everything in medline is published
        release_status = "published"
        if medline.Article.PublicationTypeList.find(string="Corrected and Republished Article"):
            release_status = "updated"
        if medline.Article.PublicationTypeList.find(string="Retracted Publication"):
            release_status = "retracted"

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
            # TODO: will filter out later
            title = None

        original_title = medline.Article.find("VernacularTitle", recurse=False)
        if original_title:
            original_title = original_title.string or None
            if original_title and original_title.endswith('.'):
                original_title = original_title[:-1]

        # TODO: happening in alpha order, not handling multi-language well.
        # also need to convert lang codes: https://www.nlm.nih.gov/bsd/language_table.html
        language = medline.Article.Language
        if language:
            language = language.string
            # TODO: map to two-letter
            if language in ("und", "un"):
                # "undetermined"
                language = None

        ### Journal/Issue Metadata
        # MedlineJournalInfo is always present
        container = dict()
        container_extra = dict()
        mji = medline.MedlineJournalInfo
        if mji.find("Country"):
            container_extra['country_name'] = mji.Country.string
        if mji.find("ISSNLinking"):
            container['issnl'] = mji.ISSNLinking.string

        journal = medline.Article.Journal
        issnp = journal.find("ISSN", IssnType="Print")
        if issnp:
            container_extra['issnp'] = issnp.string

        pub_date = journal.PubDate
        release_date = None
        if pub_date.find("MedlineDate"):
            release_year = int(pub_date.MedlineDate.string.split()[0][:4])
        else:
            release_year = int(pub_date.Year.string)
            if pub_date.find("Day") and pub_date.find("Month"):
                release_date = datetime.date(
                    release_year,
                    MONTH_ABBR_MAP[pub_date.Month.string],
                    int(pub_date.Day.string))
                release_date = release_date.isoformat()
       
        ji = journal.JournalIssue
        volume = None
        if ji.find("Volume"):
            volume = ji.Volume.string
        issue = None
        if ji.find("Issue"):
            issue = ji.Issue.string
        if journal.find("Title"):
            container['name'] = journal.Title.string

        if extra_pubmed:
            extra['pubmed'] = extra_pubmed
        if not extra:
            extra = None

        ### Abstracts
        # "All abstracts are in English"
        abstracts = []
        first_abstract = medline.find("AbstractText")
        if first_abstract and first_abstract.get('NlmCategory'):
            joined = "\n".join([m.get_text() for m in medline.find_all("AbstractText")])
            abstracts.append(dict(
                content=joined,
                mimetype="text/plain",
                lang="en",
            ))
        else:
            for abstract in medline.find_all("AbstractText"):
                abstracts.append(dict(
                    content=abstract.get_text().strip(),
                    mimetype="text/plain",
                    lang="en",
                ))
                if abstract.find('math'):
                    abstracts.append(dict(
                        # strip the <AbstractText> tags
                        content=str(abstract)[14:-15],
                        mimetype="application/mathml+xml",
                        lang="en",
                    ))
        if not abstracts:
            abstracts = None

        ### Contribs
        contribs = []
        if medline.AuthorList:
            for author in medline.AuthorList.find_all("Author"):
                contrib = dict(
                    role="author",
                )
                if author.ForeName:
                    contrib['raw_name'] = "{} {}".format(author.ForeName.string, author.LastName.string)
                elif author.LastName:
                    contrib['raw_name'] = author.LastName.string
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
                    contrib_extra['orcid'] = orcid
                affiliation = author.find("Affiliation")
                if affiliation:
                    contrib['raw_affiliation'] = affiliation.string
                if author.find("EqualContrib"):
                    # TODO: schema for this?
                    contrib_extra['equal_contrib'] = True
                if contrib_extra:
                    contrib['extra'] = contrib_extra
                contribs.append(contrib)

            if medline.AuthorList['CompleteYN'] == 'N':
                contribs.append(dict(raw_name="et al."))
        if not contribs:
            contribs = None

        ### References
        refs = []
        if pubmed.ReferenceList:
            for ref in pubmed.ReferenceList.find_all('Reference'):
                ref_obj = dict()
                ref_extra = dict()
                ref_pmid = ref.find("ArticleId", IdType="pubmed")
                if ref_pmid:
                    ref_extra['pmid'] = ref_pmid.string
                ref_raw = ref.Citation
                if ref_raw:
                    ref_extra['raw'] = ref_raw.string
                if ref_extra:
                    ref_obj['extra'] = ref_extra
                refs.append(ref_obj)
        if not refs:
            refs = None

        re = dict(
            work_id=None,
            title=title,
            original_title=original_title,
            release_type=release_type,
            release_status=release_status,
            release_date=release_date,
            release_year=release_year,
            doi=doi,
            pmid=int(medline.PMID.string), # always present
            pmcid=pmcid,
            #isbn13     # never in Article
            volume=volume,
            issue=issue,
            pages=pages,
            #publisher  # not included?
            language=language,
            #license_slug   # not in MEDLINE

            # content, mimetype, lang
            abstracts=abstracts,

            # raw_name, role, raw_affiliation, extra
            contribs=contribs,

            # key, year, container_name, title, locator
            # extra: volume, authors, issue, publisher, identifiers
            refs=refs,

            #   name, type, publisher, issnl
            #   extra: issnp, issne, original_name, languages, country
            container=container,

            # extra:
            #   withdrawn_date
            #   translation_of
            #   subtitle
            #   aliases
            #   container_name
            #   group-title
            #   pubmed: retraction refs
            extra=extra,
        )

        return re

if __name__=='__main__':
    parser = PubMedParser()
    parser.parse_file(open(sys.argv[1]))
