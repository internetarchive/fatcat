
When doing a bulk import, could try:


Probably want to set:

    --cache=.25 --max-sql-memory=.25

Check FD limits:

    cat /proc/sys/fs/file-max
    ulimit -a

Set with:

    echo 150000 > /proc/sys/fs/file-max

TODO:
- set `num_replicas=1` when testing
- close browser to free up RAM
- disable raft fsync
    https://github.com/cockroachdb/cockroach/issues/19784
- increase import batch size (10x current)

## Log of Tests

build: CCL v2.0.5 @ 2018/08/13 17:59:42 (go1.10)

COCKROACH branch e386022ba051f46ff70c28b90d9a2fe1f856c57f

cockroach start --insecure --store=fatcat-dev --host=localhost

    time ./fatcat_import.py import-issn /home/bnewbold/code/oa-journals-analysis/upload-2018-04-05/journal_extra_metadata.csv

    real    3m20.781s
    user    0m6.900s
    sys     0m0.336s

    cat /data/crossref/crossref-works.2018-01-21.badsample_100k.json | time parallel -j4 --round-robin --pipe ./fatcat_import.py import-crossref - /data/issn/20180216.ISSN-to-ISSN-L.txt

    => gave up after ~30 minutes with only ~40% done

With cockroach, container lookups seem to be particularly slow, like 1000ms+,
though not reflected in cockroach-reported stats . Very high CPU usage by
cockroach.

Cockroach is self-reporting P50 latency: 0.3 ms, P99 latency: 104.9 ms

Seeing lots of Queue Processing Failures, of the "replication" type. Probably because there are no other nodes.

CHANGES:
- closed browser, restarted postgres, restarted fatcatd to free up RAM
- cockroach start --insecure --store=fatcat-dev --host=localhost --cache=.25 --max-sql-memory=.25
- cockroach zone set --insecure .default -f -
    num_replicas: 1
    ^D
- SET CLUSTER SETTING kv.raft_log.synchronize=false;
- batch size from default (50) to 500

    time ./fatcat_import.py import-issn --batch-size 500 /home/bnewbold/code/oa-journals-analysis/upload-2018-04-05/journal_extra_metadata.csv

    real    1m10.531s
    user    0m5.204s
    sys     0m0.128s

Have 12 SQL connections

    cat /data/crossref/crossref-works.2018-01-21.badsample_100k.json | time parallel -j4 --round-robin --pipe ./fatcat_import.py import-crossref --batch-size 500 - /data/issn/20180216.ISSN-to-ISSN-L.txt

    GETs are still very slow end-to-end (~1sec)
    during this time, though, cockroach is quoting very low latency
    => halted

It's the join performance (on lookups) that is killing things for release rev import:

    select * from container_rev where issnl = '0010-7824';
    Time: 2.16272ms

    select * from container_rev inner join container_ident on container_rev.id = container_ident.rev_id where issnl = '0010-7824';
    Time: 147.44647ms

CHANGED: CCL v2.1.0-beta.20180904 @ 2018/09/04 03:51:03 (go1.10.3)

    time ./fatcat_import.py import-issn --batch-size 500 /home/bnewbold/code/oa-journals-analysis/upload-2018-04-05/journal_extra_metadata.csv

    real    0m59.903s
    user    0m4.716s
    sys     0m0.076s

    cat /data/crossref/crossref-works.2018-01-21.badsample_100k.json | time parallel -j4 --round-robin --pipe ./fatcat_import.py import-crossref --batch-size 500 - /data/issn/20180216.ISSN-to-ISSN-L.txt

    => still extremely slow API latency, though cockroach is reporting very
       fast (P50 latency 1.0 ms, P99 latency 4.7 ms)
    => maybe a timeout or something going on?
    => single-level joins from CLI are ~200 ms

