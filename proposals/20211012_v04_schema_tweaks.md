
Status: implemented

## Schema Changes for v0.4

Small SQL and API changes. Calling these a minor-level API version increment.

API Schema Changes:

- release `ext_ids`: `hdl` (handle) identifier
- fileset: `mimetype` for manifest files as a field. This is a SQL schema change as well.
- container: `issne` and `issnp` as top-level fields, indexed for lookup. SQL
  schema change.
- container: `publication_status` as a top-level field, to indicate "active",
  "discontinued", etc. SQL schema change.

API Endpoints:

- `GET /editor/lookup`: editor lookup by username

Elasticsearch Schemas:

- release: 'hdl' identifier
- release: `container_publication_status` and `container_issns`
- release: add missing `version` field (not related to any API change)
- release: add `tags` for future extensibility
- release: `is_work_alias` boolean flag for unversioned releases which point
  to the overall work, or the latest published version of the work. Included
  from field with the same name in release `extra`.
- container: `publication_status`
