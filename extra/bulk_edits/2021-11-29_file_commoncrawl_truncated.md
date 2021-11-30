
Some PDF files in wayback were crawled externally by the Common Crawl project,
and are truncted to 128 KB (130775 bytes). This corrupts the PDF files, and
they can not be read. We want to remove these from the catalog, or at least,
mark them as non-archival and "truncated" (in `content_scope` field).

See the cleanup file for prep, counts, and other background.

There were over 600 file entities with the exact file size. Some are valid, but
over 400 were found to be corrupt, and updated in the catalog. The majority
seemed to have alternate captures and distinct file entities (not truncated).

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
