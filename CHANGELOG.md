
# CHANGELOG

Notable technical changes to the fatcat schemas, Rust API server, Python web
interface, and Python client library will be recorded here.

Intent is to follow semantic versioning, with all components kept in lock-step,
but note that everything is pre-1.0, so there aren't any actual backwards
compatibility guarantees. The API prefix (eg, `/v0/`) will probably roughly
track major version number, but TBD.

See also:

- release documentation in `./extra/release_process.md`
- [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- [Semantic Versioning](https://semver.org/spec/v2.0.0.html)


## [0.5.2] - UNRELEASED

### Changes

- 'stub' releases are not included in container preservation coverage "by year"
  and "by volume" charts. they are still included in overall counts.
- rust: upgraded 'macaroon' crate to upstream v0.3.0, instead of patched version

## [0.5.1] - 2022-01-26

This is a minor version bump to mark the upgrade of dependencies. There are no
schema changes, and the schema version has not been incremented (is still
`0.5.0`).

### Changes

- expected and tested deployment environment is now Ubuntu Focal and PostgreSQL 13
- minimum rust version is now 1.41. no new features are being used, but older
  versions will no longer be tested or supported
- default local development database now called `fatcat_dev`, not `fatcat`

### Fixed

- various typos and spelling errors corrected (using `codespell`)

### Added

- "alert banner" mechanism for web interface
- API server detects SQL transaction failure due to read-only mode, and returns
  appropriate 5xx error

## [0.5.0] - 2021-11-22

Small change to the API schema (and SQL schema), adding the `content_scope`
field to file, fileset, and webcapture entities. Because there is a SQL schema
change, bumping to version v0.5.0.

An outward-facing change is that the fatcat API server now attempts to
"stabilize" array order within JSON responses by sorting elements by
appropriate identifiers or other keys. In many cases, the order of things like
URLs for a file, manifest entries for a fileset, or releases associated with a
file, may be reversed in API responses compared to what was returned
previously. They should not match what was original supplied when the entity
was created.

In particular, this may cause broad discrepancies compared to historical bulk
metadata exports. New bulk exports will be generated with the new ordering.

A number of content cleanups and changes are also taking place to the primary
catalog (fatcat.wik), see the separate content CHANGELOG for details.

### Fixed

- API array order stablization, using `ORDER BY` in `fatcatd`. See note above.

### Changed

- broad python code style updates: formatting, lint rules, and type annotations
- a number of internal refactors of metadata importers
- stopped created a small number of Datacite-specific license slugs
- stopped trying to "fix" double slashes in DOIs, in most cases
- reduced amount of metadata stored in release `extra` field in Datacite
  importer

### Added

- `content_scope` field on file, fileset, and webcapture entities
- initial fileset importers
- JSON pseudo-API for reference string match/get interface
- editgroup "diff" view in web interface

### Removed

- deleted deprecated `cdl_dash_dat` and `wayback_static` one-time importers

## [0.4.0] - 2021-10-14

Includes small API and SQL schema changes; see
`./proposals/20211012_v04_schema_tweaks.md`. Old database dumps may not work
with this version of API server; migrations can be run using the `diesel`
command line tool to upgrade old dumps.

### Added

- reference graph views, based on fuzzy reference dataset in `refcat` and
  `fatcat-scholar` projects, stored in elasticsearch index
- savepapernow webcapture/html importer
- release schema: new `hdl` (handle) external identifier
- fileset schema: new `mimetype` field for files under manifest
- container schema: new `issne` and `issnp` top-level fields, which are
  supported in lookups, as well as the `issn` lookup field, which matches any
  of the `issn*` fields
- container schema: new `publication_status` field, to indicate if container is
  still actively publishing, discontinued, or "stub"
- above API schema changes added to elasticsearch 'release' and 'container' schemas
- editor "lookup" API endpoint, for fetching editor object by username
- web `/u/{username}` redirect helper for inspecting editors, bots, etc

### Fixed

- viewing deleted release entities no longer result in 500 error
- mediawiki (wikipedia) OAuth logins (bytes/str bug in fatcat code)
- savepapernow ingest importer force flushes every 5 minutes; should fix a bug
  with many SPN ingest requests not getting imported
- added 'version' field to release elasticsearch schema (was missing)
- several release external identifier validators were not actually being run
  before insertion

## [0.3.4] - 2021-05-25

Mostly infrastructure changes: Python, Elasticsearch, and other dependencies
changed.

No changes to schema, so API remains v0.3.3.

### Changes

- require Python 3.8 (upgrade from Python 3.7)
- require Elasticsearch 7.10 (upgrade from 6.x). Queries (reads) will work
  against ES 6.x, writes will not. 7.10 is the last upstream "open source"
  version of Elasticsearch, and we intend to stick with it until a community
  open source fork emerges
- update all Elasticsearch schemas to v03c. All at least include
  `doc_index_ts`, and small other changes
- container Elasticsearch schema includes aggregated release preservation
  numbers, as well as total count. Indexing pipeline updated to (optionally)
  query these stats at index-time
- several improvements to web editing workflow: clearer design of editgroup web
  view, etc
- change web UI and labeling on release and file views to emphasize
  preservation status and archival access
- update Flask to 2.x, and other related dependencies

### Added

- display preservation holdings summary on container view page


## [0.3.3] - 2020-12-24

Minor additions to the API schema: new external identifiers for release
entities, for `doaj`, `dblp`, and `oai`. Database schema (SQL) not changed.

### Added

- three new release external identifiers: `doaj`, `dblp`, and `oai` (all
  article-level). These are API changes, but backwards compatible.
- DOAJ release import
- dblp container and release import
- free-form "coverage search" page, allowing visualization of coverage based on
  elasticsearch query (web interface)
- editing of all entity types using TOML markup (via web interface)
- basic sitemap XML generation
- initial integration of fuzzycat library to prevent duplicate release entity
  creation at import time
- import of HTML webcaptures from sandcrawler ingest
- kafka publishing of updated work entities (transitively of release updates as
  part of the work), to enable work-level entity update pipeline for archive
  scholar index

### Changed

- require Python 3.7 (upgrade from Python 3.5)
- release entity exports can now be sorted by work identifier, for easier
  work-level grouping and analysis
- refactored webface search code to use elasticsearch client library

### Fixed

- several datacite metadata import bugs
- several other bugfixes to web interface and importer code, not reported here
  granularly

## [0.3.2] - 2020-04-08

This release was tagged retro-actively; it was the last commit before upgrading
to Python 3.7.

Many small changes and tweaks to importers, web interface, etc were made in
this release.

### Fixed

- pubmed importer `text` vs. `get_text()` for HTML tags
- fatcat-python-client package now works with Python 3.7, with removal of
  `async` keyword

### Changed

- fatcat-python-client re-code-generated using openapi generator instead of
  swagger tooling
- minimum rust version now 1.36
- Switch from swagger-codegen to openapi-generator for python client generation
- switch python Kafka code from pykafka to confluent-kafka
- update release and container elasticsearch schemas to v03b. Release search is
  now over "biblio" field, allowing matches on multiple fields at the same time
- Crossref harvester using 'update-date' not 'index-date' to detect updated documents

### Removed

- OpenSSL support removed from fatcatd (Rust)

#@# Added

- webface endpoints for entity view URLs with an underscore instead of slash,
  as a redirect. Eg, `https://fatcat.wiki/release_asdf` =>
  `https://fatcat.wiki/release/asdf`. A hack to make copy/paste easier.
- pagination of search results in web interface
- sandcrawler daily crawling pipeline, including ingest-file importer and
  publishing requests to sandcrawler kafka topic
- "Save Paper Now" feature (using sandcrawler pipeline)
- Datacite DOI registrar daily harvesting and importing
- Arxiv daily harvesting, using OAI-PMH worker
- Pubmed daily harvesting, using FTP worker
- "file" entity elasticsearch schema (though pipeline not yet running
  continuously)

## [0.3.1] - 2019-09-18

Many small web interface improvements, bug fixes, and importer additions are
not explicitly noted below.

### Changed

- renamed python client library `fatcat-openapi-client` and rust api-spec
  library `fatcat-openapi`
- updated rust (cargo) dependencies; depend on rust 1.34+ (Debian buster)
- entity view page design and backend code structure completely refactored

### Fixed

- compilation of `fatcat-openapi` rust crate (including examples) now actually
  works using 2018 edition. Many lint warnings were patched, making rust
  compilation cleaner with recent compiler versions.

### Added

- container "coverage" page, showing elasticsearch-generated chart of IA
  release archival coverage
- `create_auth_token` API endpoint, for generation of API tokens via web
  interface
- API token generation in web interface
- quick web interface search (front page and top bar) now detects additional
  identifiers, such as ISBN-13, SHA-1, and arxiv

## [0.3.0] - 2019-05-23

This version includes both a SQL migration and backwards-incompatible API
changes.

### Fixed

- `edit_extra` on individual entity create was not being written to database

### Changed

- `release_status` field (on releases) renamed to `release_stage`
- moved external identifier fields on release entities into a new sub-namespace
  called `ext_ids`. Eg, instead of `release.doi`, it's now
  `release.ext_ids.doi`. This field is always required for release entities, so
  fields can be accessed directly without a null/option check on the `ext_ids`
  field itself. This impacts `doi`, `pmid`, `pmcid`, etc.
- `pmcid` field now accepts versioned identifiers, with a trailing dot and
  version number, like `PMC1234.2`.
- many more restrictions on external identifiers at creation time.
- API endpoints that mutate entities are now prefixed with
  `/endpoint/{endpoint_id}/...`, which changes the `endpoint_id` parameter from
  optional to required (in the path). In client libraries, this also changed
  the order of parameters (`endpoint_id` now comes first). Batch edits now only
  work in "auto batch" mode, and an editgroup object must be included in the
  body (not just `extra` and `description` fields as query parameters).
- several additional editgroup/changelog/history endpoints now expand `editor`
  in editgroup objects by default.
- `created` timestamp is included in editgroup objects (when GET) by default.
- in client libraries, `FileEntityUrls` renamed `FileUrl`, and several similar
  singular/plural and `Entity` removed renamings.
- elasticsearch release schema updated to match API schema changes

### Removed

- Non-auto batch mode no longer implemented.

### Added

- release contribs may include `given_name` and `surname` fields, like
  `creator` objects, in addition to existing `raw_name`.
- add `withdrawn_status`, `withdrawn_year`, and `withdrawn_date` fields to
  releases
- added `retraction` as an allowable `release_type`, for a publication. When
  used, the `release_stage` should also be `retraction`.
- `subtitle` added as a top-level release field, along with `version` and
  `number`
- `ark_id` (for ARK identifiers) and `mag_id` (for Microsoft Academic Graph
  identifiers) added to releases (under `ext_id`)
- allow expanding `releases` for file, fileset, and webcapture entities.
  Expanded release entities have their abstracts and refs hidden by default
  (for performance)
- `creator_ids` in release elasticsearch schema, for lookups
- new importers: arxiv, pubmed, JALC, and JSTOR. Most still need refactoring
  and polish

## [0.2.2] - 2019-05-08

### Fixed

- fixed major authn/authz bug with fatcat python client which leaked API tokens
  between API client handles. Almost all tests/webfact/etc were potentially
  running with the privileged (superuser) webface-bot privileges. Yikes!
- API `get_editgroup_annotations` endpoint was requiring auth; this was a typo.
  Going to call this a very minor/backwards-compatible API change and not do a
  minor version bump for it.
- DOI lookups (for release entities) were not being lower-cased as intended
  (DOIs are stored lower-case internally)

### Added

- ORCID OAuth login provider

### Changed

- small tweaks to URL domain/rel mapping list

## [0.2.1] - 2019-04-09

No API or SQL schema changes in this release. Macaroon generation and
verification was broken; all non-CLI-generated tokens will need to be
regenerated (eg, log-out, log-in).

### Fixed

- fix macaroon 'expires' caveat check (in rust), allowing actual login via OAuth/IA

### Added

- basic release, container, file webface editing
- basic editgroup annotation and control (submit/accept) in webface
- CSL/BibTeX citation endpoints
- "review bot" framework (for CI-style edit checks)
- wikipedia OAuth login provider

### Changed

- expanded example entities in SQL schema
- updated rust (cargo) dependencies; depend on rust 1.32+
- many tweaks to webface and guide

## [0.2.0] - 2019-02-05

First tagged release, and start of notable change tracking.
