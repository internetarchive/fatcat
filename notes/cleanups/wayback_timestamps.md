
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

    zcat files_20211007_moreshortts.json.gz | shuf -n10000 > files_20211007_moreshortts.10k_sample.json
    # 9,958,854

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


## Cleanup Process

Other possible cleanups to run at the same time, which would not require
external requests or other context:

- URL has ://archive.org/ link with rel=repository => rel=archive
- mimetype is bogus => clean mimetype
- bogus file => set some new extra field, like scope=stub or scope=partial (?)

It looks like the rel swap is already implemented in `generic_file_cleanups()`.
From sampling it seems like the mimetype issue is pretty small, so not going to
bite that off now. The "bogus file" issue requires thought, so also skipping.

## Commands

Running with 8x parallelism to not break things; expecting some errors along
the way, may need to add handlers for connection errors etc:

    zcat files_20211007_moreshortts.json.gz \
        | parallel -j8 --linebuffer --round-robin --pipe ./fetch_full_cdx_ts.py \
        | pv -l \
        | gzip \
        > files_20211007_moreshortts.fetched.json.gz

At 300 records/sec, this should take around 9-10 hours to process.
