
Importing fileset and file entities from initial sandcrawler ingests.

Git commit: `ede98644a89afd15d903061e0998dbd08851df6d`

Filesets:

    export FATCAT_AUTH_SANDCRAWLER=[...]
    cat /tmp/ingest_dataset_combined_results.2022-04-04.partial.json \
        | ./fatcat_import.py ingest-fileset-results -
    # editgroup_5l47i7bscvfmpf4ddytauoekea
    # Counter({'total': 195, 'skip': 176, 'skip-hit': 160, 'insert': 19, 'skip-single-file': 14, 'skip-partial-file-info': 2, 'update': 0, 'exists': 0})

    cat /srv/fatcat/datasets/ingest_dataset_combined_results.2022-04-04.partial.json \
        | ./fatcat_import.py ingest-fileset-file-results -
    # editgroup_i2k2ucon7nap3gui3z7amuiug4
    # Counter({'total': 195, 'skip': 184, 'skip-hit': 160, 'skip-status': 24, 'insert': 11, 'update': 0, 'exists': 0})

Tried running again, to ensure that there are not duplicate inserts, and that
worked ('exists' instead of 'insert' counts).

Finally!
