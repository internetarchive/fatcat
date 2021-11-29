
Simply de-duplicating container entities on the basis of ISSN-L.

Initial plan is to:

- only merge containers with zero (0) release entities pointing at them
- not update any containers which have had human edits
- not merge additional metadata from redirected entities to the "primary" entity


## Prep

Using commands from `check_issnl.sh`:

    zcat container_export.json.gz \
        | jq '[.issnl, .ident] | @tsv' -r \
        | sort -S 4G \
        | uniq -D -w 9 \
        > issnl_ident.dupes.tsv

    wc -l issnl_ident.dupes.tsv
    # 3174 issnl_ident.dupes.tsv

    cut -f1 issnl_ident.dupes.tsv | uniq | wc -l
    # 835

Run transform script:

    cat issnl_ident.dupes.tsv | ./container_dupe_to_json.py | pv -l > container_issnl_dupes.json

Create a small random sample:

    shuf -n100 container_issnl_dupes.json > container_issnl_dupes.sample.json

## QA Testing

    git log | head -n1
    # commit e72d61e60c43911b6d77c4842951441235561dcf

    export FATCAT_AUTH_API_TOKEN=[...]

    head -n25 /srv/fatcat/datasets/container_issnl_dupes.sample.json \
        | python -m fatcat_tools.mergers.containers --editgroup-description-override "Automated merging of duplicate container entities with the same ISSN-L" --dry-run merge-containers -

Got various errors and patched them:

    AttributeError: 'EntityHistoryEntry' object has no attribute 'editor'

    requests.exceptions.HTTPError: 404 Client Error: NOT FOUND for url: https://fatcat.wiki/container/%7Bident%7D/stats.json

    fatcat_openapi_client.exceptions.ApiValueError: Missing the required parameter `editgroup_id` when calling `accept_editgroup`

Run again:

    head -n25 /srv/fatcat/datasets/container_issnl_dupes.sample.json \
        | python -m fatcat_tools.mergers.containers --editgroup-description-override "Automated merging of duplicate container entities with the same ISSN-L" --dry-run merge-containers -
    # Running in dry-run mode!
    # Counter({'updated-entities': 96, 'skip-container-release-count': 84, 'lines': 25, 'merged': 25, 'skip': 0, 'updated-total': 0})

Finally! dry-run mode actually worked. Try entire sample in dry-run:

    cat /srv/fatcat/datasets/container_issnl_dupes.sample.json \
        | python -m fatcat_tools.mergers.containers --editgroup-description-override "Automated merging of duplicate container entities with the same ISSN-L" --dry-run merge-containers -
    # Running in dry-run mode!
    # Counter({'updated-entities': 310, 'skip-container-release-count': 251, 'lines': 100, 'merged': 100, 'skip': 0, 'updated-total': 0})

How about a small `max-container-releases`:

    cat /srv/fatcat/datasets/container_issnl_dupes.sample.json \
        | python -m fatcat_tools.mergers.containers --editgroup-description-override "Automated merging of duplicate container entities with the same ISSN-L" --dry-run merge-containers -
    # Running in dry-run mode!
    # Counter({'updated-entities': 310, 'skip-container-release-count': 251, 'lines': 100, 'merged': 100, 'skip': 0, 'updated-total': 0})

Exact same count... maybe something isn't working? Debugged and fixed it.

    requests.exceptions.HTTPError: 503 Server Error: SERVICE UNAVAILABLE for url: https://fatcat.wiki/container/xn7i2sdijzbypcetz77kttj76y/stats.json

    # Running in dry-run mode!
    # Counter({'updated-entities': 310, 'lines': 100, 'merged': 100, 'skip-container-release-count': 92, 'skip': 0, 'updated-total': 0})

From skimming, it looks like 100 is probably a good cut-off. There are sort of
a lot of these dupes!

Try some actual merges:

    head -n25 /srv/fatcat/datasets/container_issnl_dupes.sample.json \
        | python -m fatcat_tools.mergers.containers --editgroup-description-override "Automated merging of duplicate container entities with the same ISSN-L" merge-containers -
    # Counter({'updated-entities': 96, 'skip-container-release-count': 84, 'lines': 25, 'merged': 25, 'skip': 0, 'updated-total': 0})

Run immediately again:

    # Counter({'lines': 25, 'skip': 25, 'skip-not-active-entity': 25, 'skip-container-release-count': 2, 'merged': 0, 'updated-total': 0})

Run all the samples, with limit of 100 releases:

    cat /srv/fatcat/datasets/container_issnl_dupes.sample.json \
        | python -m fatcat_tools.mergers.containers --editgroup-description-override "Automated merging of duplicate container entities with the same ISSN-L" merge-containers - --max-container-releases 100
    # Counter({'updated-entities': 214, 'lines': 100, 'merged': 75, 'skip': 25, 'skip-not-active-entity': 25, 'skip-container-release-count': 15, 'updated-total': 0})

Wow, there are going to be a lot of these containers not merged because they
have so many releases! Will have to do a second, more carefully reviewed (?)
round of merging.

Unfortunately, not seeing any human-edited container entities here to check if
that filter is working.
