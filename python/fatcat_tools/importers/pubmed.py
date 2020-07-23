
import sys
import json
import datetime
import warnings
from bs4 import BeautifulSoup

import fatcat_openapi_client
from fatcat_tools.normal import *
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

# From: https://www.ncbi.nlm.nih.gov/books/NBK7249/
COUNTRY_NAME_MAP = {
    "Afghanistan": "af",
    "Albania": "al",
    "Algeria": "dz",
    "Andorra": "ad",
    "Angola": "ao",
    "Antigua and Barbuda": "ag",
    "Argentina": "ar",
    "Armenia": "am",
    "Australia": "au",
    "Austria": "at",
    "Azerbaijan": "az",
    "Bahamas": "bs",
    "Bahrain": "bh",
    "Bangladesh": "bd",
    "Barbados": "bb",
    "Belarus": "by",
    "Belgium": "be",
    "Belize": "bz",
    "Benin": "bj",
    "Bhutan": "bt",
    "Bolivia": "bo",
    "Bosnia and Herzegowina": "ba",
    "Botswana": "bw",
    "Brazil": "br",
    "Brunei Darussalam": "bn",
    "Bulgaria": "bg",
    "Burkina Faso": "bf",
    "Burundi": "bi",
    "Cambodia": "kh",
    "Cameroon": "cm",
    "Canada": "ca",
    "Cape Verde": "cv",
    "Central African Republic": "cf",
    "Chad": "td",
    "Chile": "cl",
    "China": "cn",
    "Colombia": "co",
    "Comoros": "km",
    "Congo, Democratic Republic": "cd",
    "Congo, People’s Republic": "cg",
    "Costa Rica": "cr",
    "Cote d'Ivoire": "ci",
    "Croatia (Local Name: Hrvatska)": "hr",
    "Cuba": "cu",
    "Cyprus": "cy",
    "Czech Republic": "cz",
    "Denmark": "dk",
    "Djibouti": "dj",
    "Dominica": "dm",
    "Dominican Republic": "do",
    "East Timor": "tl",
    "Ecuador": "ec",
    "El Salvador": "sv",
    "Equatorial Guinea": "gq",
    "Eritrea": "er",
    "Estonia": "ee",
    "Ethiopia": "et",
    "Fiji": "fj",
    "Finland": "fi",
    "France": "fr",
    "Gabon": "ga",
    "Gambia": "gm",
    "Georgia": "ge",
    "Germany": "de",
    "Ghana": "gh",
    "Greece": "gr",
    "Greenland": "gl",
    "Grenada": "gd",
    "Guatemala": "gt",
    "Guinea": "gn",
    "Guinea-Bissau": "gw",
    "Guyana": "gy",
    "Haiti": "ht",
    "Honduras": "hn",
    "Hong Kong": "hk",
    "Hungary": "hu",
    "Iceland": "is",
    "India": "in",
    "Indonesia": "id",
    "Iran": "ir",
    "Iraq": "iq",
    "Ireland": "ie",
    "Israel": "il",
    "Italy": "it",
    "Jamaica": "jm",
    "Japan": "jp",
    "Jordan": "jo",
    "Kazakhstan": "kz",
    "Kenya": "ke",
    "Kiribati": "ki",
    "Korea, Democratic People's Republic": "kp",
    "Korea, Republic": "kr",
    "Kuwait": "kw",
    "Kyrgyzstan": "kg",
    "Laos": "la",
    "Latvia": "lv",
    "Lebanon": "lb",
    "Lesotho": "ls",
    "Liberia": "lr",
    "Libya": "ly",
    "Liechtenstein": "li",
    "Lithuania": "lt",
    "Luxembourg": "lu",
    "Macedonia": "mk",
    "Madagascar": "mg",
    "Malawi": "mw",
    "Malaysia": "my",
    "Maldives": "mv",
    "Mali": "ml",
    "Malta": "mt",
    "Marshall Islands": "mh",
    "Mauritania": "mr",
    "Mauritius": "mu",
    "Mexico": "mx",
    "Micronesia": "fm",
    "Moldova": "md",
    "Monaco": "mc",
    "Mongolia": "mn",
    "Morocco": "ma",
    "Mozambique": "mz",
    "Myanmar": "mm",
    "Namibia": "na",
    "Nauru": "nr",
    "Nepal": "np",
    "Netherlands": "nl",
    "New Zealand": "nz",
    "Nicaragua": "ni",
    "Niger": "ne",
    "Nigeria": "ng",
    "Norway": "no",
    "Oman": "om",
    "Pakistan": "pk",
    "Palau": "pw",
    "Panama": "pa",
    "Papua New Guinea": "pg",
    "Paraguay": "py",
    "Peru": "pe",
    "Philippines": "ph",
    "Poland": "pl",
    "Portugal": "pt",
    "Puerto Rico": "pr",
    "Qatar": "qa",
    "Romania": "ro",
    "Russian Federation": "ru",
    "Rwanda": "rw",
    "Saint Kitts and Nevis": "kn",
    "Saint Lucia": "lc",
    "Saint Vincent and the Grenadines": "vc",
    "Samoa": "ws",
    "San Marino": "sm",
    "Sao Tome and Príncipe": "st",
    "Saudi Arabia": "sa",
    "Senegal": "sn",
    "Serbia and Montenegro": "cs",
    "Seychelles": "sc",
    "Sierra Leone": "sl",
    "Singapore": "sg",
    "Slovakia (Slovak Republic)": "sk",
    "Slovenia": "si",
    "Solomon Islands": "sb",
    "Somalia": "so",
    "South Africa": "za",
    "Spain": "es",
    "Sri Lanka": "lk",
    "Sudan": "sd",
    "Suriname": "sr",
    "Swaziland": "sz",
    "Sweden": "se",
    "Switzerland": "ch",
    "Syrian Arab Republic": "sy",
    "Taiwan": "tw",
    "Tajikistan": "tj",
    "Tanzania": "tz",
    "Tanzania": "tz",
    "Thailand": "th",
    "Togo": "tg",
    "Tonga": "to",
    "Trinidad and Tobago": "tt",
    "Tunisia": "tn",
    "Turkey": "tr",
    "Turkmenistan": "tm",
    "Tuvalu": "tv",
    "Uganda": "ug",
    "Ukraine": "ua",
    "United Arab Emirates": "ae",
    "United Kingdom": "gb",
    "United States": "us",
    "Uruguay": "uy",

    # Additions from running over large files
    "Bosnia and Herzegovina": "ba",
    #"International"
    "China (Republic : 1949- )": "tw", # pretty sure this is tw not cn
    "Russia (Federation)": "ru",
    "Scotland": "gb",
    "England": "gb",
    "Korea (South)": "kr",
    "Georgia (Republic)": "ge",
    "Egypt": "eg",
}


class PubmedImporter(EntityImporter):
    """
    Importer for PubMed/MEDLINE XML metadata.

    If lookup_refs is true, will do identifer-based lookups for all references.

    TODO: MEDLINE doesn't include PMC/OA license; could include in importer?
    """

    def __init__(self, api, issn_map_file, lookup_refs=True, **kwargs):

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
        self.create_containers = kwargs.get('create_containers', True)
        self.read_issn_map_file(issn_map_file)

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
            extra_pubmed['pub_types'] = pub_types
        if medline.Article.PublicationTypeList.find(string="Retraction of Publication"):
            release_type = "retraction"
            retraction_of = medline.find("CommentsCorrections", RefType="RetractionOf")
            if retraction_of:
                if retraction_of.RefSource:
                    extra_pubmed['retraction_of_raw'] = retraction_of.RefSource.string
                if retraction_of.PMID:
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

        title = medline.Article.ArticleTitle.get_text() # always present
        if title:
            title = title.replace('\n', ' ')
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
            original_title = original_title.get_text() or None
            original_title = original_title.replace('\n', ' ')
            if original_title and original_title.endswith('.'):
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
                    warnings.warn("MISSING MARC LANG: {}".format(medline.Article.Language.string))

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
                container_extra['country'] = country_code
            elif country_name:
                container_extra['country_name'] = country_name
        if mji.find("ISSNLinking"):
            issnl = mji.ISSNLinking.string

        journal = medline.Article.Journal
        issnp = journal.find("ISSN", IssnType="Print")
        if issnp:
            container_extra['issnp'] = issnp.string
        if not issnl:
            issnl = self.issn2issnl(issnp)

        if issnl:
            container_id = self.lookup_issnl(issnl)

        pub_date = medline.Article.find('ArticleDate')
        if not pub_date:
            pub_date = journal.PubDate
        if not pub_date:
            pub_date = journal.JournalIssue.PubDate
        release_date = None
        release_year = None
        if pub_date.Year:
            release_year = int(pub_date.Year.string)
            if pub_date.find("Day") and pub_date.find("Month"):
                try:
                    release_date = datetime.date(
                        release_year,
                        MONTH_ABBR_MAP[pub_date.Month.string],
                        int(pub_date.Day.string))
                    release_date = release_date.isoformat()
                except ValueError as ve:
                    print("bad date, skipping: {}".format(ve), file=sys.stderr)
                    release_date = None
        elif pub_date.MedlineDate:
            medline_date = pub_date.MedlineDate.string.strip()
            if len(medline_date) >= 4 and medline_date[:4].isdigit():
                release_year = int(medline_date[:4])
                if release_year < 1300 or release_year > 2040:
                    print("bad medline year, skipping: {}".format(release_year), file=sys.stderr)
                    release_year = None
            else:
                print("unparsable medline date, skipping: {}".format(medline_date), file=sys.stderr)

        if journal.find("Title"):
            container_name = journal.Title.get_text()

        if (container_id is None and self.create_containers and (issnl is not None)
                and container_name):
            # name, type, publisher, issnl
            # extra: issnp, issne, original_name, languages, country
            ce = fatcat_openapi_client.ContainerEntity(
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
                if abstract.find('math'):
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
            lang = "en"
            if other.get('Language'):
                lang = LANG_MAP_MARC.get(other['Language'])
            abst = fatcat_openapi_client.ReleaseAbstract(
                content=other.AbstractText.get_text().strip(),
                mimetype="text/plain",
                lang=lang,
            )
            if abst.content:
                abstracts.append(abst)
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
                    given_name = author.ForeName.get_text().replace('\n', ' ')
                if author.LastName:
                    surname = author.LastName.get_text().replace('\n', ' ')
                if given_name and surname:
                    raw_name = "{} {}".format(given_name, surname)
                elif surname:
                    raw_name = surname
                if not raw_name and author.CollectiveName and author.CollectiveName.get_text():
                    raw_name = author.CollectiveName.get_text().replace('\n', ' ')
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
                    raw_affiliation = affiliations[0].get_text().replace('\n', ' ')
                    if len(affiliations) > 1:
                        contrib_extra['more_affiliations'] = [ra.get_text().replace('\n', ' ') for ra in affiliations[1:]]
                if author.find("EqualContrib"):
                    # TODO: schema for this?
                    contrib_extra['equal'] = True
                contribs.append(fatcat_openapi_client.ReleaseContrib(
                    raw_name=raw_name,
                    given_name=given_name,
                    surname=surname,
                    role="author",
                    raw_affiliation=raw_affiliation,
                    creator_id=creator_id,
                    extra=contrib_extra,
                ))

            if medline.AuthorList['CompleteYN'] == 'N':
                contribs.append(fatcat_openapi_client.ReleaseContrib(raw_name="et al."))

        for i, contrib in enumerate(contribs):
            if contrib.raw_name != "et al.":
                contrib.index = i
        if not contribs:
            contribs = None

        ### References
        refs = []
        if pubmed.ReferenceList:
            # note that Reference always exists within a ReferenceList, but
            # that there may be multiple ReferenceList (eg, sometimes one per
            # Reference)
            for ref in pubmed.find_all('Reference'):
                ref_extra = dict()
                ref_doi = ref.find("ArticleId", IdType="doi")
                if ref_doi:
                    ref_doi = clean_doi(ref_doi.string)
                ref_pmid = ref.find("ArticleId", IdType="pubmed")
                if ref_pmid:
                    ref_pmid = clean_pmid(ref_pmid.string)
                ref_release_id = None
                if ref_doi:
                    ref_extra['doi'] = ref_doi
                    if self.lookup_refs:
                        ref_release_id = self.lookup_doi(ref_doi)
                if ref_pmid:
                    ref_extra['pmid'] = ref_pmid
                    if self.lookup_refs:
                        ref_release_id = self.lookup_pmid(ref_pmid)
                ref_raw = ref.Citation
                if ref_raw:
                    ref_extra['unstructured'] = ref_raw.get_text()
                if not ref_extra:
                    ref_extra = None
                refs.append(fatcat_openapi_client.ReleaseRef(
                    target_release_id=ref_release_id,
                    extra=ref_extra,
                ))
        if not refs:
            refs = None

        # extra:
        #   translation_of
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

        re = fatcat_openapi_client.ReleaseEntity(
            work_id=None,
            title=title,
            original_title=clean(original_title),
            release_type=release_type,
            release_stage=release_stage,
            release_date=release_date,
            release_year=release_year,
            withdrawn_status=withdrawn_status,
            ext_ids=fatcat_openapi_client.ReleaseExtIds(
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
                    existing.ident, existing.ext_ids.pmid, re.ext_ids.pmid)
                warnings.warn(warn_str)
                self.counts['warn-pmid-doi-mismatch'] += 1
                # don't clobber DOI, but do group together
                re.ext_ids.doi = None
                re.work_id = existing.work_id

        if existing and not self.do_updates:
            self.counts['exists'] += 1
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

            existing.container_id = existing.container_id or re.container_id
            existing.refs = existing.refs or re.refs
            existing.abstracts = existing.abstracts or re.abstracts
            existing.extra['pubmed'] = re.extra['pubmed']

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
            if not existing.subtitle and existing.extra.get('subtitle'):
                subtitle = existing.extra.pop('subtitle')
                if type(subtitle) == list:
                    subtitle = subtitle[0]
                if subtitle:
                    existing.subtitle = subtitle
            if not existing.subtitle:
                existing.subtitle = re.subtitle

            try:
                self.api.update_release(self.get_editgroup_id(), existing.ident, existing)
                self.counts['update'] += 1
            except fatcat_openapi_client.rest.ApiException as err:
                # there is a code path where we try to update the same release
                # twice in a row; if that happens, just skip
                # NOTE: API behavior might change in the future?
                if "release_edit_editgroup_id_ident_id_key" in err.body:
                    self.counts['skip-update-conflict'] += 1
                    return False
                else:
                    raise err
            finally:
                return False

        return True

    def insert_batch(self, batch):
        self.api.create_release_auto_batch(fatcat_openapi_client.ReleaseAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
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
