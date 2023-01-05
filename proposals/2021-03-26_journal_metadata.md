
status: brainstorm

What improved journal-level metadata could we store?


## Names, Aliases

Translated names, as a dict of { lang: name }


## API Endpoints

OAI-PMH endpoint and type (for ingest)


## Homepage URLs


## Fulltext URL Info

Fulltext SURT prefix/pattern, by type and year range

    surt_prefix
    url_regex
    url_pattern
        including ext_id pattern substitutions; can generate URL from release entity
    mimetype
    year_span
        optional
    confidence
        "if not this pattern, then isn't published version"
        "if matches, definitely fulltext"
        "might be fulltext, might not"
        etc. as a slug/code


## Other

for releases, could store DOAJ access URL in release extra metadata
