
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

## Fetch Complete URL

Want to export JSON like:

    file_entity
        [existing file entity]
    full_urls[]
        <short>: <long>
    status: str

Status one of:

- 'success-self': the file already has a fixed URL internally
- 'success-db': lookup URL against sandcrawler-db succeeded, and SHA1 matched
- 'success-cdx': CDX API lookup succeeded, and SHA1 matched
- 'fail-hash': found a CDX record, but wrong hash
- 'fail-not-found': no matching CDX record found
