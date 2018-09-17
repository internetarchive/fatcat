
Run in order:

- ISSN
- ORCID
- Crossref
- Manifest

Lots of trouble with encoding; always `export LC_ALL=C.UTF-8`

## Data Sources

Download the following; uncompress the sqlite file, but **do not** uncompress
the others:

    https://archive.org/download/crossref_doi_dump_201801/crossref-works.2018-01-21.json.xz
    https://archive.org/download/ia_papers_manifest_2018-01-25/index/idents_files_urls.sqlite.gz
    https://archive.org/download/ia_journal_metadata_explore_2018-04-05/journal_extra_metadata.csv
    https://archive.org/download/issn_issnl_mappings/20180216.ISSN-to-ISSN-L.txt
    https://archive.org/download/orcid-dump-2017/public_profiles_API-2.0_2017_10_json.tar.gz

## ISSN

From CSV file:

    export LC_ALL=C.UTF-8
    time ./fatcat_import.py import-issn /srv/datasets/journal_extra_metadata.csv

    real    2m42.148s
    user    0m11.148s
    sys     0m0.336s

Pretty quick, a few minutes.

## ORCID

Directly from compressed tarball; takes about 2 hours in production:

    tar xf /srv/datasets/public_profiles_API-2.0_2017_10_json.tar.gz -O | jq -c . | grep '"person":' | time parallel -j12 --pipe --round-robin ./fatcat_import.py import-orcid -

After tuning database, `jq` CPU seems to be bottleneck, so, from pre-extracted
tarball:

    tar xf /srv/datasets/public_profiles_API-2.0_2017_10_json.tar.gz -O | jq -c . | rg '"person":' > /srv/datasets/public_profiles_1_2_json.all.json
    time parallel --bar --pipepart -j8 -a /srv/datasets/public_profiles_1_2_json.all.json ./fatcat_import.py import-orcid -

Does not work:

    ./fatcat_import.py import-orcid /data/orcid/partial/public_profiles_API-2.0_2017_10_json/3/0000-0001-5115-8623.json

Instead:

    cat /data/orcid/partial/public_profiles_API-2.0_2017_10_json/3/0000-0001-5115-8623.json | jq -c . | ./fatcat_import.py import-orcid -

Or for many files:

    find /data/orcid/partial/public_profiles_API-2.0_2017_10_json/3 -iname '*.json' | parallel --bar jq -c . {} | rg '"person":' | ./fatcat_import.py import-orcid -

### ORCID Performance

for ~9k files:

    (python-B2RYrks8) bnewbold@orithena$ time parallel --pipepart -j4 -a /data/orcid/partial/public_profiles_API-2.0_2017_10_json/all.json ./fatcat_import.py import-orcid -
    real    0m15.294s
    user    0m28.112s
    sys     0m2.408s

    => 636/second

    (python-B2RYrks8) bnewbold@orithena$ time ./fatcat_import.py import-orcid /data/orcid/partial/public_profiles_API-2.0_2017_10_json/all.json
    real    0m47.268s
    user    0m2.616s
    sys     0m0.104s

    => 203/second

For the full batch, on production machine with 12 threads, around 3.8 million records:

    3550.76 user
    190.16 system
    1:40:01 elapsed

    => 644/second

After some simple database tuning:

    2177.86 user
    145.60 system
    56:41.26 elapsed

    => 1117/second

## Crossref

From compressed:

    xzcat /srv/datasets/crossref-works.2018-01-21.json.xz | time parallel -j20 --round-robin --pipe ./fatcat_import.py import-crossref - /srv/datasets/20180216.ISSN-to-ISSN-L.txt

## Manifest 

    time ./fatcat_import.py import-manifest /srv/datasets/idents_files_urls.sqlite

    [...]
    Finished a batch; row 284518671 of 9669646 (2942.39%).  Total inserted: 6606900
    Finished a batch; row 284518771 of 9669646 (2942.39%).  Total inserted: 6606950
    Finished a batch; row 284518845 of 9669646 (2942.39%).  Total inserted: 6607000
    Finished a batch; row 284518923 of 9669646 (2942.39%).  Total inserted: 6607050
    Done! Inserted 6607075

    real    1590m36.626s
    user    339m40.928s
    sys     19m3.576s

Really sped up once not contending with Crossref import, so don't run these two at the same time.

## Matched

    zcat /srv/datasets/2018-08-27-2352.17-matchcrossref.insertable.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py import-matched -

