
PLOS publishes a number of non-articles, and many are not correctly marked in
metadata.

## Issue Images

    fatcat-cli search releases doi_prefix:10.1371 title:image --index-json -n0 | rg '10.1371/image.' | wc -l
    # Got 1142 hits in 92ms
    # 348

    fatcat-cli search releases doi_prefix:10.1371 title:"issue image" --count
    # 348

    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    # start small
    fatcat-cli search releases doi_prefix:10.1371 title:"issue image" release_type:article-journal --entity-json -n 400 \
        | jq 'select(.release_type == "article-journal")' -c \
        | rg '10.1371/image.' \
        | head -n50 \
        | fatcat-cli batch update release release_type=graphic --description "PLoS Issue Images as type 'graphic'"
    # Got 348 hits in 121ms
    # editgroup_cq5cch7pmjglpehojhmza5hvxq

    # the rest
    fatcat-cli search releases doi_prefix:10.1371 title:"issue image" release_type:article-journal --entity-json -n 400 \
        | jq 'select(.release_type == "article-journal")' -c \
        | rg '10.1371/image.' \
        | fatcat-cli batch update release release_type=graphic --description "PLoS Issue Images as type 'graphic'" --auto-accept
    # Got 298 hits in 105ms

## Non-PLOS DOI Releases

    !doi_prefix:10.1371 container_id:iznnn644szdwva7khyxqzc73bi
    # 10

Some of these are "repo DOIs with `container_id`", some are DOAJ. The DOAJ ones
did not fuzzy-match mostly because of greek characters, and should be merged...
manually? In this case there are only a handful, but there will be more
elsewhere.

    fatcat-cli search releases title:"authors reply" 'container_id:*' 'doaj_id:*' --count
    # 275

    fatcat-cli search releases title:"authors reply" 'container_id:*' 'doaj_id:*' plos --count
    # 5

    fatcat-cli search releases '!doi_prefix:10.1371' '!pmid:*' '!doi:*' 'container_id:*' journal:plos 'doaj_id:*' --count
    # 1511

    fatcat-cli search releases '!doi_prefix:10.1371' '!pmid:*' '!doi:*' 'container_id:*' journal:plos 'doaj_id:*' '!title:correction' --count
    # 35

    fatcat-cli search releases '!doi_prefix:10.1371' 'container_id:*' journal:plos --count
    # 2012

(note: the above run while in the process of removing a lot of "RWTH" repo DOIs)

Ok, after the batch fixups:

    fatcat-cli search releases '!doi_prefix:10.1371' 'container_id:*' journal:plos --count
    1507

    fatcat-cli search releases '!doi_prefix:10.1371' 'container_id:*' journal:plos '!doaj_id:*' --count
    4

Will fix these up manually. The DOAJ cleanups will be more involved... should
probably add a simple blocklist in DOAJ article importer to skip attempts.
