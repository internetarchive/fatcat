#!/usr/bin/env bash

set -e              # fail on error
set -u              # fail if variable not set in substitution
set -o pipefail     # fail if part of a '|' command fails

: ${1?' You you did not supply a date argument'}
: ${2?' You you did not supply an input file (JSON gzip)'}
if [ ! -f $2 ] ; then
  echo "Input file not found: $2" && exit 1;
fi

# eg, 2020-08-19
DATE="$1"
# eg, container_export.json.gz
EXPORT_FILE_GZ="$2"

zcat $EXPORT_FILE_GZ \
    | jq .ident -r \
    | awk '{print "https://fatcat.wiki/container/" $1 }' \
    | split --lines 20000 - sitemap-containers-$DATE- -d -a 5 --additional-suffix .txt

gzip sitemap-containers-*.txt
