
Status: implemented

# Release External ID Refactor

Goal is to make the external identifier "namespace" (number of external
identifiers supported) less scarce, while still allowing fast lookups. Adding
new identifiers would still require API schema and rust code changes.

This change would also bring the number of columns on `release_rev` back under
32, which makes diesel happier.

Unclear whether existing extids (like DOI) should get merged and the old
columns dropped. This could be done consistently by, eg, a rust worker process
that re-writes tables, probably in multiple stages (eg, first copy, then update
code to read from new location, then drop old columns and indices). Perhaps for
very hot/popular columns (like DOI, and maybe ISBN13?) it is better to have
separate columns/indices.

It would be possible to roll this out as a SQL change first, with identifiers
still at the top level of the API schema, then later switch the API schema. Not
sure this is worth it though.

## New API

All identifers as text

    release_entity
        ext_ids (required)
            doi
            pmid
            pmcid
            wikidata_qid
            core
            isbn13
            arxiv
            jstor
            [...]

## New SQL Schema

Something like:

    release_rev_extid (
        release_rev_id  UUID foreign key to release_rev.id
        extid_type      TEXT
        extid_value     TEXT
    )
    PRIMARY KEY (release_rev_id, extid_type)
    INDEX (extid_type, extid_value)

### Existing prod Column Use

Queries like:

    fatcat_prod=# SELECT COUNT(*) FROM release_rev WHERE wikidata_qid IS NOT NULL;
    13460723

Results:

    wikidata_qid: 13460723
    isbn13: 1
    core_id: 7160477
    arxiv_id: 3
    jstor_id: 0
    pmcid: 3688945

Keep in SQL:

- `doi`
- `pmid`
- `pmcid`
- `wikidata_qid`
- `core_id`

Drop columns:

- `isbn13`
- `arxiv_id`
- `jstor_id`

In new table:

- isbn13
- arxiv
- jstor
- mag
- ark
- dblp
