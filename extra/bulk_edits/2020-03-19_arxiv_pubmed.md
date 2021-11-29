
On 2020-03-20, automated daily harvesting and importing of arxiv and pubmed
metadata started. In the case of pubmed, updates are enabled, so that recently
created DOI releases get updated with PMID and extra metadata.

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

First, mirror update files from FTP, e.g. via lftp:

    mkdir -p /srv/fatcat/datasets/pubmed_updates
    lftp -e 'mirror -c /pubmed/updatefiles /srv/fatcat/datasets/pubmed_updates; bye' ftp://ftp.ncbi.nlm.nih.gov

Inspect completed dates from kafka:

    kafkacat -b $KAFKA_BROKER -t fatcat-prod.ftp-pubmed-state -C

Show dates and corresponding files:

    find /srv/fatcat/datasets/pubmed_updates -name "*html" | xargs cat | grep "Created" | sort

For this bulk import, we used files pubmed20n1016.xml.gz (2019-12-16) up to pubmed20n1110.xml.gz (2020-03-06).

To import the corresponding files, run:

    printf "%s\n" /srv/fatcat/datasets/pubmed_updates/pubmed20n{1016..1110}.xml.gz | shuf | \
        parallel -j16 'gunzip -c {} | ./fatcat_import.py pubmed --do-updates - /srv/fatcat/datasets/ISSN-to-ISSN-L.txt'

Import took 254 min, there were 1715427 PubmedArticle docs in these update files.
