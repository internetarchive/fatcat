#!/bin/bash

DATABASE_URL="${DATABASE_URL:-fatcat}"
OUTPUT_DIR="${1:-.}"
set -e -u -o pipefail

# This script needs to be run from the directory with the 'dump_idents.sql'
# script in it.

# The DATABASE_URL env variable is optional, defaults to 'fatcat' (meaning,
# local postgres, database named 'fatcat')

# An optional argument is a path to a directory to save output.

DATESLUG="`date +%Y-%m-%d.%H%M%S`"

echo "Will move output to '${OUTPUT_DIR}'"
echo "Running SQL (from '${DATABASE_URL}')..."
psql $DATABASE_URL < ./dump_idents.sql

CHANGELOG_REV="`head -n1 /srv/fatcat/tmp/fatcat_ident_latest_changelog.tsv`"
OUTFILE="${OUTPUT_DIR}/fatcat_idents.$DATESLUG.r$CHANGELOG_REV.tar.gz"

echo "Compressing..."
tar -C /srv/fatcat/tmp -c --verbose \
    --use-compress-program=pigz \
    -f $OUTFILE \
    fatcat_ident_latest_changelog.tsv \
    fatcat_ident_containers.tsv \
    fatcat_ident_creators.tsv \
    fatcat_ident_files.tsv \
    fatcat_ident_filesets.tsv \
    fatcat_ident_webcaptures.tsv \
    fatcat_ident_releases.tsv \
    fatcat_ident_works.tsv \
    fatcat_ident_releases_by_work.tsv

echo "Done: $OUTFILE"
