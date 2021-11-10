
Relevant github issue: https://github.com/internetarchive/fatcat/issues/83

How many existing fatcat releases have a non-lowercase DOI? As of June 2021:

    zcat release_extid.tsv.gz | cut -f3 | rg '[A-Z]' | pv -l | wc -l
    139964

## Prep

    wget https://archive.org/download/fatcat_bulk_exports_2021-11-05/release_extid.tsv.gz

    # scratch:bin/fcid.py is roughly the same as `fatcat_util.py uuid2fcid`

    zcat release_extid.tsv.gz \
        | cut -f1,3 \
        | rg '[A-Z]' \
        | /fast/scratch/bin/fcid.py \
        | pv -l \
        > nonlowercase_doi_releases.tsv
    # 140k 0:03:54 [ 599 /s]

    wc -l nonlowercase_doi_releases.tsv
    140530 nonlowercase_doi_releases.tsv

Uhoh, there are ~500 more than previously? Guess those are from after the fix?

Create a sample for testing:

    shuf -n10000 nonlowercase_doi_releases.tsv \
        > nonlowercase_doi_releases.10k_sample.tsv

## Test in QA

In pipenv:

    export FATCAT_AUTH_WORKER_CLEANUP=[...]

    head -n100 /srv/fatcat/datasets/nonlowercase_doi_releases.10k_sample.tsv \
        | python -m fatcat_tools.cleanups.release_lowercase_doi -
    # Counter({'total': 100, 'update': 100, 'skip': 0, 'insert': 0, 'exists': 0})

    head -n100 /srv/fatcat/datasets/nonlowercase_doi_releases.10k_sample.tsv \
        | python -m fatcat_tools.cleanups.release_lowercase_doi -
    # Counter({'total': 100, 'skip-existing-doi-fine': 100, 'skip': 0, 'insert': 0, 'update': 0, 'exists': 0})

    head -n2000 /srv/fatcat/datasets/nonlowercase_doi_releases.10k_sample.tsv \
        | python -m fatcat_tools.cleanups.release_lowercase_doi -
    # no such release_ident found: dcjsybvqanffhmu4dhzdnptave

Presumably because this is being run in QA, and there are some newer prod releases in the snapshot.

Did a quick update, and then:

    head -n2000 /srv/fatcat/datasets/nonlowercase_doi_releases.10k_sample.tsv \
        | python -m fatcat_tools.cleanups.release_lowercase_doi -
    # Counter({'total': 2000, 'skip-existing-doi-fine': 1100, 'update': 898, 'skip-existing-not-found': 2, 'skip': 0, 'insert': 0, 'exists': 0})

Did some spot checking in QA. Out of 20 DOIs checked, 15 were valid, 5 were not
valid (doi.org 404). It seems like roughly 1/3 have a dupe DOI (the lower-case
DOI exists); didn't count exact numbers.

This cleanup is simple and looks good to go. Batch size of 50 is good for full
releases.

Example of parallelization:

    cat /srv/fatcat/datasets/nonlowercase_doi_releases.10k_sample.tsv \
        | parallel -j8 --linebuffer --round-robin --pipe python -m fatcat_tools.cleanups.release_lowercase_doi -

Ready to go!
