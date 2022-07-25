#!/usr/bin/env bash

# run this as 'fatcat' user on a production machine

rm -f dblp.dtd
rm -f dblp.xml.gz
rm -f dblp.xml
rm -f dblp_releases_partial.json
rm -f prefix_list.txt
rm -f dblp_container_meta.json
rm -f existing_dblp_containers.tsv
rm -f all_dblp_containers.tsv

rm -rf ./journals/
rm -rf ./conf/
rm -rf ./series/

