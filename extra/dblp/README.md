
This file describes hacks used to import dblp container and release metadata.

The container metadata must be processed and imported first, to create
containers for non-ISSN venues. However, dblp only publishes structured
metadata for articles (releases), not venues (containers), so we need to
process the articles, then import the containers, then import the articles.

There is a path that scrapes venue metadata out of dblp.org HTML.


## New Process (2022)

Usually all of this gets run on a production fatcat instance. It may be
possible to run parts elsewhere, but not confirmed, and would require copying
some set of files around.

    # remove any old/stale files
    ./cleanup.sh

    ./prep_container_metadata.sh

This will take a while to run, after which the container metadata can be
imported, like:

    cd ../../python
    pipenv shell
    export FATCAT_AUTH_WORKER_DBLP=[...]
    ./fatcat_import.py dblp-container --issn-map-file /srv/fatcat/datasets/ISSN-to-ISSN-L.txt --dblp-container-map-file ../extra/dblp/existing_dblp_containers.tsv --dblp-container-map-output ../extra/dblp/all_dblp_containers.tsv ../extra/dblp/dblp_container_meta.json

Check that counts look sane:

    wc -l existing_dblp_containers.tsv all_dblp_containers.tsv dblp_container_meta.json prefix_list.txt

Then do release import like:

    cd ../../python
    pipenv shell
    export FATCAT_AUTH_WORKER_DBLP=[...]
    ./fatcat_import.py dblp-release --dblp-container-map-file ../extra/dblp/all_dblp_containers.tsv ../extra/dblp/dblp.xml

Lastly, to generate sandcrawler ingest requests, from the JSON-dumped partial
release objects::

    cat dblp_releases_partial.json | pipenv run ./dblp2ingestrequest.py - | pv -l | gzip > dblp_sandcrawler_ingest_requests.json.gz


## [OLD] Manual Commands

Set up a working directory somewhere:

    export DBLP_DIR=/data/dblp

Starting with a complete dblp.xml (and dblp.dtd) dump, do a dry-run transform
and dump release entities in JSON; this takes some time:

    export FATCAT_API_AUTH_TOKEN=[...]
    ./fatcat_import.py dblp-release $DBLP_DIR/dblp.xml --dump-json-mode | pv -l > $DBLP_DIR/dblp_releases.json

Next extract the unique set of dblp identifier prefixes, which will be used as
container identifiers:

    cat $DBLP_DIR/dblp_releases.json | jq ._dblp_prefix -r | grep -v ^null | sort -u > $DBLP_DIR/prefix_list.txt

Then fetch HTML documents from dblp.org for each prefix. Note that currently
only single-level containers will download successfully, and only journals,
conf, and series sections. Books, Tech Reports, etc may be nice to include in
the future.

    mkdir -p journals
    mkdir -p conf
    mkdir -p series

    shuf $DBLP_DIR/prefix_list.txt | pv -l | parallel -j1 wget -nc -q "https://dblp.org/db/{}/index.html" -O {}.html

    # clean up any failed/empty files, then re-run the above parallel/wget command
    find . -empty -type f -delete

Using the python script in this directory, extract metadata from these HTML documents:

    fd html conf/ journals/ series/ | /srv/fatcat/src/extra/dblp/dblp_html_extract.py | pv -l > dblp_container_meta.json

This can be imported into fatcat using the dblp-container importer:

    ./fatcat_import.py dblp-container --issn-map-file /srv/fatcat/datasets/ISSN-to-ISSN-L.txt --dblp-container-map-file $DBLP_DIR/existing_dblp_containers.tsv --dblp-container-map-output $DBLP_DIR/all_dblp_containers.tsv $DBLP_DIR/dblp_container_meta.json
