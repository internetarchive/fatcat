# Common Entity Fields

All entities have:

- `extra` (dict, optional): free-form JSON metadata

The "extra" field is an "escape hatch" to include extra fields not in the
regular schema. It is intended to enable gradual evolution of the schema, as
well as accommodating niche or field-specific content. Reasonable care should
be taken with this extra metadata: don't include large text or binary fields,
hundreds of fields, duplicate metadata, etc.

All full entities (distinct from revisions) also have the following fields:

- `state` (string, read-only): summarizes the status of the entity in the
  catalog. One of a small number of fixed values, see vocabulary below.
- `ident` (string, Fatcat identifier, read-only): the Fatcat entity identifier
- `revision` (string, UUID): the current revision record that this entity
  `ident` points to
- `redirect` (string, Fatcat identifier, optional): if set, this entity ident
  has been redirected to the `redirect` one. This is a mechanism of merging or
  "deduplicating" entities.
- `edit_extra` (dict, optional): not part of the bibliographic schema, but can
  be included when creating or updating entities using the API, and the
  contents of field will be included in the entity's edit history.


## `state` Vocabulary

- `active`: entity exists in the catalog
- `redirect`: the entity `ident` exists in the catalog, but is a redirect to
  another entity ident.
- `deleted`: an entity with the `ident` did exist in the catalog previously,
  but it was deleted. The `ident` is retained as a "tombstone" record (aka,
  there is a record that an entity *did* exist previously).
- `wip` ("Work in Progress"): an entity identifier has been created as part of
  an editgroup, but that editgroup has not been accepted yet into the catalog,
  and there is no previous/current version of the entity.
