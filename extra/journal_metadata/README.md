
This folder contains scripts to merge journal metadat from multiple sources and
provide a snapshot for bulk importing into fatcat.

Specific bots will probably be needed to do continous updates; that's out of
scope for this first import.


## Sources

The `./data/fetch.sh` script will fetch mirrored snapshots of all these
datasets.

A few sources of normalization/mappings:

- ISSN-L (from ISSN org)
    - Original:
    - Snapshot: <https://archive.org/download/issn_issnl_mappings/20180216.ISSN-to-ISSN-L.txt>
- ISO 639-1 language codes: https://datahub.io/core/language-codes
- ISO 3166-1 alpha-2 nation codes

In order of precedence (first higher than later):

- NCBI Entrez (Pubmed)
    - Original: <ftp://ftp.ncbi.nlm.nih.gov/pubmed/J_Entrez.txt>
    - Snapshot: <https://archive.org/download/ncbi-entrez-2019/J_Entrez.txt>
- DOAJ
    - Original: <https://doaj.org/csv>
    - Snapshot: <https://archive.org/download/doaj_bulk_metadata_2019/doaj_20190124.csv>
- ROAD
    - Original: <http://road.issn.org/en/contenu/download-road-records>
    - Snapshot: <https://archive.org/download/road-issn-2018/2018-01-24/export-issn.zip>
- SHERPA/ROMEO
    - Original: <http://www.sherpa.ac.uk/downloads/journal-title-issn-urls.php> (requires reg)
    - Mirror: <http://www.moreo.info/?csv=romeo-journals.csv>
    - Snapshot:
- Norwegian Registry
    - Original: <https://dbh.nsd.uib.no/publiseringskanaler/AlltidFerskListe>
    - Snapshot: <https://archive.org/download/norwegian_register_journals>
- Wikidata (TODO: Journal-level not title-level)
    - Original: <http://uri.gbv.de/wikicite/20180903/>
    - Snapshot: <https://archive.org/download/wikicite-biblio-data-20180903>
- KBART reports: LOCKSS, CLOCKSS, Portico
    - Original: (multiple, see README in IA item)
    - Snapshot: <https://archive.org/download/keepers_reports_201901>
- JSTOR
    - Original: <https://support.jstor.org/hc/en-us/articles/115007466248-JSTOR-title-lists>
    - Snapshot: <KBART jstor_all-archive-titles.txt>
- Crossref title list (not DOIs)
    - Original: <https://wwwold.crossref.org/titlelist/titleFile.csv>
    - Snapshot: <https://archive.org/download/crossref_doi_titles>
- IA SIM Microfilm catalog
    - Original: <https://archive.org/download/SerialsOnMicrofilmCollection/MASTER%20TITLE_METADATA_LIST_20171019.xlsx>
- IA homepage crawl attempts

The SHERPA/ROMEO content comes from the list helpfully munged by moreo.info.

General form here is to build a huge python dict in memory, keyed by the
ISSN-L, then write out to disk as JSON. Then the journal-metadata importer
takes a subset of fields and inserts to fatcat. Lastly, the elasticsearch
transformer takes a subset/combination of 

## Python Helpers/Libraries

- ftfy
- pycountry

Debian:
    
    sudo apt install python3-pycountry
    sudo pip3 install ftfy
