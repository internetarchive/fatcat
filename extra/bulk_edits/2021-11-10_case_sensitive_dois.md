
## Production Run

Start small:

    export FATCAT_AUTH_WORKER_CLEANUP=[...]

    wc -l /srv/fatcat/datasets/nonlowercase_doi_releases.tsv
    # 140530

    head -n100 /srv/fatcat/datasets/nonlowercase_doi_releases.tsv \
        | python -m fatcat_tools.cleanups.release_lowercase_doi -
    # Counter({'total': 100, 'update': 100, 'skip': 0, 'insert': 0, 'exists': 0})

    # same command again to test not duping updates
    Counter({'total': 100, 'skip-existing-doi-fine': 100, 'skip': 0, 'insert': 0, 'update': 0, 'exists': 0})

    # example editgroup_cld5qe34bzg7xg7g4cz5skgaw4

Database size just before, while some other edits happening, PostgreSQL 11.6: 762.66G

Ok, run a bunch in parallel:

    cat /srv/fatcat/datasets/nonlowercase_doi_releases.tsv \
        | parallel -j8 --linebuffer --round-robin --pipe python -m fatcat_tools.cleanups.release_lowercase_doi -
    # Counter({'total': 24022, 'update': 24022, 'skip': 0, 'insert': 0, 'exists': 0})
    # Counter({'total': 38836, 'update': 38836, 'skip': 0, 'insert': 0, 'exists': 0})
    # Counter({'total': 38836, 'update': 38836, 'skip': 0, 'insert': 0, 'exists': 0})
    # Counter({'total': 38836, 'update': 38736, 'skip-existing-doi-fine': 100, 'skip': 0, 'insert': 0, 'exists': 0})

Over 3k TPS in `pg_activity`.

Should have included `pv -l` in the pipeline.

Final database size 763.14G, so only a couple hundred MByte of growth, totally
fine.


## Verification

Re-dump release extids, in production:

    sudo -u postgres psql fatcat_prod < dump_release_extid.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | pigz > /srv/fatcat/snapshots/release_extid.tsv.gz

Filter to non-lowercase DOIs:

    zcat release_extid.tsv.gz \
        | cut -f1,3 \
        | rg '[A-Z]' \
        | pv -l \
        > nonlowercase_doi.tsv

Zero returned, hurray!
