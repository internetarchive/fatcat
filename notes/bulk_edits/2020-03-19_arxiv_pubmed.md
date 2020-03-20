
On 2020-03-20, automated daily harvesting and importing of arxiv and pubmed
medata started. In the case of pubmed, updates are enabled, so that recently
created DOI releases get updated with PMID and extra metdata.

We also want to do last backfills of metadata since the last import up through
the first day updated by the continuous harvester.


## arxiv

The previous date span was 2019-05-22 through 2019-12-20. This time we should
do 2019-12-20 through today.

First do metha update from last harvest through today, and grab the new daily files:

    metha-sync -format arXivRaw http://export.arxiv.org/oai2

    mkdir arxiv_20191220_20200319
    cp 2019-12-2* 2019-12-3* 2020-* arxiv_20191220_20200319/
    tar cf arxiv_20191220_20200319.tar arxiv_20191220_20200319/
    gzip arxiv_20191220_20200319.tar

Then copy to fatcat server and run import:

    export FATCAT_AUTH_WORKER_ARXIV=...

    ./fatcat_import.py --batch-size 100 arxiv /srv/fatcat/datasets/arxiv_20191220_20200319/2019-12-31-00000000.xml
    => Counter({'exists': 1824, 'total': 1001, 'insert': 579, 'skip': 1, 'update': 0})

    fd .xml /srv/fatcat/datasets/arxiv_20191220_20200319/ | parallel -j15 ./fatcat_import.py --batch-size 100 arxiv {}

Ran fairly quickly only some ~80-90k entities to process.

## PubMed

TODO: martin will import daily update files from the 2020 baseline through XYZ date.
