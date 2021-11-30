
There are a bunch of PDF captures in wayback, crawled by common crawl, which
have been truncated at 128 KBytes (130775 bytes).

Most of these were presumably imported before GROBID success was required for
fatcat ingest.

Fixup should wait until `fatcat_meta` cleanup has completed.

## Fatcat Files

Using an old snapshot, found 553 hits in elasticsearch. Index has not been
updated in a long time. Filtering to only those with a wayback URL, found just
415.

Here are some broken examples:

    https://fatcat.wiki/file/2e64rh3rc5gbvjzy2zux3qo36y
    sha1:44b54e9d272620f4e0641cadc1aa496fced5a950
    CDX warc_path:1224043144048_15-c/1224043303833_53.arc.gz
    CDX dt:20081006225231

    https://fatcat.wiki/file/n7ydoj6b6rhdhe4sc24lb7licq
    sha1:5916db9e338f2d6845af47a3e19b82cc59079249
    CDX warc_path:1216931712771_5-c/1216932408179_9.arc.gz
    CDX dt:20080708202110

    https://fatcat.wiki/file/iazux5lur5bfveraq6m7iewf5m
    645ff8b602a0ea94fd28ce93bfea4ff2f65aa124
    CDX warc_path:1216743864162_13-c/1216744138445_15.arc.gz
    CDX dt:20080706111123

This example seems fine (not broken):

    https://fatcat.wiki/file/lww5omk3grejhb2mlml3tikywq

Should be able to write a small function which can match on the Common Crawl
`warc_path` format.


## Sandcrawler SQL Exploration

    SELECT COUNT(*)
    FROM file_meta
    WHERE
        size_bytes = 130775;
    # 4904

## Proposed Cleanup

There are not many of these in fatcat. Propose:

0. wait until `file_meta` updates are complete, and re-dump metadata
1. filter `file_meta` dump to entities having this file size (using `rg` and `jq` filter)
2. for each, do a sandcrawler-db CDX read and check `warc_path` for common crawl pattern
3. if it matches, update file with `file_scope=corrupt` or
   `file_scope=truncated`, and remove `release_ids`, then print out file entity
4. use fatcat-cli to update file entity

## Future Mitigation

Should have sandcrawler, and possibly fatcat ingest worker, check for
size=130775 before accepting files. Probably in sandcrawler, so it can check
for common crawl WARC item name.

Could run a patch crawl to ensure we have some copy of all these URLs.

## Commands

    zcat ../2021-11-25/file_export.json.gz \
        | pv -l \
        | rg '"size":130775,' \
        > possibly_truncated.json

    wc -l possibly_truncated.json 
    # 655

Pretty few! At least one is not corrupt:

    https://web.archive.org/web/20050909185221/http://www.nature.com:80/cgi-taf/DynaPage.taf?file=/bmt/journal/v31/n4/full/1703836a.html&filetype=pdf

Fetching from sandcrawler-db:

    http get http://wbgrp-svc506.us.archive.org:3030/pdf_meta sha1hex==eq.7d5093fa09dc174471e590aab252d875bdecc7ed

    cat possibly_truncated.json \
        | jq .sha1 -r \
        | parallel -j8 curl 'http://wbgrp-svc506.us.archive.org:3030/file_meta?sha1hex=eq.{}' \
        | jq . -c \
        | pv -l \
        > possibly_truncated.file_meta.json

    rg '"sha1hex"' possibly_truncated.file_meta.json | wc -l
    # 556

    cat possibly_truncated.json \
        | jq .sha1 -r \
        | parallel -j8 curl 'http://wbgrp-svc506.us.archive.org:3030/pdf_meta?sha1hex=eq.{}' \
        | jq . -c \
        | pv -l \
        > possibly_truncated.pdf_meta.json

    cat possibly_truncated.pdf_meta.json | rg '"success"' | wc -l
    # 66

    rg '"status"' possibly_truncated.pdf_meta.json | wc -l
    # 427

    cat possibly_truncated.json \
        | jq .sha1 -r \
        | parallel -j8 curl 'http://wbgrp-svc506.us.archive.org:3030/grobid?sha1hex=eq.{}' \
        | jq . -c \
        | pv -l \
        > possibly_truncated.grobid.json

    cat possibly_truncated.grobid.json | rg '"success"' | wc -l
    # 67

    rg '"status"' possibly_truncated.grobid.json | wc -l
    # 430

    cat possibly_truncated.pdf_meta.json \
        | rg '"parse-error"' \
        | jq '.[0].sha1hex' -r \
        | sort \
        > truncated_sha1.txt

    wc -l truncated_sha1.txt
    # 360 truncated_sha1.txt

    cat possibly_truncated.json \
        | jq .sha1 -r \
        | sort \
        > possibly_sha1.txt

    cat possibly_truncated.pdf_meta.json \
        | rg '"success"' \
        | jq '.[0].sha1hex' -r \
        | sort \
        > pdf_fine_sha1.txt

    cat possibly_truncated.json \
        | rg -v '"rel"' \
        | jq .sha1 -r \
        | sort \
        > nourl_sha1.txt

    comm -23 possibly_sha1.txt truncated_sha1.txt \
        | comm -23 - pdf_fine_sha1.txt \
        | comm -23 - nourl_sha1.txt \
        > unknown_sha1.txt

Randomly sampled 10 `truncated_sha1.txt` and all were broken PDFs.

For the others, will need to re-run this after finishing `file_meta` work?

## Prod Commands

Configure CLI:

    export FATCAT_API_HOST=https://api.fatcat.wiki
    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    fatcat-cli --version
    fatcat-cli 0.1.6

    fatcat-cli status
         API Version: 0.5.0 (local)
            API host: https://api.fatcat.wiki [successfully connected]
      Last changelog: 5636508
      API auth token: [configured]
             Account: cleanup-bot [bot] [admin] [active]
                      editor_vvnmtzskhngxnicockn4iavyxq

Start small and review:

    cat /srv/fatcat/datasets/truncated_sha1.txt \
        | awk '{print "sha1:" $0}' \
        | parallel -j1 fatcat-cli get {} --json \
        | jq . -c \
        | rg -v '"content_scope"' \
        | rg 130775 \
        | head -n10 \
        | fatcat-cli batch update file release_ids= content_scope=truncated --description 'Flag truncated/corrupt PDFs (due to common crawl truncation)'
    # editgroup_3mviue5zebge3d2lqafgkfgwqa

Reviewed and all were corrupt. Running the rest of the batch:

    cat /srv/fatcat/datasets/truncated_sha1.txt \
        | awk '{print "sha1:" $0}' \
        | parallel -j1 fatcat-cli get {} --json \
        | jq . -c \
        | rg -v '"content_scope"' \
        | rg 130775 \
        | fatcat-cli batch update file release_ids= content_scope=truncated --description 'Flag truncated/corrupt PDFs (due to common crawl truncation)' --auto-accept

And then the other batch for review (no `--auto-accept`):

    cat /srv/fatcat/datasets/unknown_sha1.txt \
        | awk '{print "sha1:" $0}' \
        | parallel -j1 fatcat-cli get {} --json \
        | jq . -c \
        | rg -v '"content_scope"' \
        | rg 130775 \
        | fatcat-cli batch update file release_ids= content_scope=truncated --description 'Flag truncated/corrupt PDFs (due to common crawl truncation)'
    # editgroup_7l32piag7vho5d6gz6ee6zbtgi
    # editgroup_cnoheod4jjbevdzez7m5z4o64i
    # editgroup_w3fdmv4ffjeytnfjh32t5yovsq

These files were *not* truncated:

    file_2unklhykw5dwpotmslsldhlofy / 68821b6042a0a15fc788e99a400a1e7129d651a3
    file_xyfssct5pvde3ebg64bxoudcri / 7f307ebc2ce71c5f8a32ea4a77319403c8b87d95
    file_rrwbddwwrjg5tk4zyr5g63p2xi / ebe9f9d5d9def885e5a1ef220238f9ea9907dde1
    file_5bbhyx6labaxniz3pm2lvkr3wq / e498ee945b57b2d556bb2ed4a7d83e103bb3cc07

All the other "unknown" were, and were updated (editgroups accepted).
