
"""
A bunch of helpers to parse and normalize strings: external identifiers,
free-form input, titles, etc.
"""

import re

DOI_REGEX = re.compile("^10.\d{3,6}/\S+$")

def clean_doi(raw):
    """
    Removes any:
    - padding whitespace
    - 'doi:' prefix
    - URL prefix

    Does not try to un-URL-encode

    Returns None if not a valid DOI
    """
    if not raw:
        return None
    raw = raw.strip()
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
    if not raw.startswith("10."):
        return None
    if not DOI_REGEX.fullmatch(raw):
        return None
    return raw

def test_clean_doi():
    assert clean_doi("10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("http://doi.org/10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("https://dx.doi.org/10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("doi:10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("doi:10.1234/ asdf ") == None

ARXIV_ID_REGEX = re.compile("^(\d{4}.\d{4,5}|[a-z\-]+(\.[A-Z]{2})?/\d{7})(v\d+)?$")

def clean_arxiv_id(raw):
    """
    Removes any:
    - 'arxiv:' prefix

    Works with versioned or un-versioned arxiv identifiers.
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

def test_clean_arxiv_id():
    assert clean_arxiv_id("0806.2878v1") == "0806.2878v1"
    assert clean_arxiv_id("0806.2878") == "0806.2878"
    assert clean_arxiv_id("1501.00001v1") == "1501.00001v1"
    assert clean_arxiv_id("1501.00001") == "1501.00001"
    assert clean_arxiv_id("hep-th/9901001v1") == "hep-th/9901001v1"
    assert clean_arxiv_id("hep-th/9901001") == "hep-th/9901001"
    assert clean_arxiv_id("math.CA/0611800v2") == "math.CA/0611800v2"
    assert clean_arxiv_id("math.CA/0611800") == "math.CA/0611800"
    assert clean_arxiv_id("0806.2878v1 ") == "0806.2878v1"

    assert clean_arxiv_id("https://arxiv.org/abs/0806.2878v1") == "0806.2878v1"
    assert clean_arxiv_id("arxiv:0806.2878v1") == "0806.2878v1"
    assert clean_arxiv_id("arXiv:0806.2878v1") == "0806.2878v1"

    assert clean_arxiv_id("hep-TH/9901001v1") == None
    assert clean_arxiv_id("hßp-th/9901001v1") == None
    assert clean_arxiv_id("math.CA/06l1800v2") == None
    assert clean_arxiv_id("mßth.ca/0611800v2") == None
    assert clean_arxiv_id("MATH.CA/0611800v2") == None
    assert clean_arxiv_id("0806.2878v23") == "0806.2878v23"  # ?
    assert clean_arxiv_id("0806.2878v") == None
    assert clean_arxiv_id("0806.2878") == "0806.2878"
    assert clean_arxiv_id("006.2878v1") == None
    assert clean_arxiv_id("0806.v1") == None
    assert clean_arxiv_id("08062878v1") == None

def clean_pmid(raw):
    if not raw:
        return None
    raw = raw.strip()
    if len(raw.split()) != 1:
        return None
    if raw.isdigit():
        return raw
    return None

def test_clean_pmid():
    assert clean_pmid("1234") == "1234"
    assert clean_pmid("1234 ") == "1234"
    assert clean_pmid("PMC123") == None
    assert clean_sha1("qfba3") == None
    assert clean_sha1("") == None

def clean_pmcid(raw):
    if not raw:
        return None
    raw = raw.strip()
    if len(raw.split()) != 1:
        return None
    if raw.startswith("PMC") and raw[3:] and raw[3:].isdigit():
        return raw
    return None

def clean_sha1(raw):
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

def test_clean_sha1():
    assert clean_sha1("0fba3fba0e1937aa0297de3836b768b5dfb23d7b") == "0fba3fba0e1937aa0297de3836b768b5dfb23d7b"
    assert clean_sha1("0fba3fba0e1937aa0297de3836b768b5dfb23d7b ") == "0fba3fba0e1937aa0297de3836b768b5dfb23d7b"
    assert clean_sha1("fba3fba0e1937aa0297de3836b768b5dfb23d7b") == None
    assert clean_sha1("qfba3fba0e1937aa0297de3836b768b5dfb23d7b") == None
    assert clean_sha1("0fba3fb a0e1937aa0297de3836b768b5dfb23d7b") == None

def clean_sha256(raw):
    raw = raw.strip().lower()
    if len(raw.split()) != 1:
        return None
    if len(raw) != 64:
        return None
    for c in raw:
        if c not in "0123456789abcdef":
            return None
    return raw

def test_clean_sha256():
    assert clean_sha256("6cc853f2ae75696b2e45f476c76b946b0fc2df7c52bb38287cb074aceb77bc7f") == "6cc853f2ae75696b2e45f476c76b946b0fc2df7c52bb38287cb074aceb77bc7f"
    assert clean_sha256("0fba3fba0e1937aa0297de3836b768b5dfb23d7b") == None

ISSN_REGEX = re.compile("^\d{4}-\d{3}[0-9X]$")

def clean_issn(raw):
    if not raw:
        return None
    raw = raw.strip().upper()
    if len(raw) != 9:
        return None
    if not ISSN_REGEX.fullmatch(raw):
        return None
    return raw

def test_clean_issn():
    assert clean_issn("1234-4567") == "1234-4567"
    assert clean_issn("1234-456X") == "1234-456X"
    assert clean_issn("134-4567") == None
    assert clean_issn("123X-4567") == None

ISBN13_REGEX = re.compile("^97(?:8|9)-\d{1,5}-\d{1,7}-\d{1,6}-\d$")

def clean_isbn13(raw):
    if not raw:
        return None
    raw = raw.strip()
    if not ISBN13_REGEX.fullmatch(raw):
        return None
    return raw

def test_clean_isbn13():
    assert clean_isbn13("978-1-56619-909-4") == "978-1-56619-909-4"
    assert clean_isbn13("978-1-4028-9462-6") == "978-1-4028-9462-6"
    assert clean_isbn13("978-1-56619-909-4 ") == "978-1-56619-909-4"
    assert clean_isbn13("9781566199094") == None

ORCID_REGEX = re.compile("^\d{4}-\d{4}-\d{4}-\d{3}[\dX]$")

def clean_orcid(raw):
    if not raw:
        return None
    raw = raw.strip()
    if not ORCID_REGEX.fullmatch(raw):
        return None
    return raw

def test_clean_orcid():
    assert clean_orcid("0123-4567-3456-6789") == "0123-4567-3456-6789"
    assert clean_orcid("0123-4567-3456-678X") == "0123-4567-3456-678X"
    assert clean_orcid("0123-4567-3456-6789 ") == "0123-4567-3456-6789"
    assert clean_orcid("01234567-3456-6780") == None
    assert clean_orcid("0x23-4567-3456-6780") == None

