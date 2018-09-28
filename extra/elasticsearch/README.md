
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

## Loading Data

Drop and rebuild the schema:

    http delete :9200/fatcat
    http put :9200/fatcat < release_schema.json

Put a single object (good for debugging):

    head -n1 examples.json | http post :9200/fatcat/release/0
    http get :9200/fatcat/release/0

Bulk insert from a file on disk:

    esbulk -verbose -id ident -index fatcat -type release examples.json

Or, in a bulk production live-stream conversion:

    export LC_ALL=C.UTF-8
    time zcat /srv/fatcat/snapshots/fatcat_release_dump_expanded.json.gz | ./transform_release.py | esbulk -verbose -size 20000 -id ident -w 8 -index fatcat -type release
    # 2018/09/24 21:42:26 53028167 docs in 1h0m56.853006293s at 14501.039 docs/s with 8 workers

## Full-Text Querying

A generic full-text "query string" query look like this (replace "blood" with
actual query string, and "size" field with the max results to return):

    GET /fatcat/release/_search
    {
      "query": {
        "query_string": {
          "query": "blood",
          "analyzer": "textIcuSearch",
          "default_operator": "AND",
          "analyze_wildcard": true,
          "lenient": true,
          "fields": ["title^5", "contrib_names^2", "container_title"]
        }
      },
      "size": 3
    }

In the results take `.hits.hits[]._source` as the objects; `.hits.total` is the
total number of search hits.

## TODO

- file URL domains? seems heavy
