
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

## Setting Up Elasticsearch

We use elasticsearch version 6.x, with the `analysis-icu` plugin installed:

    sudo /usr/share/elasticsearch/bin/elasticsearch-plugin install analysis-icu
    sudo service elasticsearch restart

There is a Dockerfile in this directory which includes this installation.

## Loading Data

Drop and rebuild the schema:

    http delete :9200/fatcat_release
    http delete :9200/fatcat_container
    http delete :9200/fatcat_file
    http delete :9200/fatcat_changelog
    http put :9200/fatcat_release < release_schema.json
    http put :9200/fatcat_container < container_schema.json
    http put :9200/fatcat_file < file_schema.json
    http put :9200/fatcat_changelog < changelog_schema.json

Put a single object (good for debugging):

    head -n1 examples.json | http post :9200/fatcat_release/release/0
    http get :9200/fatcat_release/release/0

Bulk insert from a file on disk:

    esbulk -verbose -id ident -index fatcat_release -type release examples.json

Or, in a bulk production live-stream conversion:

    export LC_ALL=C.UTF-8
    time zcat /srv/fatcat/snapshots/release_export_expanded.json.gz | pv -l | parallel -j20 --linebuffer --round-robin --pipe ./fatcat_transform.py elasticsearch-releases - - | esbulk -verbose -size 1000 -id ident -w 8 -index fatcat_release -type release
    time zcat /srv/fatcat/snapshots/container_export.json.gz | pv -l | ./fatcat_transform.py elasticsearch-containers - - | esbulk -verbose -size 1000 -id ident -w 8 -index fatcat_container -type container
    time zcat /srv/fatcat/snapshots/file_export.json.gz | pv -l | parallel -j20 --linebuffer --round-robin --pipe ./fatcat_transform.py elasticsearch-files - - | esbulk -verbose -size 1000 -id ident -w 8 -index fatcat_file -type file

## Index Aliases

To make re-indexing and schema changes easier, we can create versioned (or
time-stamped) elasticsearch indexes, and then point to them using index
aliases. The index alias updates are fast and atomic, so we can slowly build up
a new index and then cut over with no downtime.

    http put :9200/fatcat_release_v03 < release_schema.json

To replace a "real" index with an alias pointer, do two actions (not truely
zero-downtime, but pretty fast):

    http delete :9200/fatcat_release
    http put :9200/fatcat_release_v03/_alias/fatcat_release

To do an atomic swap from one alias to a new one ("zero downtime"):

    http post :9200/_aliases << EOF
        {
            "actions": [
                { "remove": { "index": "fatcat_release_v03", "alias": "fatcat_release" }},
                { "add":    { "index": "fatcat_release_v04", "alias": "fatcat_release" }}
            ]
        }
    EOF

## Full-Text Querying

A generic full-text "query string" query look like this (replace "blood" with
actual query string, and "size" field with the max results to return):

    GET /fatcat_release/release/_search
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
