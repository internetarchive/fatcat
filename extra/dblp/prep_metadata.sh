#!/usr/bin/env bash

# run this as 'fatcat' user on a production machine
#export FATCAT_API_HOST="https://api.fatcat.wiki/v0"

set -e -u -o pipefail

# ensure deps
#alias fd=fdfind
fd -h > /dev/null
fatcat-cli -h > /dev/null
pipenv -h > /dev/null

# ensure pipenv is ready
pipenv install
pipenv run true


wget -c 'https://dblp.org/xml/dblp.dtd'
wget -c 'https://dblp.org/xml/dblp.xml.gz'

zcat dblp.xml.gz > dblp.xml

cd ../../python
pipenv run ./fatcat_import.py dblp-release ../extra/dblp/dblp.xml --dump-json-mode | pv -l > ../extra/dblp/dblp_releases_partial.json

cd ../extra/dblp/

cat dblp_releases_partial.json | jq ._dblp_prefix -r | grep -v ^null | rg '^(journals|conf|series)' | sort -u > prefix_list.txt

mkdir -p journals
mkdir -p conf
mkdir -p series

shuf prefix_list.txt | pv -l | parallel -j1 wget -nc -q "https://dblp.org/db/{}/index.html" -O {}.html

# clean up any failed/empty files, then re-run the above parallel/wget command
find . -empty -type f -delete

shuf prefix_list.txt | pv -l | parallel -j1 wget -nc -q "https://dblp.org/db/{}/index.html" -O {}.html

find . -empty -type f -delete

fd -I html conf/ journals/ series/ | pipenv run ./dblp_html_extract.py | pv -l > dblp_container_meta.json

fatcat-cli search containers dblp_prefix:* -n 0 --index-json | jq "[.dblp_prefix, .ident] | @tsv" -r | pv -l > existing_dblp_containers.tsv

cat dblp_releases_partial.json | pipenv run ./dblp2ingestrequest.py - | pv -l | gzip > dblp_sandcrawler_ingest_requests.json.gz
