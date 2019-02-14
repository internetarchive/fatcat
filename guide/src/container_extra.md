
'extra' fields:

    doaj
        as_of: datetime of most recent check; if not set, not actually in DOAJ
        seal: bool
        work_level: bool (are work-level publications deposited with DOAJ?)
        archiving: array, can include 'library' or 'other'
    road
        as_of: datetime of most recent check; if not set, not actually in ROAD
    pubmed (TODO: delete?)
        as_of: datetime of most recent check; if not set, not actually indexed in pubmed
    norwegian (TODO: drop this?)
        as_of: datetime of most recent check; if not set, not actually indexed in pubmed
        id (integer)
        level (integer; 0-2)
    kbart
        lockss
            year_rle
            volume_rle
        portico
            ...
        clockss
            ...
    sherpa_romeo
        color
    jstor
        year_rle
        volume_rle
    scopus
        id
        TODO: print/electronic distinction?
    wos
        id
    doi
        crossref_doi: DOI of the title in crossref (if exists)
        prefixes: array of strings (DOI prefixes, up to the '/'; any registrar, not just Crossref)
    ia
        sim
            nap_id
            year_rle
            volume_rle
        longtail: boolean
        homepage
            as_of: datetime of last attempt
            url
            status: HTTP/heritrix status of homepage crawl

    issnp: string
    issne: string
    coden: string
    abbrev: string
    oclc_id: string (TODO: lookup?)
    lccn_id: string (TODO: lookup?)
    dblb_id: string
    default_license: slug
    original_name: native name (if name is translated)
    platform: hosting platform: OJS, wordpress, scielo, etc
    mimetypes: array of strings (eg, 'application/pdf', 'text/html')
    first_year: year (integer)
    last_year: if publishing has stopped
    primary_language: single ISO code, or 'mixed'
    languages: array of ISO codes
    region: TODO: continent/world-region
    nation: shortcode of nation
    discipline: TODO: highest-level subject; "life science", "humanities", etc
    field: TODO: narrower description of field
    subjects: TODO?
    url: homepage
    is_oa: boolean. If true, can assume all releases under this container are "Open Access"
    TODO: domains, if exclusive?
    TODO: fulltext_regex, if a known pattern?

For KBART, etc:
    We "over-count" on the assumption that "in-progress" status works will soon actually be preserved.
    year and volume spans are run-length-encoded arrays, using integers:
        - if an integer, means that year is preserved
        - if an array of length 2, means everything between the two numbers (inclusive) is preserved
