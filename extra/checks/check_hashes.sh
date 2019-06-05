#!/usr/bin/env bash

set -e -u -o pipefail

export LC_ALL=C

HASH_FILE=$1

zcat $HASH_FILE \
    | awk '{print $3 "\t" $1}' \
    | rg -v '^\t' \
    | sort -S 4G \
    | uniq -d -w 40 \
    > sha1_ident.dupes.tsv

wc -l sha1_ident.dupes.tsv >> counts.txt
