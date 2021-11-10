"""
This file contains lookup tables and other static data structures used in
bibliographic metadata munging.
"""

from typing import Dict, Optional

# These are very close, but maybe not exactly 1-to-1 with 639-2? Some mix of
# 2/T and 2/B?
# PubMed/MEDLINE and JSTOR use these MARC codes
# https://www.loc.gov/marc/languages/language_name.html
LANG_MAP_MARC: Dict[str, Optional[str]] = {
    "afr": "af",
    "alb": "sq",
    "amh": "am",
    "ara": "ar",
    "arm": "hy",
    "aze": "az",
    "ben": "bn",
    "bos": "bs",
    "bul": "bg",
    "cat": "ca",
    "chi": "zh",
    "cze": "cs",
    "dan": "da",
    "dut": "nl",
    "eng": "en",
    "epo": "eo",
    "est": "et",
    "fin": "fi",
    "fre": "fr",
    "geo": "ka",
    "ger": "de",
    "gla": "gd",
    "gre": "el",
    "heb": "he",
    "hin": "hi",
    "hrv": "hr",
    "hun": "hu",
    "ice": "is",
    "ind": "id",
    "ita": "it",
    "jpn": "ja",
    "kin": "rw",
    "kor": "ko",
    "lat": "la",
    "lav": "lv",
    "lit": "lt",
    "mac": "mk",
    "mal": "ml",
    "mao": "mi",
    "may": "ms",
    "nor": "no",
    "per": "fa",
    "per": "fa",
    "pol": "pl",
    "por": "pt",
    "pus": "ps",
    "rum": "ro",
    "rus": "ru",
    "san": "sa",
    "slo": "sk",
    "slv": "sl",
    "spa": "es",
    "srp": "sr",
    "swe": "sv",
    "tha": "th",
    "tur": "tr",
    "ukr": "uk",
    "urd": "ur",
    "vie": "vi",
    "wel": "cy",
    # additions
    "gle": "ga",  # "Irish" (Gaelic)
    "jav": "jv",  # Javanese
    "welsh": "cy",  # Welsh
    "oci": "oc",  # Occitan
    # Don't have ISO 639-1 codes
    "grc": "el",  # Ancient Greek; map to modern greek
    "map": None,  # Austronesian (collection)
    "syr": None,  # Syriac, Modern
    "gem": None,  # Old Saxon
    "non": None,  # Old Norse
    "emg": None,  # Eastern Meohang
    "neg": None,  # Negidal
    "mul": None,  # Multiple languages
    "und": None,  # Undetermined
}

# these are mappings from web domains to URL 'rel' for things like file entity
# URL notation
DOMAIN_REL_MAP: Dict[str, str] = {
    "archive.org": "archive",
    # LOCKSS, Portico, DuraSpace, etc would also be "archive"
    "arxiv.org": "repository",
    "babel.hathitrust.org": "repository",
    "cds.cern.ch": "repository",
    "deepblue.lib.umich.edu": "repository",
    "europepmc.org": "repository",
    "hal.inria.fr": "repository",
    "scielo.isciii.es": "repository",
    "www.dtic.mil": "repository",
    "www.jstage.jst.go.jp": "repository",
    "www.jstor.org": "repository",
    "www.ncbi.nlm.nih.gov": "repository",
    "ftp.ncbi.nlm.nih.gov": "repository",
    "www.scielo.br": "repository",
    "www.scielo.cl": "repository",
    "www.scielo.org.mx": "repository",
    "zenodo.org": "repository",
    "www.biorxiv.org": "repository",
    "www.medrxiv.org": "repository",
    "citeseerx.ist.psu.edu": "aggregator",
    "publisher-connector.core.ac.uk": "aggregator",
    "core.ac.uk": "aggregator",
    "static.aminer.org": "aggregator",
    "aminer.org": "aggregator",
    "pdfs.semanticscholar.org": "aggregator",
    "semanticscholar.org": "aggregator",
    "www.semanticscholar.org": "aggregator",
    "academic.oup.com": "publisher",
    "cdn.elifesciences.org": "publisher",
    "cell.com": "publisher",
    "dl.acm.org": "publisher",
    "downloads.hindawi.com": "publisher",
    "elifesciences.org": "publisher",
    "iopscience.iop.org": "publisher",
    "journals.plos.org": "publisher",
    "link.springer.com": "publisher",
    "onlinelibrary.wiley.com": "publisher",
    "works.bepress.com": "publisher",
    "www.biomedcentral.com": "publisher",
    "www.cell.com": "publisher",
    "www.nature.com": "publisher",
    "www.pnas.org": "publisher",
    "www.tandfonline.com": "publisher",
    "www.frontiersin.org": "publisher",
    "www.degruyter.com": "publisher",
    "www.mdpi.com": "publisher",
    "www.ahajournals.org": "publisher",
    "ehp.niehs.nih.gov": "publisher",
    "journals.tsu.ru": "publisher",
    "www.cogentoa.com": "publisher",
    "www.researchgate.net": "academicsocial",
    "academia.edu": "academicsocial",
    "wayback.archive-it.org": "webarchive",
    "web.archive.org": "webarchive",
    "archive.is": "webarchive",
}

# from: https://www.ncbi.nlm.nih.gov/books/NBK3827/table/pubmedhelp.T.publication_types/?report=objectonly
PUBMED_RELEASE_TYPE_MAP = {
    # Adaptive Clinical Trial
    "Address": "speech",
    "Autobiography": "book",
    # Bibliography
    "Biography": "book",
    # Case Reports
    "Classical Article": "article-journal",
    # Clinical Conference
    # Clinical Study
    # Clinical Trial
    # Clinical Trial, Phase I
    # Clinical Trial, Phase II
    # Clinical Trial, Phase III
    # Clinical Trial, Phase IV
    # Clinical Trial Protocol
    # Clinical Trial, Veterinary
    # Collected Works
    # Comparative Study
    # Congress
    # Consensus Development Conference
    # Consensus Development Conference, NIH
    # Controlled Clinical Trial
    "Dataset": "dataset",
    # Dictionary
    # Directory
    # Duplicate Publication
    "Editorial": "editorial",
    # English Abstract   # doesn't indicate that this is abstract-only
    # Equivalence Trial
    # Evaluation Studies
    # Expression of Concern
    # Festschrift
    # Government Document
    # Guideline
    "Historical Article": "article-journal",
    # Interactive Tutorial
    "Interview": "interview",
    "Introductory Journal Article": "article-journal",
    "Journal Article": "article-journal",
    "Lecture": "speech",
    "Legal Case": "legal_case",
    "Legislation": "legislation",
    "Letter": "letter",
    # Meta-Analysis
    # Multicenter Study
    # News
    "Newspaper Article": "article-newspaper",
    # Observational Study
    # Observational Study, Veterinary
    # Overall
    # Patient Education Handout
    # Periodical Index
    # Personal Narrative
    # Portrait
    # Practice Guideline
    # Pragmatic Clinical Trial
    # Publication Components
    # Publication Formats
    # Publication Type Category
    # Randomized Controlled Trial
    # Research Support, American Recovery and Reinvestment Act
    # Research Support, N.I.H., Extramural
    # Research Support, N.I.H., Intramural
    # Research Support, Non-U.S. Gov't Research Support, U.S. Gov't, Non-P.H.S.
    # Research Support, U.S. Gov't, P.H.S.
    # Review     # in the "literature review" sense, not "product review"
    # Scientific Integrity Review
    # Study Characteristics
    # Support of Research
    # Systematic Review
    "Technical Report": "report",
    # Twin Study
    # Validation Studies
    # Video-Audio Media
    # Webcasts
}

MONTH_ABBR_MAP: Dict[str, int] = {
    "Jan": 1,
    "01": 1,
    "Feb": 2,
    "02": 2,
    "Mar": 3,
    "03": 3,
    "Apr": 4,
    "04": 4,
    "May": 5,
    "05": 5,
    "Jun": 6,
    "06": 6,
    "Jul": 7,
    "07": 7,
    "Aug": 8,
    "08": 8,
    "Sep": 9,
    "09": 9,
    "Oct": 10,
    "10": 10,
    "Nov": 11,
    "11": 11,
    "Dec": 12,
    "12": 12,
}

# From: https://www.ncbi.nlm.nih.gov/books/NBK7249/
COUNTRY_NAME_MAP: Dict[str, str] = {
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
    # "International"
    "China (Republic : 1949- )": "tw",  # pretty sure this is tw not cn
    "Russia (Federation)": "ru",
    "Scotland": "gb",
    "England": "gb",
    "Korea (South)": "kr",
    "Georgia (Republic)": "ge",
    "Egypt": "eg",
}

CONTAINER_TYPE_MAP: Dict[str, str] = {
    "article-journal": "journal",
    "paper-conference": "conference",
    "book": "book-series",
}

# These are based, informally, on sorting the most popular licenses found in
# Crossref metadata. There were over 500 unique strings and only a few most
# popular are here; many were variants of the CC URLs. Would be useful to
# normalize CC licenses better.
# The current norm is to only add license slugs that are at least partially OA.
LICENSE_SLUG_MAP: Dict[str, str] = {
    "//creativecommons.org/publicdomain/mark/1.0": "CC-0",
    "//creativecommons.org/publicdomain/mark/1.0/": "CC-0",
    "//creativecommons.org/publicdomain/mark/1.0/deed.de": "CC-0",
    "//creativecommons.org/publicdomain/mark/1.0/deed.de": "CC-0",
    "//creativecommons.org/publicdomain/zero/1.0/": "CC-0",
    "//creativecommons.org/publicdomain/zero/1.0/legalcode": "CC-0",
    "//creativecommons.org/publicdomain/mark/1.0/deed.de": "CC-0",
    "//creativecommons.org/share-your-work/public-domain/cc0/": "CC-0",
    "//creativecommons.org/licenses/by/2.0/": "CC-BY",
    "//creativecommons.org/licenses/by/3.0/": "CC-BY",
    "//creativecommons.org/licenses/by/4.0/": "CC-BY",
    "//creativecommons.org/licenses/by-sa/3.0/": "CC-BY-SA",
    "//creativecommons.org/licenses/by-sa/4.0/": "CC-BY-SA",
    "//creativecommons.org/licenses/by-nd/3.0/": "CC-BY-ND",
    "//creativecommons.org/licenses/by-nd/4.0/": "CC-BY-ND",
    "//creativecommons.org/licenses/by-nc/3.0/": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc/4.0/": "CC-BY-NC",
    "//creativecommons.org/licenses/by-nc-sa/3.0/": "CC-BY-NC-SA",
    "//creativecommons.org/licenses/by-nc-sa/4.0/": "CC-BY-NC-SA",
    "//creativecommons.org/licenses/by-nc-nd/3.0/": "CC-BY-NC-ND",
    "//creativecommons.org/licenses/by-nc-nd/4.0/": "CC-BY-NC-ND",
    "//creativecommons.org/share-your-work/public-domain/cc0/": "CC-0",
    "//spdx.org/licenses/CC0-1.0.json": "CC-0",
    "//spdx.org/licenses/CC-BY-1.0.json": "CC-BY",
    "//spdx.org/licenses/CC-BY-4.0.json": "CC-BY",
    "//spdx.org/licenses/CC-BY-NC-4.0.json": "CC-BY-NC",
    "//spdx.org/licenses/CC-BY-SA-3.0.json": "CC-BY-SA",
    "//spdx.org/licenses/CC-BY-SA-4.0.json": "CC-BY-SA",
    "//spdx.org/licenses/MIT.json": "MIT",
    "//spdx.org/licenses/OGL-Canada-2.0.json": "OGL-Canada",
    "//www.elsevier.com/open-access/userlicense/1.0/": "ELSEVIER-USER-1.0",
    "//www.elsevier.com/tdm/userlicense/1.0/": "ELSEVIER-USER-1.0",
    "//www.karger.com/Services/SiteLicenses": "KARGER",
    "//www.karger.com/Services/SiteLicenses/": "KARGER",
    "//archaeologydataservice.ac.uk/advice/termsofuseandaccess.xhtml/": "ADS-UK",
    "//archaeologydataservice.ac.uk/advice/termsofuseandaccess/": "ADS-UK",
    "//homepage.data-planet.com/terms-use/": "SAGE-DATA-PLANET",
    "//publikationen.bibliothek.kit.edu/kitopen-lizenz/": "KIT-OPEN",
    "//pubs.acs.org/page/policy/authorchoice_ccby_termsofuse.html": "CC-BY",
    "//pubs.acs.org/page/policy/authorchoice_ccby_termsofuse.html/": "CC-BY",
    "//pubs.acs.org/page/policy/authorchoice_termsofuse.html": "ACS-CHOICE",
    "//pubs.acs.org/page/policy/authorchoice_termsofuse.html/": "ACS-CHOICE",
    "//www.ametsoc.org/PUBSReuseLicenses": "AMETSOC",
    "//www.ametsoc.org/PUBSReuseLicenses/": "AMETSOC",
    "//www.apa.org/pubs/journals/resources/open-access.aspx": "APA",
    "//www.apa.org/pubs/journals/resources/open-access.aspx/": "APA",
    "//www.biologists.com/user-licence-1-1": "BIOLOGISTS-USER",
    "//www.biologists.com/user-licence-1-1/": "BIOLOGISTS-USER",
    "//www.biologists.com/user-licence-1-1/": "BIOLOGISTS-USER",
    "//www.gnu.org/licenses/gpl-3.0.en.html/": "GPLv3",
    "//www.gnu.org/licenses/old-licenses/gpl-2.0.en.html/": "GPLv2",
    # //onlinelibrary.wiley.com/termsAndConditions doesn't seem like a license
    # //www.springer.com/tdm doesn't seem like a license
    # //iopscience.iop.org/page/copyright is closed
    # //www.acm.org/publications/policies/copyright_policy#Background is closed
    # //rsc.li/journals-terms-of-use is closed for vor (am open)
    # //www.ieee.org/publications_standards/publications/rights/ieeecopyrightform.pdf is 404 (!)
    "//arxiv.org/licenses/nonexclusive-distrib/1.0/": "ARXIV-1.0",
    # skip these TDM licenses; they don't apply to content
    # "//www.springer.com/tdm/": "SPRINGER-TDM",
    # "//journals.sagepub.com/page/policies/text-and-data-mining-license/": "SAGE-TDM",
    # "//doi.wiley.com/10.1002/tdm_license_1.1/": "WILEY-TDM-1.1",
}

# Map various datacite type types to CSL-ish types. None means TODO or remove.
DATACITE_TYPE_MAP: Dict[str, Dict[str, Optional[str]]] = {
    "ris": {
        "THES": "thesis",
        "SOUND": "song",  # 99.9% maps to citeproc song, so use that (exception: report)
        "CHAP": "chapter",
        "FIGURE": "figure",
        "RPRT": "report",
        "JOUR": "article-journal",
        "MPCT": "motion_picture",
        "GEN": "article-journal",  # GEN consist of 99% article and report, post-weblog, misc - and one dataset
        "BOOK": "book",
        "DATA": "dataset",
        "COMP": "software",
    },
    "schemaOrg": {
        "Dataset": "dataset",
        "Book": "book",
        "ScholarlyArticle": "article-journal",
        "ImageObject": "graphic",
        "Collection": None,
        "MediaObject": None,
        "Event": None,
        "SoftwareSourceCode": "software",
        "Chapter": "chapter",
        "CreativeWork": None,  # Seems to be a catch-all resourceType, from PGRFA Material, Pamphlet, to music score.
        "PublicationIssue": "article",
        "AudioObject": None,
        "Thesis": "thesis",
    },
    "citeproc": {
        "article": "article",
        "article-journal": "article-journal",
        "article-magazine": "article-magazine",
        "article-newspaper": "article-newspaper",
        "bill": "bill",
        "book": "book",
        "broadcast": "broadcast",
        "chapter": "chapter",
        "dataset": "dataset",
        "entry-dictionary": "entry-dictionary",
        "entry-encyclopedia": "entry-encyclopedia",
        "entry": "entry",
        "figure": "figure",
        "graphic": "graphic",
        "interview": "interview",
        "legal_case": "legal_case",
        "legislation": "legislation",
        "manuscript": "manuscript",
        "map": "map",
        "motion_picture": "motion_picture",
        "musical_score": "musical_score",
        "pamphlet": "pamphlet",
        "paper-conference": "paper-conference",
        "patent": "patent",
        "personal_communication": "personal_communication",
        "post": "post",
        "post-weblog": "post-weblog",
        "report": "report",
        "review-book": "review-book",
        "review": "review",
        "song": "song",
        "speech": "speech",
        "thesis": "thesis",
        "treaty": "treaty",
        "webpage": "webpage",
    },  # https://docs.citationstyles.org/en/master/specification.html#appendix-iii-types
    "bibtex": {
        "phdthesis": "thesis",
        "inbook": "chapter",
        "misc": None,
        "article": "article-journal",
        "book": "book",
    },
    "resourceTypeGeneral": {
        "Image": "graphic",
        "Dataset": "dataset",
        "PhysicalObject": None,
        "Collection": None,
        "Text": None,  # "Greyliterature, labnotes, accompanyingmaterials"
        "Sound": None,
        "InteractiveResource": None,
        "Event": None,
        "Software": "software",
        "Other": None,
        "Workflow": None,
        "Audiovisual": None,
    },  # https://schema.datacite.org/meta/kernel-4.0/doc/DataCite-MetadataKernel_v4.0.pdf#page=32
}
