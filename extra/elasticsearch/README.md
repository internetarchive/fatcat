
# Elasticsearch Schemas and Pipeline Docs

Eventually, we might end up with schemas for multiple entity types, and in
particular glom/merge releases under their work, but for now we just have a
release-oriented schema that pulls in collection and files metadata.

Elasticsearch has at least two uses: user-facing search for entities, and
exploring aggregate numbes.

The schema tries to stay close to the release entity type, but adds some extra
aggregated fields and flags.

The simple batch update pipeline currently in use is to:

- make a fresh "expanded" release entity dump (JSON)
- transform using `parallel` and a python script
- bulk import into elastic using `esbulk`

In the future, it would be nice to have a script that "tails" the changelog for
edits and updates just those entities in the database. This is somewhat
non-trivial because the "expand" data requires more sophisticated cache
invalidation (entity updates), particularly in the case where an inter-entity
relation is *removed*. For example, if a file match against a given release is
removed, the old release elastic object needs to be updated to remove the file
from it's `files`.

## TODO

"enum" types, distinct from "keyword"?

Other identifiers in search index? core, wikidata
