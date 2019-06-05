#!/usr/bin/env bash

set -e -u -o pipefail

export LC_ALL=C

EXTID_FILE=$1

zcat $EXTID_FILE \
    | awk '{print $3 "\t" $1}' \
    | rg -v '^\t' \
    | sort -S 4G \
    > doi_ident.tsv
zcat $EXTID_FILE \
    | awk '{print $4 "\t" $1}' \
    | rg -v '^\t' \
    | sort -S 4G \
    > pmid_ident.tsv
zcat $EXTID_FILE \
    | awk '{print $5 "\t" $1}' \
    | rg -v '^\t' \
    | sort -S 4G \
    > pmcid_ident.tsv
zcat $EXTID_FILE \
    | awk '{print $6 "\t" $1}' \
    | rg -v '^\t' \
    | sort -S 4G \
    > wikidata_ident.tsv

# these identifiers aren't fixed-width, so we need to join (sigh)
cut -f1 doi_ident.tsv \
    | uniq -d \
    | join -t$'\t' - doi_ident.tsv \
    > doi_ident.dupes.tsv
cut -f1 pmid_ident.tsv \
    | uniq -d \
    | join -t$'\t' - pmid_ident.tsv \
    > pmid_ident.dupes.tsv
cut -f1 pmcid_ident.tsv \
    | uniq -d \
    | join -t$'\t' - pmcid_ident.tsv \
    > pmcid_ident.dupes.tsv
cut -f1 wikidata_ident.tsv \
    | uniq -d \
    | join -t$'\t' - wikidata_ident.tsv \
    > wikidata_ident.dupes.tsv

wc -l doi_ident.dupes.tsv pmid_ident.dupes.tsv pmcid_ident.dupes.tsv wikidata_ident.dupes.tsv >> counts.txt

