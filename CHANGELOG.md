
# CHANGELOG

Notable technical changes to the fatcat schemas, Rust API server, Python web
interface, and Python client library will be recorded here.

Intent is to follow semantic versioning, with all components kept in lock-step,
but note that everything is pre-1.0, so there aren't any actual backwards
compatibility guarantees. The API prefix (eg, `/v0/`) will probably roughly
track major version number, but TBD.

See also:

- [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

## [Unreleased]

## [0.3.1] - 2019-09-18

Many small web interface improvements, bug fixes, and importer additions are
not explicitly noted below.

### Changed

- renamed python client library `fatcat-openapi-client` and rust api-spec
  library `fatcat-openapi`
- updated rust (cargo) dependencies; depend on rust 1.34+ (Debian buster)
- entity view page design and backend code structure completely refactored

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
  singlular/plural and `Entity` removed renamings.
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
