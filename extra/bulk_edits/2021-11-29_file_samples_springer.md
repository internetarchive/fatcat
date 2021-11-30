
Ran a cleanup of ~24k file entities from the domain
`page-one.live.cf.public.springer.com`, which are not entire journal articles
but just "samples" (one or two pages).

See `file_single_page` cleanup notes for prep and background.


## Prod

Configure CLI:

    export FATCAT_API_HOST=https://api.fatcat.wiki
    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    fatcat-cli --version
    fatcat-cli 0.1.6

    fatcat-cli status
         API Version: 0.5.0 (local)
            API host: https://api.fatcat.wiki [successfully connected]
      Last changelog: 5634988
      API auth token: [configured]
             Account: cleanup-bot [bot] [admin] [active]
                      editor_vvnmtzskhngxnicockn4iavyxq

Start small:

    zcat /srv/fatcat/datasets/files_pageone.json.gz \
        | jq '"file_" + .ident' -r \
        | head -n50 \
        | parallel -j1 fatcat-cli get {} --json \
        | jq . -c \
        | rg -v '"content_scope"' \
        | rg 'page-one.live.cf.public.springer.com' \
        | pv -l \
        | fatcat-cli batch update file release_ids= content_scope=sample --description 'Un-link and mark Springer "page-one" preview PDF files as content_scope=sample'
    # editgroup_hcumfatcvjg3fheycnm2uay5aq

Looks good, accepted that editgroup.

Run entire batch, in auto-accept mode:

    zcat /srv/fatcat/datasets/files_pageone.json.gz \
        | jq '"file_" + .ident' -r \
        | parallel -j1 fatcat-cli get {} --json \
        | jq . -c \
        | rg -v '"content_scope"' \
        | rg 'page-one.live.cf.public.springer.com' \
        | pv -l \
        | fatcat-cli batch update file release_ids= content_scope=sample --description 'Un-link and mark Springer "page-one" preview PDF files as content_scope=sample' --auto-accept
    # 24.4k 0:20:06 [20.2 /s]
