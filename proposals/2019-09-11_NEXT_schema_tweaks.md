
Status: planned

## Schema Changes for Next Release

Proposed schema changes for next fatcat iteration with SQL changes (v0.6? v1.0?).

SQL (and API, and elasticsearch):

- `db_get_range_for_editor` is slow when there are many editgroups for editor; add sorted index? meh.
- release: `release_month` (to complement `release_date` and `release_year`)
- file: `file_scope` as a string enum indicating how much content this file
  includes. Eg, `book`, `chapter`, `article`/`work`, `issue`, `volume`,
  `abstract`, `component`. Unclear how to initialize this field; default to
  `article`/`work`?
- file: some way of marking bad/bogus files... by scope? type? status?
- TODO: webcapture: lookup by primary URL sha1?
- TODO: release: switch how pages work? first/last?
- TODO: indication of peer-review process? at release or container level?
- TODO: container: separate canonical and disambiguating titles (?)
- TODO: container: "imprint" field?
- TODO: container: "series" field? eg for conferences
- TODO: release inter-references using SCHOLIX/Datacite schema
    https://zenodo.org/record/1120265
    https://support.datacite.org/docs/connecting-research-outputs#section-related-identifiers
- TODO: fileset: some sort of lookup; hashes of hashes?
- TODO: fileset: some indication/handling of git repositories

API tweaks:

- add regex restrictions on more `ext_ids`, especially `wikidata_qid`
- add explicit enums for more keyword fields

API endpoints:

- `GET /auth/token/<editor_id>` endpoint to generate new API token for given
  editor. Used by web interface, or bot wranglers.
- create editor endpoint, to allow bot account creation
- `GET /editor/<ident>/bots` (?) endpoint to enumerate bots wrangled by a
  specific editor

See `2020_search_improvements` for elasticsearch-only schema updates.
