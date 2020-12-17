
This file describes hacks used to import dblp container metadata.


## Quick Bootstrap Commands

Starting with a complete dblp.xml (and dblp.dtd) dump, do a dry-run transform
and dump release entities in JSON; this takes some time:

    ./fatcat_import.py dblp-release /data/dblp/dblp.xml --dump-json-mode > /data/dblp/dblp_releases.json

Next extract the unique set of dblp identifier prefixes, which will be used as
container identifiers:

    cat /data/dblp/dblp_releases.json | jq ._dblp_prefix | grep -v ^none | sort -u > /data/dblp/prefix_list.txt

Then fetch HTML documents from dblp.org for each prefix:

    mkdir -p journals
    mkdir -p conf
    mkdir -p series

    shuf /data/dblp/prefix_list.txt | pv -l | parallel -j1 wget -nc -q "https://dblp.org/db/{}/index.html" -O {}.html

    # clean up any failed/empty files, then re-run the above parallel/wget command
    find . -empty -type f -delete

Using the python script in this directory, extract metadata from these HTML documents:

    fd .html | ./dblp_html_extract.py | pv -l > dblp_container_meta.json

This can be imported into fatcat using the dblp-container importer:

    ./fatcat_import.py dblp-container --issn-map-file /data/issn/20200323.ISSN-to-ISSN-L.txt --dblp-container-map-file /data/dblp/existing_dblp_containers.tsv --dblp-container-map-output /data/dblp/all_dblp_containers.tsv dblp_container_meta.json
