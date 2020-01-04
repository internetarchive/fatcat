
Status: planned

## Schema Changes for v0.4 Release

Proposed schema changes for next fatcat iteration (v0.4? v0.5?).

SQL (and API, and elasticsearch):

- container:`container_status` as a string enum: eg, "stub",
  "out-of-print"/"ended" (?), "active", "new"/"small" (?).  Particularly to
  deal with disambiguation of multiple containers by the same title but
  separate ISSN-L. For example, "The Lancet".
- release: `release_month` (to complement `release_date` and `release_year`)
- file: `file_scope` as a string enum indicating how much content this file
  includes. Eg, `book`, `chapter`, `article`/`work`, `issue`, `volume`,
  `abstract`, `component`. Unclear how to initialize this field; default to
  `article`/`work`?
- TODO: release: switch how pages work? first/last?
- TODO: indication of peer-review process? at release or container level?
- TODO: container: separate canonical and disambiguating titles (?)
- TODO: release inter-references using SCHOLIX/Datacite schema
    https://zenodo.org/record/1120265
    https://support.datacite.org/docs/connecting-research-outputs#section-related-identifiers

API tweaks:

- add regex restrictions on more `ext_ids`, especially `wikidata_qid`
- add explicit enums for more keyword fields

API endpoints:

- `GET /auth/token/<editor_id>` endpoint to generate new API token for given
  editor. Used by web interface, or bot wranglers.
- create editor endpoint, to allow bot account creation
- `GET /editor/<ident>/bots` (?) endpoint to enumerate bots wrangled by a
  specific editor

Elasticsearch schema:

- releases *may* need an "_all" field (or `biblio`?) containing most fields to
  make some search experiences work
- releases should include volume, issue, pages
- releases *could* include reference and creator lists, as a faster/cheaper
  mechanism for doing reverse lookups
