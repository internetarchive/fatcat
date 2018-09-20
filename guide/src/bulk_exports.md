# Bulk Exports

There are a few different database dump formats folks might want:

- raw native database backups, for disaster recovery (would include
  volatile/unsupported schema details, user API credentials, full history,
  in-process edits, comments, etc)
- a sanitized version of the above: roughly per-table dumps of the full state
  of the database. Could use per-table SQL expressions with sub-queries to pull
  in small tables ("partial transform") and export JSON for each table; would
  be extra work to maintain, so not pursuing for now.
- full history, full public schema exports, in a form that might be used to
  mirror or enitrely fork the project. Propose supplying the full "changelog"
  in API schema format, in a single file to capture all entity history, without
  "hydrating" any inter-entity references. Rely on separate dumps of
  non-entity, non-versioned tables (editors, abstracts, etc). Note that a
  variant of this could use the public interface, in particular to do
  incremental updates (though that wouldn't capture schema changes).
- transformed exports of the current state of the database (aka, without
  history). Useful for data analysis, search engines, etc. Propose supplying
  just the Release table in a fully "hydrated" state to start. Unclear if
  should be on a work or release basis; will go with release for now. Harder to
  do using public interface because of the need for transaction locking.
