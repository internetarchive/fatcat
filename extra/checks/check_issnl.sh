#!/usr/bin/env bash

set -e -u -o pipefail

export LC_ALL=C

CONTAINER_DUMP=$1

zcat $CONTAINER_DUMP \
    | jq '[.issnl, .ident] | @tsv' -r \
    | sort -S 4G \
    | uniq -d -w 9 \
    > issnl_ident.dupes.tsv

wc -l issnl_ident.dupes.tsv >> counts.txt
