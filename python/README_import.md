
Run in order:

- ISSN
- ORCID
- Crossref
- Manifest

Lots of trouble with encoding; always `export LC_ALL=C.UTF-8`

Start off with:

    sudo su webcrawl
    cd /srv/fatcat/src/python
    export LC_ALL=C.UTF-8
    pipenv shell
    export LC_ALL=C.UTF-8

## Data Sources

Download the following; uncompress the sqlite file, but **do not** uncompress
the others:

    cd /srv/fatcat/datasets
    wget https://archive.org/download/crossref_doi_dump_201809/crossref-works.2018-09-05.json.xz
    wget https://archive.org/download/ia_papers_manifest_2018-01-25/index/idents_files_urls.sqlite.gz
    wget https://archive.org/download/ia_journal_metadata_explore_2018-04-05/journal_extra_metadata.csv
    wget https://archive.org/download/issn_issnl_mappings/20180216.ISSN-to-ISSN-L.txt
    wget https://archive.org/download/orcid-dump-2017/public_profiles_1_2_json.all.json.gz
    wget https://archive.org/download/ia_journal_pid_map_munge_20180908/release_ids.ia_munge_20180908.sqlite3.gz
    wget https://archive.org/download/ia_test_paper_matches/2018-08-27-2352.17-matchcrossref.insertable.json.gz
    wget https://archive.org/download/ia_papers_manifest_2018-01-25_matched/ia_papers_manifest_2018-01-25.matched.json.gz

    gunzip public_profiles_1_2_json.all.json.gz

## Journal Metadata

From JSON file:

    # See "start off with" command above
    time ./fatcat_import.py journal-metadata /srv/fatcat/datasets/journal_metadata.json

Usually a couple minutes at most on fast production machine.

## ORCID

Usually tens of minutes on fast production machine.

    time parallel --bar --pipepart -j8 -a /srv/fatcat/datasets/public_profiles_1_2_json.all.json ./fatcat_import.py orcid -

## Crossref

Usually 24 hours or so on fast production machine.

    time xzcat /srv/fatcat/datasets/crossref-works.2018-09-05.json.xz | time parallel -j20 --round-robin --pipe ./fatcat_import.py crossref - /srv/fatcat/datasets/ISSN-to-ISSN-L.txt --extid-map-file /srv/fatcat/datasets/release_ids.ia_munge_20180908.sqlite3

## JALC

First import a random subset single threaded to create (most) containers. On a
fast machine, this takes a couple minutes.

    time ./fatcat_import.py jalc /srv/fatcat/datasets/JALC-LOD-20180907.sample10k.rdf /srv/fatcat/datasets/ISSN-to-ISSN-L.txt --extid-map-file /srv/fatcat/datasets/release_ids.ia_munge_20180908.sqlite3

Then, in parallel:

    zcat /srv/fatcat/datasets/JALC-LOD-20180907.gz | pv -l | time parallel -j20 --round-robin --pipe ./fatcat_import.py jalc - /srv/fatcat/datasets/ISSN-to-ISSN-L.txt --extid-map-file /srv/fatcat/datasets/release_ids.ia_munge_20180908.sqlite3

## JSTOR

Looks like:

    fd . /data/jstor/metadata/ | time parallel -j20 --round-robin --pipe ./fatcat_import.py jstor - /data/issn/ISSN-to-ISSN-L.txt

## arXiv

Single file:

    ./fatcat_import.py arxiv /srv/fatcat/datasets/arxiv_raw_oai_snapshot_2019-05-22/2007-12-31-00000001.xml

Bulk (one file per process):

    fd '.xml$' /srv/fatcat/datasets/arxiv_raw_oai_snapshot_2019-05-22/ | parallel -j15 ./fatcat_import.py arxiv {}

## PubMed

Run single:

    time ./fatcat_import.py pubmed /srv/fatcat/datasets/pubmed_medline_baseline_2019/pubmed19n0400.xml /srv/fatcat/datasets/ISSN-to-ISSN-L.txt

    real    13m21.756s
    user    9m10.720s
    sys     0m14.100s

Bulk:

    fd '.xml$' /srv/fatcat/datasets/pubmed_medline_baseline_2019 | time parallel -j16 ./fatcat_import.py pubmed {} /srv/fatcat/datasets/ISSN-to-ISSN-L.txt

## Matched

These each take 2-4 hours:

    # No file update for the first import...
    time zcat /srv/fatcat/datasets/ia_papers_manifest_2018-01-25.matched.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py matched --no-file-updates -

    # ... but do on the second
    zcat /srv/fatcat/datasets/2018-08-27-2352.17-matchcrossref.insertable.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py matched -

    # GROBID extracted (release+file)
    time zcat /srv/fatcat/datasets/2018-09-23-0405.30-dumpgrobidmetainsertable.longtail_join.filtered.tsv.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py grobid-metadata -

## Arabesque Matches

Prep JSON files from sqlite (for parallel import):

    ~/arabesque/arabesque.py dump_json s2_doi.sqlite --only-identifier-hits | pv -l | gzip > s2_doi.json.gz

Run import in parallel:

    export FATCAT_AUTH_WORKER_CRAWL=...
    zcat /srv/fatcat/datasets/s2_doi.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py arabesque --json-file - --extid-type doi --crawl-id DIRECT-OA-CRAWL-2019 --no-require-grobid

## Other Matched

    export FATCAT_EDITGROUP_DESCRIPTION="File/DOI matching to user-uploaded pre-1923 and pre-1909 paper corpus on archive.org"
    export FATCAT_API_AUTH_TOKEN=... (FATCAT_AUTH_WORKER_ARCHIVE_ORG)

    zcat /srv/fatcat/datasets/crossref-pre-1923-scholarly-works.matched.json.gz | time parallel -j12 --round-robin --pipe ./fatcat_import.py matched - --default-mime 'application/pdf'

## DOAJ

Takes a few hours.

    export FATCAT_API_AUTH_TOKEN=... (FATCAT_AUTH_WORKER_DOAJ)

    zcat /srv/fatcat/datasets/doaj_article_data_2020-11-13_all.json.gz | pv -l | parallel -j12 --round-robin --pipe ./fatcat_import.py doaj-article --issn-map-file /srv/fatcat/datasets/ISSN-to-ISSN-L.txt -

## dblp

See `extra/dblp/README.md` for notes about first importing container metadata
and getting a TSV mapping flie to help with import. This is needed because
there is not (yet) a lookup mechanism for `dblp_prefix` as an identifier of
container entities.

    export FATCAT_AUTH_WORKER_DBLP=...
    ./fatcat_import.py dblp-release --dblp-container-map-file /data/dblp/all_dblp_containers.tsv /data/dblp/dblp.xml

