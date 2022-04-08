#!/usr/bin/env bash

set -euo pipefail

export CONTAINER_INDEX=fatcat_container_v05_20220110

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
    | esbulk -verbose -index $CONTAINER_INDEX -optype update -id ident
