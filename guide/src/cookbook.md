# Cookbook

### Updating an Existing Entity

1. Fetch (GET) the existing entity
2. Create (POST) a new editgroup
3. Update (PUT) the entity, with the current revision number in the `prev` edit
   field, and the editgroup id set
4. Submit (POST? TBD) the editgroup for review

### Merging Duplicate Entities

1. Fetch (GET) both entities
2. Decide which will be the "primary" entity (the other will redirect to it)
3. Create (POST) a new editgroup
4. Update (PUT) the "primary" entity with any updated metadata merged from the
   other entity (optional), and the editgroup id set
5. Update (PUT) the "other" entity with the redirect flag set to the primary's
   identifier, with the current revision id (of the "other" entity) in the
   `prev` field, and the editgroup id set
4. Submit (POST? TBD) the editgroup for review

### Lookup Fulltext URLs by DOI

1. Use release lookup endpoint (GET) with the DOI a query parameter, with
   `expand=files`
2. If a release hit is found, iterate over the linked `file` entities, and
   create a ranked list of URLs based on mimetype, URL "rel" type, file size,
   or host domain.

### Batch Insert New Entities (Bootstrapping)

When bootstrapping a blank catalog, we need to insert 10s or 100s of millions
of entities as fast as possible.

1. Create (POST) a new editgroup, with provenance information included
2. Batch create (POST) entities
