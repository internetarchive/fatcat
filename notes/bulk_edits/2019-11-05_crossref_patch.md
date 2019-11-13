
Goal is to make sure we have imported all in-scope crossref DOI objects. There
were a few months gap between the snapshot used as initial bootstrap and the
start of continuous ingest; any DOIs registered during that gap and not updated
since are not in fatcat. Expectation is that this will be a relatively small
import.

## QA Run

Started Thu 31 Oct 2019 08:07:20 PM PDT

    export FATCAT_AUTH_WORKER_CROSSREF="..."
	time xzcat /srv/fatcat/datasets/crossref-works.2019-09-09.json.xz | time parallel -j20 --round-robin --pipe ./fatcat_import.py crossref - /srv/fatcat/datasets/20181203.ISSN-to-ISSN-L.txt --extid-map-file /srv/fatcat/datasets/release_ids.ia_munge_20180908.sqlite3

    # postgresql DB at start: fresh 2019-10 dump imported, 357 GB
    # over 15k TPS against postgres

    20x theads of:
    Counter({'total': 5397349, 'exists': 4961058, 'skip': 360156, 'insert': 76135, 'inserted.container': 113, 'update': 0})

    real    1173m52.497s => 20hr
    user    13058m24.460s
    sys     319m27.716s

    1.5 million new releases
    7.2 million skips (total)

Ran again with null subtitle fix and granular stats:

    20x threads of:
    Counter({'total': 5368366, 'exists': 5122104, 'skip': 244072, 'skip-blank-title': 38399, 'skip-release-type': 5296, 'insert': 2190, 'skip-huge-contribs': 70, 'skip-huge-refs': 7, 'update': 0})

    43k additional insets (still about 1.5m total)
    of 4.8 million skipped (why not closer to 7.2 million?), most seem to be blank title

## Production Run

Git: 44c23290c72ec67db38f1e1d40b76ba795b40d9d

started around Tue 05 Nov 2019 02:51:19 PM PST

    export FATCAT_AUTH_WORKER_CROSSREF="..."
	time xzcat /srv/fatcat/datasets/crossref-works.2019-09-09.json.xz | time parallel -j20 --round-robin --pipe ./fatcat_import.py crossref - /srv/fatcat/datasets/20190730.ISSN-to-ISSN-L.txt --extid-map-file /srv/fatcat/datasets/release_ids.ia_munge_20180908.sqlite3

    # postgresql DB at start: 399.03G

    # 20x of:
    Counter({'total': 5347938, 'exists': 5023305, 'skip': 251747, 'skip-blank-title': 247969, 'insert': 72886, 'skip-release-type': 3686, 'inserted.container': 103, 'skip-huge-contribs': 88, 'skip-huge-refs': 4, 'update': 0})
    # 1.45m new releases
    # 2k more new containers
    # 4.96m blank titles

    real    1139m42.231s
    user    13307m10.124s
    sys     355m18.904s

    # postgresql DB: 402.76G

