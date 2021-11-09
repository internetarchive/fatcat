
At some point, using the arabesque importer (from targetted crawling), we
accidentially imported a bunch of files with wayback URLs that have 12-digit
timestamps, instead of the full canonical 14-digit timestamps.


## Prep (2021-11-04)

Download most recent file export:

    wget https://archive.org/download/fatcat_bulk_exports_2021-10-07/file_export.json.gz

Filter to files with problem of interest:

    zcat file_export.json.gz \
        | pv -l \
        | rg 'web.archive.org/web/\d{12}/' \
        | gzip \
        > files_20211007_shortts.json.gz
    # 111M 0:12:35

    zcat files_20211007_shortts.json.gz | wc -l
    # 7,935,009

    zcat files_20211007_shortts.json.gz | shuf -n10000 > files_20211007_shortts.10k_sample.json

Wow, this is a lot more than I thought!

There might also be some other short URL patterns, check for those:

    zcat file_export.json.gz \
        | pv -l \
        | rg 'web.archive.org/web/\d{1,11}/' \
        | gzip \
        > files_20211007_veryshortts.json.gz
    # skipped, mergine with below

    zcat file_export.json.gz \
        | rg 'web.archive.org/web/None/' \
        | pv -l \
        > /dev/null
    # 0.00  0:10:06 [0.00 /s]
    # whew, that pattern has been fixed it seems

    zcat file_export.json.gz | rg '/None/' | pv -l > /dev/null
    # 2.00  0:10:01 [3.33m/s]

    zcat file_export.json.gz \
        | rg 'web.archive.org/web/\d{13}/' \
        | pv -l \
        > /dev/null
    # 0.00  0:10:09 [0.00 /s]

Yes, 4-digit is a popular pattern as well, need to handle those:

    zcat file_export.json.gz \
        | pv -l \
        | rg 'web.archive.org/web/\d{4,12}/' \
        | gzip \
        > files_20211007_moreshortts.json.gz
    # 111M 0:13:22 [ 139k/s]

    zcat files_20211007_moreshortts.json.gz | wc -l
    # 9,958,854

    zcat files_20211007_moreshortts.json.gz | shuf -n10000 > files_20211007_moreshortts.10k_sample.json


## Fetch Complete URL

Want to export JSON like:

    file_entity
        [existing file entity]
    full_urls[]: list of Dicts[str,str]
        <short_url>: <full_url>
    status: str

Status one of:

- 'success-self': the file already has a fixed URL internally
- 'success-db': lookup URL against sandcrawler-db succeeded, and SHA1 matched
- 'success-cdx': CDX API lookup succeeded, and SHA1 matched
- 'fail-not-found': no matching CDX record found

Ran over a sample:

    cat files_20211007_shortts.10k_sample.json | ./fetch_full_cdx_ts.py > sample_out.json

    cat sample_out.json | jq .status | sort | uniq -c
          5 "fail-not-found"
        576 "success-api"
       7212 "success-db"
       2207 "success-self"

    head -n1000  | ./fetch_full_cdx_ts.py > sample_out.json

    zcat files_20211007_veryshortts.json.gz | head -n1000 | ./fetch_full_cdx_ts.py | jq .status | sort | uniq -c
          2 "fail-not-found"
        168 "success-api"
        208 "success-db"
        622 "success-self"

Investigating the "fail-not-found", they look like http/https URL
not-exact-matches. Going to put off handling these for now because it is a
small fraction and more delicate.

Again with the broader set:

    cat files_20211007_moreshortts.10k_sample.json | ./fetch_full_cdx_ts.py > sample_out.json

    cat sample_out.json | jq .status | sort | uniq -c
          9 "fail-not-found"
        781 "success-api"
       6175 "success-db"
       3035 "success-self"

While running a larger batch, got a CDX API error:

    requests.exceptions.HTTPError: 403 Client Error: Forbidden for url: https://web.archive.org/cdx/search/cdx?url=https%3A%2F%2Fwww.psychologytoday.com%2Ffiles%2Fu47%2FHenry_et_al.pdf&from=2017&to=2017&matchType=exact&output=json&limit=20

    org.archive.util.io.RuntimeIOException: org.archive.wayback.exception.AdministrativeAccessControlException: Blocked Site Error

So maybe need to use credentials after all.


## Cleanup Process

Other possible cleanups to run at the same time, which would not require
external requests or other context:

- URL has ://archive.org/ link with rel=repository => rel=archive
- mimetype is bogus => clean mimetype
- bogus file => set some new extra field, like scope=stub or scope=partial (?)

It looks like the rel swap is already implemented in `generic_file_cleanups()`.
From sampling it seems like the mimetype issue is pretty small, so not going to
bite that off now. The "bogus file" issue requires thought, so also skipping.


## Commands (old)

Running with 8x parallelism to not break things; expecting some errors along
the way, may need to add handlers for connection errors etc:

    # OLD SNAPSHOT
    zcat files_20211007_moreshortts.json.gz \
        | parallel -j8 --linebuffer --round-robin --pipe ./fetch_full_cdx_ts.py \
        | pv -l \
        | gzip \
        > files_20211007_moreshortts.fetched.json.gz

At 300 records/sec, this should take around 9-10 hours to process.



## Prep Again (2021-11-09)

After fixing "sort" issue and re-dumping file entities (2021-11-05 snapshot).

Filter again:

    # note: in the future use pigz instead of gzip here
    zcat file_export.json.gz \
        | pv -l \
        | rg 'web.archive.org/web/\d{4,12}/' \
        | gzip \
        > files_20211105_moreshortts.json.gz
    # 112M 0:13:27 [ 138k/s]

    zcat files_20211105_moreshortts.json.gz | wc -l
    # 9,958,854
    # good, exact same number as previous snapshot

    zcat files_20211105_moreshortts.json.gz | shuf -n10000 > files_20211105_moreshortts.10k_sample.json
    # done

    cat files_20211105_moreshortts.10k_sample.json \
        | ./fetch_full_cdx_ts.py \
        | pv -l \
        > files_20211105_moreshortts.10k_sample.fetched.json
    # 10.0k 0:03:36 [46.3 /s]

    cat files_20211105_moreshortts.10k_sample.fetched.json | jq .status | sort | uniq -c
         13 "fail-not-found"
        774 "success-api"
       6193 "success-db"
       3020 "success-self"

After tweaking `success-self` logic:

         13 "fail-not-found"
        859 "success-api"
       6229 "success-db"
       2899 "success-self"


## Testing in QA

Copied `sample_out.json` to fatcat QA instance and renamed as `files_20211007_moreshortts.10k_sample.fetched.json`

    # OLD ATTEMPT
    export FATCAT_API_AUTH_TOKEN=[...]
    head -n10 /srv/fatcat/datasets/files_20211007_moreshortts.10k_sample.fetched.json \
        | python -m fatcat_tools.cleanups.file_short_wayback_ts -

Ran in to issues, iterated above.

Trying again with updated script and sample file:

    export FATCAT_AUTH_WORKER_CLEANUP=[...]

    head -n10 /srv/fatcat/datasets/files_20211105_moreshortts.10k_sample.fetched.json \
        | python -m fatcat_tools.cleanups.file_short_wayback_ts -
    # Counter({'total': 10, 'update': 10, 'skip': 0, 'insert': 0, 'exists': 0})

Manually inspected and these look good. Trying some repeats and larger batched:

    head -n10 /srv/fatcat/datasets/files_20211105_moreshortts.10k_sample.fetched.json \
        | python -m fatcat_tools.cleanups.file_short_wayback_ts -
    # Counter({'total': 10, 'skip-revision-changed': 10, 'skip': 0, 'insert': 0, 'update': 0, 'exists': 0})

    head -n1000 /srv/fatcat/datasets/files_20211105_moreshortts.10k_sample.fetched.json \
        | python -m fatcat_tools.cleanups.file_short_wayback_ts -

    [...]
    bad replacement URL: partial_ts=201807271139 original=http://www.scielo.br/pdf/qn/v20n1/4918.pdf fix_url=https://web.archive.org/web/20170819080342/http://www.scielo.br/pdf/qn/v20n1/4918.pdf
    bad replacement URL: partial_ts=201904270207 original=https://www.matec-conferences.org/articles/matecconf/pdf/2018/62/matecconf_iccoee2018_03008.pdf fix_url=https://web.archive.org/web/20190501060839/https://www.matec-conferences.org/articles/matecconf/pdf/2018/62/matecconf_iccoee2018_03008.pdf
    bad replacement URL: partial_ts=201905011445 original=https://cdn.intechopen.com/pdfs/5886.pdf fix_url=https://web.archive.org/web/20190502203832/https://cdn.intechopen.com/pdfs/5886.pdf
    [...]

    # Counter({'total': 1000, 'update': 969, 'skip': 19, 'skip-bad-replacement': 18, 'skip-revision-changed': 10, 'skip-bad-wayback-timestamp': 2, 'skip-status': 1, 'insert': 0, 'exists': 0})


It looks like these "bad replacement URLs" are due to timestamp mismatches. Eg, the partial timestamp is not part of the final timestamp.

Tweaked fetch script and re-ran:

    # Counter({'total': 1000, 'skip-revision-changed': 979, 'update': 18, 'skip-bad-wayback-timestamp': 2, 'skip': 1, 'skip-status': 1, 'insert': 0, 'exists': 0})

Cool. Sort of curious what the deal is with those `skip-bad-wayback-timestamp`.

Run the rest through:

    cat /srv/fatcat/datasets/files_20211105_moreshortts.10k_sample.fetched.json \
        | python -m fatcat_tools.cleanups.file_short_wayback_ts -
    # Counter({'total': 10000, 'update': 8976, 'skip-revision-changed': 997, 'skip-bad-wayback-timestamp': 14, 'skip': 13, 'skip-status': 13, 'insert': 0, 'exists': 0})

Should tweak batch size to 100 (vs. 50).

How to parallelize import:

    # from within pipenv
    cat /srv/fatcat/datasets/files_20211105_moreshortts.10k_sample.fetched.json \
        | parallel -j8 --linebuffer --round-robin --pipe python -m fatcat_tools.cleanups.file_short_wayback_ts -


## Full Batch Commands

Running in bulk again:

    zcat files_20211105_moreshortts.json.gz \
        | parallel -j8 --linebuffer --round-robin --pipe ./fetch_full_cdx_ts.py \
        | pv -l \
        | gzip \
        > files_20211105_moreshortts.fetched.json.gz

