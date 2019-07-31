#!/bin/bash

set -eu

#wget -c https://archive.org/download/road-issn-2018/2018-01-24/export-issn.zip -O road-2018-01-24-export-issn.zip
#unzip -n road-2018-01-24-export-issn.zip
wget -c https://archive.org/download/road-issn-2018/road-2018-01-24.tsv

wget -c https://archive.org/download/doaj_bulk_metadata_2019/doaj_20190124.csv

#wget -c https://archive.org/download/issn_issnl_mappings/20190129.ISSN-to-ISSN-L.txt
wget -c https://archive.org/download/issn_issnl_mappings/20190730.ISSN-to-ISSN-L.txt

wget -c https://archive.org/download/crossref_doi_titles/doi_titles_file_2019-01-24.csv

#wget -c https://archive.org/download/ncbi-entrez-2019/J_Entrez.txt -O ncbi-entrez-2019.txt

wget -c https://archive.org/download/moreo.info-2018-12-20/romeo-journals.csv
wget -c https://archive.org/download/moreo.info-2018-12-20/romeo-policies.csv
wget -c https://archive.org/download/moreo.info-2018-12-20/entrez-journals.csv

wget -c https://archive.org/download/doaj_bulk_metadata_2019/doaj_20190124.csv

wget -c https://archive.org/download/keepers_reports_201901/jstor_all-archive-titles.txt
wget -c https://archive.org/download/keepers_reports_201901/JSTOR_Global_AllCurrentJournalTitles_2019-01-07.txt
wget -c https://archive.org/download/keepers_reports_201901/JSTOR_Global_EarlyJournalContent_2017-06-08.txt
wget -c https://archive.org/download/keepers_reports_201901/kbart_CLOCKSS.txt
wget -c https://archive.org/download/keepers_reports_201901/kbart_LOCKSS.txt
wget -c https://archive.org/download/keepers_reports_201901/Portico_Holding_KBart.txt

wget -c https://archive.org/download/ia_journal_metadata_explore_2018-04-05/journal_homepage_results.partial.tsv

#wget -c https://archive.org/download/SerialsOnMicrofilmCollection/MASTER%20TITLE_METADATA_LIST_20171019.xlsx
wget -c https://archive.org/download/SerialsOnMicrofilmCollection/MASTER%20TITLE_METADATA_LIST_20171019.converted.csv

wget -c https://archive.org/download/norwegian_register_journals/2018-03-02%20Norwegian%20Register%20for%20Scientific%20Journals%20and%20Series.csv

#wget -c https://archive.org/download/open_academic_graph_2019/mag_venues.zip
#unzip mag_venues.zip

wget -c https://archive.org/download/szczepanski-oa-journal-list-2018/Jan-Szczepanski-Open-Access-Journals-2018_0.fixed.json

wget -c https://archive.org/download/ezb_snapshot_2019-07-11/ezb_metadata.json
wget -c https://archive.org/download/ISSN-GOLD-OA-3/ISSN_Gold-OA_3.0.csv
wget -c https://archive.org/download/openapc-dataset/apc_de.2019-07-30.csv
wget -c https://archive.org/download/wikidata-journal-metadata/wikidata_journals_sparql.2019-07-30.tsv

