

## Prep

Using `check_hashes.sh`:

    zcat $HASH_FILE \
        | awk '{print $3 "\t" $1}' \
        | rg -v '^\t' \
        | sort -S 4G \
        | uniq -D -w 40 \
        > sha1_ident.dupes.tsv

    wc -l sha1_ident.dupes.tsv 
    # 6,350

    cut -f1 sha1_ident.dupes.tsv | uniq | wc -l
    # 2,039

Want to create JSON for each group, like:

    entity_type: "file"
    primary_id: str or None
    duplicate_ids: [str]
    evidence:
        extid: str
        extid_type: "sha1"

Run transform script:

    cat sha1_ident.dupes.tsv | ./file_dupe_to_json.py | pv -l > file_sha1_dupes.json
    # 2.04k 0:00:00 [9.16k/s]


## QA Testing

    export FATCAT_AUTH_API_TOKEN=[...]

    head -n25 /srv/fatcat/datasets/file_sha1_dupes.json \
        | python -m fatcat_tools.mergers.files --editgroup-description-override "Automated merging of file entities with duplicate SHA-1 hashes" --dry-run merge-files -

Hit some small bugs running in QA; test coverage isn't great, but I think hits
the important parts.

    head -n25 /srv/fatcat/datasets/file_sha1_dupes.json \
        | python -m fatcat_tools.mergers.files --editgroup-description-override "Automated merging of file entities with duplicate SHA-1 hashes" --dry-run merge-files -
    # Running in dry-run mode!
    # Counter({'updated-entities': 60, 'lines': 25, 'merged': 25, 'skip': 0, 'updated-total': 0})

Dry-run mode didn't actually work, and edits actually happened (!).

Edits do look good.

Try again, not dry-run, to ensure that case is handled:

    head -n25 /srv/fatcat/datasets/file_sha1_dupes.json | python -m fatcat_tools.mergers.files --editgroup-description-override "Automated merging of file entities with duplicate SHA-1 hashes" merge-files -
    # Counter({'lines': 25, 'skip': 25, 'skip-not-active-entity': 25, 'merged': 0, 'updated-total': 0})

And then run 500 through for more testing:

    head -n500 /srv/fatcat/datasets/file_sha1_dupes.json | python -m fatcat_tools.mergers.files --editgroup-description-override "Automated merging of file entities with duplicate SHA-1 hashes" merge-files -
    # Counter({'updated-entities': 1341, 'lines': 500, 'merged': 474, 'skip': 26, 'skip-not-active-entity': 25, 'skip-entity-not-found': 1, 'updated-total': 0})

The majority of merges seem to be cases where there are multiple articles in the same PDF.
