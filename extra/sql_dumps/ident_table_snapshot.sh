#!/bin/bash

set -e -u -o pipefail

DATESLUG="`date +%Y-%m-%d.%H%M%S`"
DATABASE="fatcat"

echo "Running SQL..."
psql fatcat < ./dump_idents.sql

CHANGELOG_REV="`head -n1 /tmp/fatcat_ident_latest_changelog.tsv`"
OUTFILE="fatcat_idents.$DATESLUG.r$CHANGELOG_REV.tar.xz"

echo "Compressing..."
tar -C /tmp -c --xz --verbose \
    -f $OUTFILE \
    fatcat_ident_latest_changelog.tsv \
    fatcat_ident_containers.tsv \
    fatcat_ident_creators.tsv \
    fatcat_ident_files.tsv \
    fatcat_ident_releases.tsv \
    fatcat_ident_works.tsv

echo "Done: $OUTFILE"
