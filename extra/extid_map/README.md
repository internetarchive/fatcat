
Process for generating a sqlite3 database file mapping between:

- DOI
- Wikidata QID
- CORE database id
- PubMed Central id
- PubMed id

This fast/indexed local database file, which is about a gigabyte compressed, is
useful for optimizing some fatcat 'release' entity imports and operations.

There is an example database at <https://archive.org/details/ia_journal_pid_map_munge_20180908>

## Data Sources

EuropePMC mapping (more works than the USA PubMedCentral mapping)
- <ftp://ftp.ebi.ac.uk/pub/databases/pmc/DOI/PMID_PMCID_DOI.csv.gz>
- <https://archive.org/details/europepmc-id-map-2018-08-31>

Wikicite data snapshot
- <https://archive.org/details/wikicite-biblio-data-20180903>

CORE dataset
- <https://core.ac.uk/services#api>
- <https://archive.org/download/core_oa_metadata_20180301>

## Wikidata Extract

Early query:

    zcat wikidata-20180806-publications.ndjson.gz.crdownload | rg '"P356"' | jq '{qid: .id, doi: .claims.P356[0]}' -c | pv -l | head

Polished:

    zcat wikidata-20180903-publications.ndjson.gz | rg '"P356"' | jq '[(.claims.P356[0] | ascii_downcase), .id] | @tsv' -cr | pv -l > doi_wikidata.20180903.tsv

    # 13.6M 0:26:13 [8.65k/s]

Hrm, got this but seemed to continue:

    jq: error (at <stdin>:455616): explode input must be a string

## CORE Extract

    xzcat core_json/*.json.xz | jq -rc 'select(.doi != null) | [(.doi | ascii_downcase), .coreId] | @tsv' | pv -l > doi_core.20180301.tsv

    # bnewbold@bnewbold-dev$ wc -l doi_core.20180301.tsv
    # 28210779 doi_core.20180301.tsv

## PMCID/PMID

In a nice CSV format, no extract needed.

    bnewbold@bnewbold-dev$ zcat PMID_PMCID_DOI.csv.gz | rg doi.org | wc -l
    19441168

## sqlite schema

    CREATE TABLE ids (doi text not null, core int, pmid int, pmcid text, wikidata text);
    CREATE UNIQUE INDEX ids_doi on ids (doi);

Run CORE import first (largest mapping, thus fewest exception/update paths),
then pubmed, then wikidata.

## Run import

    cat doi_core.20180301.tsv | ./load_core.py release_ids.db
    => read 28210000, wrote 28210000
    => overnight? but seemed to run fast

    zcat doi_wikidata.20180903.tsv.gz | ./load_wikidata.py release_ids.db
    => uhoh, seems very slow. lots of IOWAIT. switching to host with SSD
    => wow, like 5-10x faster at least. sigh.
    => ran in a few hours

    zcat PMID_PMCID_DOI.csv.gz | ./load_pmc.py release_ids.db
    => read 29692000, wrote 19441136


    sqlite> select count(*) from ids;
    24831337

    # only 2.3 million works have all IDs
    sqlite> select count(*) from ids where wikidata not null and core not null and pmid not null;
    2314700

    # almost (but not all) PubMedCentral items are PubMed items
    sqlite> select count(*) from ids where pmid not null;
    19328761
    sqlite> select count(*) from ids where pmcid not null;
    3739951
    sqlite> select count(*) from ids where pmcid not null and pmid not null;
    3682486

    # wikidata DOIs seem to mostly come from pmid mapping
    sqlite> select count(*) from ids where wikidata not null;
    13608497
    sqlite> select count(*) from ids where pmid not null and wikidata not null;
    13179156

    # core IDs are more independent (?)
    sqlite> select count(*) from ids where core not null;
    7903910
    sqlite> select count(*) from ids where core not null and wikidata not null;
    2372369
    sqlite> select count(*) from ids where core not null and pmid not null;
    2783344

