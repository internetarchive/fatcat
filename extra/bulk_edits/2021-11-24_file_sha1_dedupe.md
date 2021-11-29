
See notes and scripts about `file_sha1_dedupe` cleanup for prep details.

## Prod Run

Run as `cleanup-bot`:

    export FATCAT_AUTH_API_TOKEN=[...]

    git log | head -n1
    # commit 5bc5eeed5e3ba54c2129c4233b881291c5fa7449

First do a sample in dry-run mode:

    head -n25 /srv/fatcat/datasets/file_sha1_dupes.json \
        | python -m fatcat_tools.mergers.files --editgroup-description-override "Automated merging of file entities with duplicate SHA-1 hashes" --dry-run merge-files -
    # Counter({'updated-entities': 59, 'lines': 25, 'merged': 25, 'skip': 0, 'updated-total': 0})

Gah, the dry-run mode still creates (empty) editgroups:

    https://fatcat.wiki/editgroup/iqzjg3vxu5elvotknmmjln3gv4
    https://fatcat.wiki/editgroup/2mxsl7lxo5dezem42whnr7zxxe

Actually run (merge) the sample:

    head -n25 /srv/fatcat/datasets/file_sha1_dupes.json \
        | python -m fatcat_tools.mergers.files --editgroup-description-override "Automated merging of file entities with duplicate SHA-1 hashes" merge-files -
    # Counter({'updated-entities': 59, 'lines': 25, 'merged': 25, 'skip': 0, 'updated-total': 0})


Run the full batch:

    cat /srv/fatcat/datasets/file_sha1_dupes.json \
        | python -m fatcat_tools.mergers.files --editgroup-description-override "Automated merging of file entities with duplicate SHA-1 hashes" merge-files -
    # Counter({'updated-entities': 6197, 'lines': 2039, 'merged': 2014, 'skip': 25, 'skip-not-active-entity': 25, 'updated-total': 0})
