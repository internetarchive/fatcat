
"""
A bunch of helpers to parse and normalize strings: external identifiers,
free-form input, titles, etc.
"""

import re


def clean_doi(raw):
    """
    Removes any:
    - padding whitespace
    - 'doi:' prefix
    - URL prefix

    Does not try to un-URL-encode

    Returns None if not a valid DOI
    """
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
    # TODO: actual regex
    return raw

def test_clean_doi():
    assert clean_doi("10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("http://doi.org/10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("https://dx.doi.org/10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("doi:10.1234/asdf ") == "10.1234/asdf"
    assert clean_doi("doi:10.1234/ asdf ") == None

def clean_arxiv_id(raw):
    """
    Removes any:
    - 'arxiv:' prefix

    Works with versioned or un-versioned arxiv identifiers.
    """
    pass

def test_clean_arxiv_id():
    pass

def clean_pmcid(raw):
    raw = raw.strip()
    if len(raw.split()) != 1:
        return None
    if raw.startswith("PMC") and raw[3:] and raw[3:].isdigit():
        return raw
    return None

def clean_sha1(raw):
    raw = raw.strip()
    if len(raw.split()) != 1:
        return None
    pass

def clean_issn(raw):
    raw = raw.strip()
    if len(raw.split()) != 1:
        return None
    if len(raw) == 9 and raw[4] == "-" and raw[0:4].isdigit():
        return raw
    return None

def test_clean_issn():
    assert clean_issn("1234-4567") == "1234-4567"
    assert clean_issn("134-4567") == None
    assert clean_issn("123X-4567") == None

def clean_isbn13(raw):
    raw = raw.strip()
    if len(raw.split()) != 1:
        return None
    return None

def clean_orcid(raw):
    raw = raw.strip()
    if len(raw.split()) != 1:
        return None
    return None
