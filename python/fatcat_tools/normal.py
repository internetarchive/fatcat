"""
A bunch of helpers to parse and normalize strings: external identifiers,
free-form input, titles, etc.
"""

import base64
import re
import unicodedata
from typing import Optional, Union

import ftfy
import langdetect
import pycountry

from .biblio_lookup_tables import LICENSE_SLUG_MAP

DOI_REGEX = re.compile(r"^10.\d{3,6}/\S+$")


def clean_doi(raw: Optional[str]) -> Optional[str]:
    """
    Removes any:
    - padding whitespace
    - 'doi:' prefix
    - URL prefix

    Lower-cases the DOI.

    Does not try to un-URL-encode

    Returns None if not a valid DOI
    """
    if not raw:
        return None
    raw = raw.strip().lower()
    if "\u2013" in raw:
        # Do not attempt to normalize "en dash" and since FC does not allow
        # unicode in DOI, treat this as invalid.
        return None
    if len(raw.split()) != 1:
        return None
    if raw.startswith("doi:"):
        raw = raw[4:]
    if raw.startswith("http://"):
        raw = raw[7:]
    if raw.startswith("https://"):
        raw = raw[8:]
    if raw.startswith("doi.org/"):
        raw = raw[8:]
    if raw.startswith("dx.doi.org/"):
        raw = raw[11:]
    if raw[7:9] == "//" and "10.1037//" in raw:
        raw = raw[:8] + raw[9:]

    # fatcatd uses same REGEX, but Rust regex rejects these characters, while
    # python doesn't. DOIs are syntaxtually valid, but very likely to be typos;
    # for now filter them out.
    for c in ("¬",):
        if c in raw:
            return None

    if not raw.startswith("10."):
        return None
    if not DOI_REGEX.fullmatch(raw):
        return None
    # will likely want to expand DOI_REGEX to exclude non-ASCII characters, but
    # for now block specific characters so we can get PubMed importer running
    # again.
    # known characters to skip: ä á \u200e \u2043 \u2012
    if not raw.isascii():
        return None
    return raw


def test_clean_doi() -> None:
    assert clean_doi("10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("10.1037//0002-9432.72.1.50") == "10.1037/0002-9432.72.1.50"
    assert clean_doi("10.1037/0002-9432.72.1.50") == "10.1037/0002-9432.72.1.50"
    assert clean_doi("10.1026//1616-1041.3.2.86") == "10.1026//1616-1041.3.2.86"
    assert clean_doi("10.23750/abm.v88i2 -s.6506") is None
    assert clean_doi("10.17167/mksz.2017.2.129–155") is None
    assert clean_doi("http://doi.org/10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("https://dx.doi.org/10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("doi:10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("doi:10.1234/ asdf ") is None
    assert clean_doi("10.4149/gpb¬_2017042") is None  # "logical negation" character
    assert (
        clean_doi("10.6002/ect.2020.häyry") is None
    )  # this example via pubmed (pmid:32519616)
    assert clean_doi("10.30466/vrf.2019.98547.2350\u200e") is None
    assert clean_doi("10.12016/j.issn.2096⁃1456.2017.06.014") is None
    assert clean_doi("10.4025/diálogos.v17i2.36030") is None
    assert clean_doi("10.19027/jai.10.106‒115") is None
    assert clean_doi("10.15673/атбп2312-3125.17/2014.26332") is None
    assert clean_doi("10.7326/M20-6817") == "10.7326/m20-6817"


ARXIV_ID_REGEX = re.compile(r"^(\d{4}.\d{4,5}|[a-z\-]+(\.[A-Z]{2})?/\d{7})(v\d+)?$")


def clean_arxiv_id(raw: Optional[str]) -> Optional[str]:
    """
    Removes any:
    - 'arxiv:' prefix

    Works with versioned or un-versioned arxiv identifiers.

    TODO: version of this function that only works with versioned identifiers?
    That is the behavior of fatcat API
    """
    if not raw:
        return None
    raw = raw.strip()
    if raw.lower().startswith("arxiv:"):
        raw = raw[6:]
    if raw.lower().startswith("https://arxiv.org/abs/"):
        raw = raw[22:]
    if not ARXIV_ID_REGEX.fullmatch(raw):
        return None
    return raw


def test_clean_arxiv_id() -> None:
    assert clean_arxiv_id("0806.2878v1") == "0806.2878v1"
    assert clean_arxiv_id("0806.2878") == "0806.2878"
    assert clean_arxiv_id("1501.00001v1") == "1501.00001v1"
    assert clean_arxiv_id("1501.00001") == "1501.00001"
    assert clean_arxiv_id("hep-th/9901001v1") == "hep-th/9901001v1"
    assert clean_arxiv_id("hep-th/9901001") == "hep-th/9901001"
    assert clean_arxiv_id("math.CA/0611800v2") == "math.CA/0611800v2"
    assert clean_arxiv_id("math.CA/0611800") == "math.CA/0611800"
    assert clean_arxiv_id("0806.2878v1 ") == "0806.2878v1"
    assert clean_arxiv_id("cs/0207047") == "cs/0207047"

    assert clean_arxiv_id("https://arxiv.org/abs/0806.2878v1") == "0806.2878v1"
    assert clean_arxiv_id("arxiv:0806.2878v1") == "0806.2878v1"
    assert clean_arxiv_id("arXiv:0806.2878v1") == "0806.2878v1"

    assert clean_arxiv_id("hep-TH/9901001v1") is None
    assert clean_arxiv_id("hßp-th/9901001v1") is None
    assert clean_arxiv_id("math.CA/06l1800v2") is None
    assert clean_arxiv_id("mßth.ca/0611800v2") is None
    assert clean_arxiv_id("MATH.CA/0611800v2") is None
    assert clean_arxiv_id("0806.2878v23") == "0806.2878v23"  # ?
    assert clean_arxiv_id("0806.2878v") is None
    assert clean_arxiv_id("0806.2878") == "0806.2878"
    assert clean_arxiv_id("006.2878v1") is None
    assert clean_arxiv_id("0806.v1") is None
    assert clean_arxiv_id("08062878v1") is None


def clean_wikidata_qid(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    raw = raw.strip()
    if len(raw.split()) != 1 or len(raw) < 2:
        return None
    if raw[0] == "Q" and raw[1] != "0" and raw[1:].isdigit():
        return raw
    return None


def test_clean_wikidata_qid() -> None:
    assert clean_wikidata_qid("Q1234") == "Q1234"
    assert clean_wikidata_qid("Q1") == "Q1"
    assert clean_wikidata_qid(" Q1234 ") == "Q1234"
    assert clean_wikidata_qid(" Q1 234 ") is None
    assert clean_wikidata_qid("q1234") is None
    assert clean_wikidata_qid("1234 ") is None
    assert clean_wikidata_qid("Q0123") is None
    assert clean_wikidata_qid("PMC123") is None
    assert clean_wikidata_qid("qfba3") is None
    assert clean_wikidata_qid("") is None


def clean_pmid(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    raw = raw.strip()
    if len(raw.split()) != 1:
        return None
    if raw.isdigit():
        return raw
    return None


def test_clean_pmid() -> None:
    assert clean_pmid("1234") == "1234"
    assert clean_pmid("1234 ") == "1234"
    assert clean_pmid("PMC123") is None
    assert clean_pmid("qfba3") is None
    assert clean_pmid("") is None


def clean_pmcid(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    raw = raw.strip()
    if len(raw.split()) != 1:
        return None
    if raw.startswith("PMC") and raw[3:] and raw[3:].isdigit():
        return raw
    return None


def clean_sha1(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    raw = raw.strip().lower()
    if len(raw.split()) != 1:
        return None
    if len(raw) != 40:
        return None
    for c in raw:
        if c not in "0123456789abcdef":
            return None
    return raw


def test_clean_sha1() -> None:
    assert (
        clean_sha1("0fba3fba0e1937aa0297de3836b768b5dfb23d7b")
        == "0fba3fba0e1937aa0297de3836b768b5dfb23d7b"
    )
    assert (
        clean_sha1("0fba3fba0e1937aa0297de3836b768b5dfb23d7b ")
        == "0fba3fba0e1937aa0297de3836b768b5dfb23d7b"
    )
    assert clean_sha1("fba3fba0e1937aa0297de3836b768b5dfb23d7b") is None
    assert clean_sha1("qfba3fba0e1937aa0297de3836b768b5dfb23d7b") is None
    assert clean_sha1("0fba3fb a0e1937aa0297de3836b768b5dfb23d7b") is None


def clean_sha256(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    raw = raw.strip().lower()
    if len(raw.split()) != 1:
        return None
    if len(raw) != 64:
        return None
    for c in raw:
        if c not in "0123456789abcdef":
            return None
    return raw


def test_clean_sha256() -> None:
    assert (
        clean_sha256("6cc853f2ae75696b2e45f476c76b946b0fc2df7c52bb38287cb074aceb77bc7f")
        == "6cc853f2ae75696b2e45f476c76b946b0fc2df7c52bb38287cb074aceb77bc7f"
    )
    assert clean_sha256("0fba3fba0e1937aa0297de3836b768b5dfb23d7b") is None


ISSN_REGEX = re.compile(r"^\d{4}-\d{3}[0-9X]$")


def clean_issn(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    raw = raw.strip().upper()
    if len(raw) != 9:
        return None
    if not ISSN_REGEX.fullmatch(raw):
        return None
    return raw


def test_clean_issn() -> None:
    assert clean_issn("1234-4567") == "1234-4567"
    assert clean_issn("1234-456X") == "1234-456X"
    assert clean_issn("134-4567") is None
    assert clean_issn("123X-4567") is None


ISBN13_REGEX = re.compile(r"^97(?:8|9)-\d{1,5}-\d{1,7}-\d{1,6}-\d$")


def clean_isbn13(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    raw = raw.strip()
    if not ISBN13_REGEX.fullmatch(raw):
        return None
    return raw


def test_clean_isbn13() -> None:
    assert clean_isbn13("978-1-56619-909-4") == "978-1-56619-909-4"
    assert clean_isbn13("978-1-4028-9462-6") == "978-1-4028-9462-6"
    assert clean_isbn13("978-1-56619-909-4 ") == "978-1-56619-909-4"
    assert clean_isbn13("9781566199094") is None


ORCID_REGEX = re.compile(r"^\d{4}-\d{4}-\d{4}-\d{3}[\dX]$")


def clean_orcid(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    raw = raw.strip()
    if not ORCID_REGEX.fullmatch(raw):
        return None
    return raw


def test_clean_orcid() -> None:
    assert clean_orcid("0123-4567-3456-6789") == "0123-4567-3456-6789"
    assert clean_orcid("0123-4567-3456-678X") == "0123-4567-3456-678X"
    assert clean_orcid("0123-4567-3456-6789 ") == "0123-4567-3456-6789"
    assert clean_orcid("01234567-3456-6780") is None
    assert clean_orcid("0x23-4567-3456-6780") is None


HDL_REGEX = re.compile(r"^\d+(\.\d+)*/\S+$")


def clean_hdl(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    raw = raw.strip().lower()
    if raw.startswith("hdl:"):
        raw = raw[4:]
    if raw.startswith("http://"):
        raw = raw[7:]
    if raw.startswith("https://"):
        raw = raw[8:]
    if raw.startswith("hdl.handle.net/"):
        raw = raw[15:]
    if not HDL_REGEX.fullmatch(raw):
        return None
    if raw.startswith("10."):
        return None
    return raw


def test_clean_hdl() -> None:
    assert clean_hdl("20.500.23456/ABC/DUMMY") == "20.500.23456/abc/dummy"
    assert clean_hdl("hdl:20.500.23456/ABC/DUMMY") == "20.500.23456/abc/dummy"
    assert (
        clean_hdl("https://hdl.handle.net/20.500.23456/ABC/DUMMY") == "20.500.23456/abc/dummy"
    )
    assert clean_hdl("http://hdl.handle.net/20.500.23456/ABC/DUMMY") == "20.500.23456/abc/dummy"
    assert clean_hdl("21.1234/aksjdfh") == "21.1234/aksjdfh"
    assert clean_hdl("2381/12775") == "2381/12775"
    assert clean_hdl("10.1234/aksjdfh") is None
    assert clean_hdl("20.1234") is None
    assert clean_hdl("20.1234/") is None
    assert clean_hdl("20./asdf") is None


def clean_str(thing: Any, force_xml: bool = False) -> Optional[str]:
    """
    This function is appropriate to be called on any random, non-markup string,
    such as author names, titles, etc.

    It will try to clean up common unicode mangles, HTML characters, etc.

    This will detect XML/HTML and "do the right thing" (aka, not remove
    entities like '&amp' if there are tags in the string), unless you pass the
    'force_xml' parameter, which might be appropriate for, eg, names and
    titles, which generally should be projected down to plain text.

    Also strips extra whitespace.
    """
    if thing is None:
        return None
    thing = f"{thing}"
    unescape_html: Union[str, bool] = "auto"
    if force_xml:
        unescape_html = True
    fixed = ftfy.fix_text(thing, unescape_html=unescape_html).strip()
    if not fixed or len(fixed) <= 1:
        # wasn't zero-length before, but is now; return None
        return None
    return fixed


def test_clean_str() -> None:

    assert clean_str(None) is None
    assert clean_str("") is None
    assert clean_str("1") is None
    assert clean_str("123") == "123"
    assert clean_str("a&amp;b") == "a&b"
    assert clean_str("<b>a&amp;b</b>") == "<b>a&amp;b</b>"
    assert clean_str("<b>a&amp;b</b>", force_xml=True) == "<b>a&b</b>"
    assert clean_str(1) == "1"
    assert clean_str(1.1) == "1.1"


def b32_hex(s: str) -> str:
    s = s.strip().split()[0].lower()
    if s.startswith("sha1:"):
        s = s[5:]
    if len(s) != 32:
        return s
    return base64.b16encode(base64.b32decode(s.upper())).lower().decode("utf-8")


def is_cjk(s: Optional[str]) -> bool:
    if not s:
        return False
    for c in s:
        if c.isalpha():
            lang_prefix = unicodedata.name(c).split()[0]
            return lang_prefix in ("CJK", "HIRAGANA", "KATAKANA", "HANGUL")
    return False


def test_is_cjk() -> None:
    assert is_cjk(None) is False
    assert is_cjk("") is False
    assert is_cjk("blah") is False
    assert is_cjk("岡, 鹿, 梨, 阜, 埼") is True
    assert is_cjk("[岡, 鹿, 梨, 阜, 埼]") is True
    assert is_cjk("菊") is True
    assert is_cjk("岡, 鹿, 梨, 阜, 埼 with eng after") is True
    assert is_cjk("水道") is True
    assert is_cjk("オウ, イク") is True  # kanji
    assert is_cjk("ひヒ") is True
    assert is_cjk("き゚ゅ") is True
    assert is_cjk("ㄴ, ㄹ, ㅁ, ㅂ, ㅅ") is True


MONTH_MAP = {
    "jan": 1,
    "january": 1,
    "feb": 2,
    "febuary": 2,
    "mar": 3,
    "march": 3,
    "apr": 4,
    "april": 4,
    "may": 5,
    "may": 5,
    "jun": 6,
    "june": 6,
    "jul": 7,
    "july": 7,
    "aug": 8,
    "august": 8,
    "sep": 9,
    "september": 9,
    "oct": 10,
    "october": 10,
    "nov": 11,
    "nov": 11,
    "dec": 12,
    "december": 12,
}


def parse_month(raw: Optional[str]) -> Optional[int]:
    """
    Parses a string into a month number (1 to 12)
    """
    if not raw:
        return None
    raw = raw.strip().lower()
    if raw.isdigit():
        raw_int = int(raw)
        if raw_int >= 1 and raw_int <= 12:
            return raw_int
        else:
            return None
    if raw in MONTH_MAP:
        return MONTH_MAP[raw]
    return None


def test_parse_month() -> None:

    assert parse_month(None) is None
    assert parse_month("") is None
    assert parse_month("0") is None
    assert parse_month("10") == 10
    assert parse_month("jan") == 1
    assert parse_month("September") == 9


def detect_text_lang(raw: Optional[str]) -> Optional[str]:
    """
    Tries to determine language of, eg, an abstract.

    Returns an ISO 631 2-char language code, or None.
    """
    if not raw:
        return None
    try:
        lang = langdetect.detect(raw)
        lang = lang.split("-")[0]
        assert len(lang) == 2
        return lang
    except (langdetect.lang_detect_exception.LangDetectException, TypeError):
        return None
    return None


def test_detect_text_lang() -> None:
    assert detect_text_lang("") is None
    EN_SAMPLE = "this is a string of English text for testing"
    assert detect_text_lang(EN_SAMPLE) == "en"
    JA_SAMPLE = "モーラの種類は、以下に示すように111程度存在する。ただし、研究者により数え方が少しずつ異なる。"
    assert detect_text_lang(JA_SAMPLE) == "ja"
    ZH_SAMPLE = "随着分布式清洁能源的普及,通信技术在协调各个分布式电源的控制中显得尤为重要。在电力信息传输的过程中,不同的网络状态下表现出不同的通信特性,严重的甚至会发生信息错乱丢包等行为,这对电网的实时控制产生严重影响。为研究信息系统对电力物理系统的实时影响,搭建了电力信息物理融合仿真平台,运用RT-LAB与OPNET两款实时仿真器,通过TCP/IP进行数据交互,对微电网电压、频率的集中式恢复与分布式恢复问题展开研究。仿真结果表明,该平台能有效地反映通信网络对电网控制的影响,提供了一种可靠的未来电力信息物理融合系统研究技术。随着分布式清洁能源的普及,通信技术在协调各个分布式电源的控制中显得尤为重要。在电力信息传输的过程中,不同的网络状态下表现出不同的通信特性,严重的甚至会发生信息错乱丢包等行为,这对电网的实时控制产生严重影响。为研究信息系统对电力物理系统的实时影响,搭建了电力信息物理融合仿真平台,运用RT-LAB与OPNET两款实时仿真器,通过TCP/IP进行数据交互,对微电网电压、频率的集中式恢复与分布式恢复问题展开研究。仿真结果表明,该平台能有效地反映通信网络对电网控制的影响,提供了一种可靠的未来电力信息物理融合系统研究技术。"
    # XXX: why does this detect as `ko` sometimes?
    assert detect_text_lang(ZH_SAMPLE) in ("zh", "ko")


def parse_lang_name(raw: Optional[str]) -> Optional[str]:
    """
    Parses a language name and returns a 2-char ISO 631 language code.
    """
    if not raw:
        return None
    try:
        lang = pycountry.languages.lookup(raw)
        if lang.alpha_3 in ("mul", "mis"):
            return None
        return lang.alpha_2.lower()
    except LookupError:
        # print(f"  unknown language: '{raw}', file=sys.stderr)
        return None
    except AttributeError:
        # print(f"  partial language metadata: '{lang}', file=sys.stderr)
        return None
    return None


def test_parse_lang_name() -> None:

    assert parse_lang_name(None) is None
    assert parse_lang_name("") is None
    assert parse_lang_name("asdf ") is None
    assert parse_lang_name("english") == "en"
    assert parse_lang_name("ENGLISH") == "en"
    assert parse_lang_name("asdf blah") is None
    assert parse_lang_name("en") == "en"
    assert parse_lang_name("EN") == "en"
    assert parse_lang_name("ENG") == "en"
    assert parse_lang_name("English") == "en"
    assert parse_lang_name("Portuguese") == "pt"


def parse_country_name(s: Optional[str]) -> Optional[str]:
    """
    Parses a country name into a ISO country code (2-char).

    This version copied from the chocula repository.
    """
    if not s or s in ("Unknown"):
        return None

    s = s.strip()
    if s.lower() in ("usa", "new york (state)", "washington (state)"):
        return "us"
    if s.lower() in ("russia (federation)", "russia"):
        return "ru"
    if s == "Québec (Province)":
        s = "Canada"
    if s == "China (Republic : 1949- )":
        return "tw"
    if s == "Brunei":
        return "bn"
    if s.startswith("Congo "):
        s = "Congo"
    if s.lower() == "iran":
        return "ir"
    if s.lower() == "bermuda islands":
        return "bm"
    if s.lower() == "burma":
        s = "myanmar"
    if s.lower() in ("korea (south)", "south korea"):
        return "kr"
    if s.lower() in ("england", "scotland", "wales"):
        return "uk"
    s = s.replace(" (Republic)", "").replace(" (Federation)", "")

    try:
        country = pycountry.countries.lookup(s)
    except LookupError:
        country = None

    if country:
        return country.alpha_2.lower()
    try:
        sub = pycountry.subdivisions.lookup(s)
    except LookupError:
        sub = None

    s = s.replace(" (State)", "").replace(" (Province)", "")
    if sub:
        return sub.country_code.lower()

    else:
        # print(f"unknown country: {s}", file=sys.stderr)
        return None


def test_parse_country_name() -> None:
    assert parse_country_name("") is None
    assert parse_country_name("asdf blah") is None
    assert parse_country_name("us") == "us"
    assert parse_country_name("USA") == "us"
    assert parse_country_name("United States of America") == "us"
    assert parse_country_name("united States") == "us"
    assert parse_country_name("Massachusetts") == "us"
    assert parse_country_name("Russia") == "ru"
    assert parse_country_name("Japan") == "jp"


def lookup_license_slug(raw: Optional[str]) -> Optional[str]:
    if not raw:
        return None
    # normalize to lower-case and not ending with a slash
    raw = raw.strip().lower()
    if raw.endswith("/"):
        raw = raw[:-1]
    # remove http/https prefix
    raw = raw.replace("http://", "//").replace("https://", "//")
    # special-case normalization of CC licenses
    if "creativecommons.org" in raw:
        raw = raw.replace("/legalcode", "").replace("/uk", "")
    return LICENSE_SLUG_MAP.get(raw)


def test_lookup_license_slug() -> None:

    assert lookup_license_slug("https://creativecommons.org/licenses/by-nc/3.0/") == "CC-BY-NC"
    assert (
        lookup_license_slug("http://creativecommons.org/licenses/by/2.0/uk/legalcode")
        == "CC-BY"
    )
    assert (
        lookup_license_slug("https://creativecommons.org/publicdomain/zero/1.0/legalcode")
        == "CC-0"
    )
    assert lookup_license_slug("http://creativecommons.org/licenses/by/4.0") == "CC-BY"
    assert (
        lookup_license_slug("https://creativecommons.org/licenses/by-nc-sa/4.0/")
        == "CC-BY-NC-SA"
    )
    assert lookup_license_slug("https://www.ametsoc.org/PUBSReuseLicenses") == "AMETSOC"
    assert lookup_license_slug("https://www.amec.org/PUBSReuseLicenses") is None
    assert lookup_license_slug("") is None
    assert lookup_license_slug(None) is None
