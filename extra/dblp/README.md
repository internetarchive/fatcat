
This file describes hacks used to import dblp container metadata.

As of December 2020 this is part of the dblp release metadata import pipeline:
we must have conference and other non-ISSN containers created before running
the release import. dblp does not publish container-level metadata in a
structured format (eg, in their dumps), so scraping the HTML is unfortunately
necessary.


## Quick Bootstrap Commands

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
