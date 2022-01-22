
Here are the fields we want to populate:

    "releases_total":           { "type": "integer" },
    "preservation_bright":      { "type": "integer" },
    "preservation_dark":        { "type": "integer" },
    "preservation_shadows_only":{ "type": "integer" },
    "preservation_none":        { "type": "integer" },

Populate local index for testing:

    fatcat-cli search container --index-json --limit 100 state:active \
        | pv -l \
        > container_es_docs.json

    cat container_es_docs.json \
        | esbulk -verbose -index fatcat_container_v03c -id ident

    cat container_es_docs.json \
        | jq .ident -r \
        > container_idents.tsv

Quick way to dump all idents in the current index:

    fatcat-cli search container --index-json --limit 0 state:active \
        | jq .ident -r \
        | pv -l \
        > container_idents.tsv

    cat container_idents.tsv \
        | parallel -j10 curl --fail -s 'https://fatcat.wiki/container/{}/stats.json' \
        | jq -c . \
        | pv -l \
        > container_stats.json

    cat container_stats.json \
        | jq '{ ident: .ident, releases_total: .total, preservation_bright: .preservation.bright, preservation_dark: .preservation.dark, preservation_shadows_only: .preservation.shadows_only, preservation_none: .preservation.none }' -c \
        | esbulk -verbose -index fatcat_container_v03c -optype update -id ident

This requires a recent version of esbulk (v0.7.5+)

