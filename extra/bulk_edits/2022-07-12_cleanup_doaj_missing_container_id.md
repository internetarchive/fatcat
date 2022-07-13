
There is a batch of about 480 releases with DOAJ identifiers but no container
linkage. These seem to all be from the same actual container:

    fatcat-cli search releases 'doaj_id:*' '!container_id:*' --count
    # 486

    fatcat-cli search releases 'doaj_id:*' '!container_id:*' --index-json -n 0 | jq .containe
    # Got 486 hits in 138ms
    # "Revista de Sistemas, Cibernética e Informática"

Edit pipeline:

    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    # start small
    fatcat-cli search releases 'doaj_id:*' '!container_id:*' 'journal:Cibernética' --entity-json --limit 50 \
        | jq 'select(.container_id == null)' -c \
        | rg 'Cibernética' \
        | fatcat-cli batch update release container_id=ubwuhr4obzgr7aadszhurhef5m --description "Add container linkage for DOAJ articles with ISSN 1690-8627"
    # editgroup_g2zrm3wkmneoldtqfxpbkaoeh4

Looks good, merged.

    # full auto
    fatcat-cli search releases 'doaj_id:*' '!container_id:*' 'journal:Cibernética' --entity-json --limit 500 \
        | jq 'select(.container_id == null)' -c \
        | rg 'Cibernética' \
        | fatcat-cli batch update release container_id=ubwuhr4obzgr7aadszhurhef5m --description "Add container linkage for DOAJ articles with ISSN 1690-8627" --auto-accept

Verify:

    fatcat-cli search releases 'doaj_id:*' '!container_id:*' --count
    # 0

Also planning to have DOAJ article importer 'skip' in the future for articles
with no `container_id` match.
