# Cookbook

### Updating an Existing Entity

1. Fetch (GET) the existing entity
2. Create (POST) a new editgroup
3. Update (PUT) the entity, with the current revision number in the `prev` edit
   field, and the `editgroup_id` set
4. Submit (PUT) the editgroup for review
5. Somebody (human or bot) with admin privileges will Accept (POST) the editgroup.

### Merging Duplicate Entities

1. Fetch (GET) both entities
2. Decide which will be the "primary" entity (the other will redirect to it)
3. Create (POST) a new editgroup
4. Update (PUT) the "primary" entity with any updated metadata merged from the
   other entity (optional), and the editgroup id set
5. Update (PUT) the "other" entity with the redirect flag set to the primary's
   identifier, with the current revision id (of the "other" entity) in the
   `prev` field, and the editgroup id set
6. Submit (PUT) the editgroup for review
7. Somebody (human or bot) with admin privileges will Accept (POST) the editgroup.

### Lookup Fulltext URLs by DOI

1. Use release lookup endpoint (GET) with the `doi` query parameter in
   URL-escaped format, with `expand=files`. You may want to `hide`
   `abstracts,references` for faster responses if you aren't interested in
   those fields.
2. If a release hit is found, iterate over the linked `file` entities, and
   create a ranked list of URLs based on mimetype, URL "rel" type, file size,
   or host domain.

### Batch Insert New Entities (Bootstrapping)

When bootstrapping a blank catalog, we need to insert 10s or 100s of millions
of entities as fast as possible.

1. Batch create (POST) a set of entities, with editgroup metadata included
   along with list of entities (all of a single type). Entire batch is inserted
   and the editgroup accepted (requiring admin bits) in a single transaction.

