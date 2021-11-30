
In the past we have crawled and imported many PDF files from Springer, which in
fact are only first-page samples of articles, not the entire fulltext work itself.

The original domain these are served from is:

- `page-one.live.cf.public.springer.com`

The cleanup for these is to update the files with:

1. set the `content_scope` field to `sample`, and remove any `release_ids`
2. in downstream contexts, don't treat `content_scope=sample` as a complete entity

An alternative would be to just delete these files. But this would likely
result in them being re-imported in the future.

We might also want to retain the `release_ids` linkage between files and
releases. The current semantics of file/release are that the file represents a
valid preservation copy of the release though, and that is not the case here.
In this specific case the relationship is partially preserved in edit history,
and could be resurected programatically if needed.

## Existing Files with URL

The easy case to detect is when these files are in fatcat with the original
URL. How many files fall in that cateogry?

    zcat file_export.json.gz \
        | rg '//page-one.live.cf.public.springer.com/' \
        | pv -l \
        | pigz \
        > files_pageone.json.gz
    # 24.4k 0:10:03 [40.5 /s]

## Existing Files without URL

After a partial cleanup, export, and re-load of the `release_file` table in
sandcrawler-db, we could do a join between the `fatcat_file` and the `cdx`
table to identify PDFs which have ever been crawled from
`page-one.live.cf.public.springer.com` and have an entity in fatcat.


## QA

    export FATCAT_API_HOST=https://api.qa.fatcat.wiki
    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    fatcat-cli --version
    fatcat-cli 0.1.6


    fatcat-cli status
         API Version: 0.5.0 (local)
            API host: https://api.qa.fatcat.wiki [successfully connected]
      Last changelog: 5326137
      API auth token: [configured]
             Account: cleanup-bot [bot] [admin] [active]
                      editor_vvnmtzskhngxnicockn4iavyxq

    zcat /srv/fatcat/datasets/files_pageone.json.gz \
        | jq '"file_" + .ident' -r \
        | head -n100 \
        | parallel -j1 fatcat-cli get {} --json \
        | jq . -c \
        | rg -v '"content_scope"' \
        | rg 'page-one.live.cf.public.springer.com' \
        | pv -l \
        | fatcat-cli batch update file release_ids= content_scope=sample --description 'Un-link and mark Springer "page-one" preview PDF files as content_scope=sample' --auto-accept

Looks good!


## Prod

See bulk edits log.
