
## In Progress

## Next Up

- formalize release_status:
    => https://wiki.surfnet.nl/display/DRIVERguidelines/DRIVER-VERSION+Mappings
- page-one.live.cf.public.springer.com seems to serve up bogus one-pagers; should exclude
- QA sentry has very little host info; also not URL of request
- should elastic release_year be of date type, instead of int?
- subtitle as array vs. string

## Production Public Launch Blockers

- update /about page
- handle 'wip' status entities in web UI
- guide updates for auth
- privacy policy, and link from: create account, create edit

## Production Tech Sanity

- postgresql replication
- haproxy somewhere/how
- logging iteration: larger journald buffers? point somewhere?

## Ideas

- ansible: ISSN-L download/symlink
- webface: still need to collapse links by domain better, and also vs. www.x/x
- entity edit JSON objects could include `entity_type`
- refactor 'fatcatd' to 'fatcat-api'
- changelog elastic stuff (is there even a fatcat-export for this?)
- container count "enrich"
- ORCID apparently has 37 mil "work activities" (patents, etc), and only 14 mil
  unique DOIs; could import those other "work activities"? do they have
  identifiers?
- 'hide' flag for exporter (eg, to skip abstracts and refs in some release dumps)
- https://tech.labs.oliverwyman.com/blog/2019/01/14/serialising-rust-tests/
- use https://github.com/codelucas/newspaper to extract fulltext+metadata from HTML crawls
- changelog elastic index (for stats)
- import from arabesque output (eg, specific crawls)
- more logins: orcid, wikimedia
- `fatcat-auth` tool should support more caveats, both when generating new or mutating existing tokens
- fast path to skip recursive redirect checks for bulk inserts
- when getting "wip" entities, require a parameter ("allow_wip"), else get a 404
- consider dropping CORE identifier
- maybe better 'success' return message? eg, "success: true" flag
- idea: allow users to generate their own editgroup UUIDs, to reduce a round
  trips and "hanging" editgroups (created but never edited)
- API: allow deletion of empty, un-accepted editgroups
- refactor API schema for some entity-generic methos (eg, history, edit
  operations) to take entity type as a URL path param. greatly reduce macro
  foolery and method count/complexity, and ease creation of new entities
    => /{entity}/edit/{edit_id}
    => /{entity}/{ident}/redirects
    => /{entity}/{ident}/history
- investigate data quality by looking at, eg, most popular author strings, most popular titles, duplicated containers, etc

## Metadata Import

- 158 "NULL" publishers in journal metadata
- crossref: many ISBNs not getting copied; use python library to convert?
- remove 'first' from contrib crossref 'seq' (not helpful?)
- should probably check for 'jats:' in abstract before setting mimetype, even from crossref
- web.archive.org response not SHA1 match? => need /<dt>id_/ thing
- XML etc in metadata
    => (python) tests for these!
    https://qa.fatcat.wiki/release/search?q=xmlns
    https://qa.fatcat.wiki/release/search?q=%24gt
- bad/weird titles
    "[Blank page]", "blank page"
    "Temporary Empty DOI 0"
    "ADVERTISEMENT"
    "Full title page with Editorial board (with Elsevier tree)"
    "Advisory Board Editorial Board"
- better/complete reltypes probably good (eg, list of IRs, academic domain)
- 'expand' in lookups (derp! for single hit lookups)
- include crossref-capitalized DOI in extra
- manifest: multiple URLs per SHA1
- crossref: relations ("is-preprint-of")
- crossref: two phase: no citations, then matched citations (via DOI table)
- special "alias" DOIs... in crossref metadata?

new importers:
- pubmed (medline) (filtered)
    => and/or, use pubmed ID lookups on crossref import
- arxiv.org
- DOAJ
- CORE (filtered)
- semantic scholar (up to 39 million; includes author de-dupe)

## Guide / Book / Style

- release_type, release_status, url.rel schemas (enforced in API)
- more+better terms+policies: https://tosdr.org/index.html

## Fun Features

- "save paper now"
    => is it in GWB? if not, SPN
    => get hash + url from GWB, verify mimetype acceptable
    => is file in fatcat?
    => what about HBase? GROBID?
    => create edit, redirect user to editgroup submit page
- python client tool and library in pypi
    => or maybe rust?
- bibtext (etc) export

## Metadata Harvesting

- datacite ingest seems to have failed... got a non-HTTP-200 status code, but also "got 50 (161950 of 21084)"

## Schema / Entity Fields

- elastic transform should only include authors, not editors (?)
- `retracted`, `translation`, and perhaps `corrected` as flags on releases, instead of release_status?
    => see notes file on retractions, etc
- 'part-of' relation for releases (release to release, eg for book chapters) and possibly containers
- `container_type` for containers (journal, conference, book series, etc)
    => in schema, needs vocabulary and implementation

## API Schema / Design

- refactor entity mutation (CUD) endpoints to be like `/editgroup/{editgroup_id}/release/{ident}`
    => changes editgroup_id from query param to URL param
- refactor bulk POST to include editgroup plus array of entity objects (instead of just a couple fields as query params)

## Web Interface

- include that ISO library to do lang/country name decodes
- container-name when no `container_id`. eg: 10.1016/b978-0-08-037302-7.50022-7
- fileset/webcapture webface anything

## Other / Backburner

- file entity full update with all hashes, file size, corrected/expanded wayback links
    => some number of files *did* get inserted to fatcat with short (year) datetimes, from old manifest. also no file size.
- searching 'N/A' is a bug, because not quoted; auto-quote it?
- regression test imports for missing orcid display and journal metadata name
- try out beautifulsoup? (https://stackoverflow.com/a/34532382/4682349)
- `doi` field for containers (at least for "journal" type; maybe for "series" as well?)
- refactor webface views to use shared entity_view.html template
- shadow library manifest importer
- book identifiers: OCLC, openlibrary
- ref from guide: https://creativecommons.org/2012/08/14/library-catalog-metadata-open-licensing-or-public-domain/
- test redirect/delete elasticsearch change
- fake DOI (use in examples): 10.5555/12345678
- refactor elasticsearch inserter to be a class (eg, for command line use)
- document: elastic query date syntax is like: date:[2018-10-01 TO 2018-12-31]
- display abstracts better. no hashes or metadata; prefer plain or HTML,
  convert JATS if necessary
- switch from slog to simple pretty_env_log
- format returned datetimes with only second precision, not millisecond (RFC mode)
    => burried in model serialization internals
- refactor openapi schema to use shared response types
- consider using "HTTP 202: Accepted" for entity-mutating calls
- basic python hbase/elastic matcher
  => takes sha1 keys
  => checks fatcat API + hbase
  => if not matched yet, tries elastic search
  => simple ~exact match heuristic
  => proof-of-concept, no tests
- add_header Strict-Transport-Security "max-age=3600";
    => 12 hours? 24?
- haproxy for rate-limiting

better API docs
- readme.io has a free open source plan (or at least used to)
- https://github.com/readmeio/api-explorer
- https://github.com/lord/slate
- https://sourcey.com/spectacle/
- https://github.com/DapperDox/dapperdox

CSL:
- https://citationstyles.org/
- https://github.com/citation-style-language/documentation/blob/master/primer.txt
- https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html
- https://github.com/citation-style-language/schema/blob/master/csl-types.rnc
- perhaps a "create from CSL" endpoint?
