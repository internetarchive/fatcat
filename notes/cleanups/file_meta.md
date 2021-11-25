
Over 500k file entities still lack complete metadata. For example, SHA-256
checksums and verified mimetypes.

Presumably these also lack GROBID processing. It seems that most or all of
these are simply wayback captures with no CDX metadata in sandcrawler-db, so
they didn't get update in prior cleanups.

Current plan, re-using existing tools and processes, is to:

1. create stub ingest requests containing file idents
2. process them "locally" on a large VM, in 'bulk' mode; writing output to stdout but using regular grobid and pdfextract "sinks" to Kafka
3. transform ingest results to a form for existing `file_meta` importer
4. run imports

The `file_meta` importer requires just the `file_meta` dict from sandcrawler.

## Prep

    zcat file_hashes.tsv.gz | pv -l | rg '\t\t' | wc -l
    # 521,553

    zcat file_export.json.gz \
        | rg -v '"sha256":' \
        | pv -l \
        | pigz \
        > files_missing_sha256.json.gz
    # 521k 0:10:21 [ 839 /s]

Want ingest requests with:

    base_url: str
    ingest_type: "pdf"
    link_source: "fatcat"
    link_source_id: file ident (with "file_" prefix)
    ingest_request_source: "file-backfill"
    ext_ids:
        sha1: str

Use `file2ingestrequest.py` helper:

    zcat files_missing_sha256.json.gz \
        | ./file2ingestrequest.py \
        | pv -l \
        | pigz \
        > files_missing_sha256.ingest_request.json.gz
    # 519k 0:00:19 [26.5k/s]

So about 2k filtered out, will investigate later.

    zcat files_missing_sha256.ingest_request.json.gz \
        | shuf -n1000 \
        > files_missing_sha256.ingest_request.sample.json

    head -n100 files_missing_sha256.ingest_request.sample.json | ./ingest_tool.py requests --no-spn2 - > sample_results.json
         4 "no-capture"
         1 "no-pdf-link"
        95 "success"

Seems like this is going to be a good start, but will need iteration.

Dev testing:

    head files_missing_sha256.ingest_request.sample.json \
        | ./ingest_tool.py file-requests-backfill - --kafka-env qa --kafka-hosts wbgrp-svc263.us.archive.org:9092,wbgrp-svc284.us.archive.org:9092,wbgrp-svc285.us.archive.org:9092 \
        > out_sample.json


## Commands

Production warm-up:

    cat /srv/sandcrawler/tasks/files_missing_sha256.ingest_request.sample.json \
        | ./ingest_tool.py file-requests-backfill - --kafka-env prod --kafka-hosts wbgrp-svc263.us.archive.org:9092,wbgrp-svc284.us.archive.org:9092,wbgrp-svc285.us.archive.org:9092 --grobid-host http://localhost:8070 \
        > /srv/sandcrawler/tasks/files_missing_sha256.ingest_results.sample.json

Production parallel run:

    zcat /srv/sandcrawler/tasks/files_missing_sha256.ingest_request.json \
        | parallel -j24 --linebuffer --round-robin --pipe ./ingest_tool.py file-requests-backfill - --kafka-env qa --kafka-hosts wbgrp-svc263.us.archive.org:9092,wbgrp-svc284.us.archive.org:9092,wbgrp-svc285.us.archive.org:9092 --grobid-host http://localhost:8070 \
        > /srv/sandcrawler/tasks/files_missing_sha256.ingest_results.json

Filter and select file meta for import:

    head files_missing_sha256.ingest_results.json \
        | rg '"sha256hex"' \
        | jq 'select(.request.ext_ids.sha1 == .file_meta.sha1hex) | .file_meta' -c \
        > files_missing_sha256.file_meta.json
    # Worker: Counter({'total': 20925, 'success': 20003, 'no-capture': 545, 'link-loop': 115, 'wrong-mimetype': 104, 'redirect-loop': 46, 'wayback-error': 25, 'null-body': 20, 'no-pdf-link': 18, 'skip-url-blocklist': 17, 'terminal-bad-status': 16, 'cdx-error': 9, 'wayback-content-error': 4, 'blocked-cookie': 3})
    # [etc]


Had some GROBID issues, so are not going to be able to get everything in first
pass. Merge our partial results, as just `file_meta`:

    cat files_missing_sha256.ingest_results.batch1.json files_missing_sha256.ingest_results.json \
        | jq .file_meta -c \
        | rg '"sha256hex"' \
        | pv -l \
        > files_missing_sha256.file_meta.json
    # 386k 0:00:41 [9.34k/s]

A bunch of these will need to be re-run once GROBID is in a healthier place.

Check that we don't have (many) dupes:

    cat files_missing_sha256.file_meta.json \
        | jq .sha1hex -r \
        | sort \
        | uniq -D \
        | wc -l
    # 86520

Huh, seems like a weirdly large number. Maybe related to re-crawling? Will need
to dedupe by sha1hex.

Check how many dupes in original:

    zcat files_missing_sha256.ingest_request.json.gz | jq .ext_ids.sha1 -r | sort | uniq -D | wc -l

That lines up with dupes expected before SHA-1 de-dupe run.

    cat files_missing_sha256.file_meta.json \
        | sort -u -S 4G \
        | pv -l \
        > files_missing_sha256.file_meta.uniq.json

    cat files_missing_sha256.file_meta.uniq.json \
        | jq .sha1hex -r \
        | sort \
        | uniq -D \
        | wc -l
    # 0

Have seen a lot of errors like:

    %4|1637808915.562|TERMINATE|rdkafka#producer-1| [thrd:app]: Producer terminating with 1 message (650 bytes) still in queue or transit: use flush() to wait for outstanding message delivery

TODO: add manual `finish()` calls on sinks in tool `run` function

## QA Testing

    export FATCAT_API_AUTH_TOKEN... # sandcrawler-bot

    cat /srv/fatcat/datasets/files_missing_sha256.file_meta.uniq.sample.json \
        | ./fatcat_import.py --editgroup-description-override 'backfill of full file-level metadata for early-imported papers' file-meta -
    # Counter({'total': 1000, 'update': 503, 'skip-existing-complete': 403, 'skip-no-match': 94, 'skip': 0, 'insert': 0, 'exists': 0})

    head -n1000 /srv/fatcat/datasets/files_missing_sha256.file_meta.uniq.json \
        | parallel -j8 --round-robin --pipe -q ./fatcat_import.py --editgroup-description-override 'backfill of full file-level metadata for early-imported papers' file-meta -
    # Counter({'total': 1000, 'update': 481, 'skip-existing-complete': 415, 'skip-no-match': 104, 'skip': 0, 'insert': 0, 'exists': 0})

