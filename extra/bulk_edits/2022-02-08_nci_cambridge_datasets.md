
Spectra DSpace Instance Cleanups
================================

Basic query:

    doi_prefix:10.14469

There were a big spike of these in 2014, marked as `article`, but should be
`dataset` (or `entry`). On the order of 150k releases. In particular, causes a
weird bump in unarchived OA papers in coverage plots for the year 2014.

This is technically a dspace instance and might have various types of content
in it, so might want to narrow down the filter in some way. Eg, title prefix,
DOI pattern, etc.

    fatcat-cli search releases doi_prefix:10.14469 type:article --count
    196236

    fatcat-cli search releases doi_prefix:10.14469 type:article 'title:NSC*' --count
    158380

    fatcat-cli search releases doi_prefix:10.14469 type:article 'title:NSC*' author:"Imperial College High Performance Computing Service" --count
    158380

That seems to nail it down pretty well; these only fall under 2014 and a bit in
2015.

Want to just mark these as `release_type:entry` (they are sort of datasets, but
really it is all one big database and these are individual entries within
that).

Commands: 

    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    # start small
    fatcat-cli search releases doi_prefix:10.14469 type:article 'title:NSC*' author:"Imperial College High Performance Computing Service" --entity-json --limit 50 \
        | jq 'select(.release_type == "article")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=entry --description "Correct release_type for 'Revised Cambridge NCI database' entries"
    # Got 158380 hits
    # editgroup_mwuqpc5j3fhtjg5vxvr2xnitda

Looks good, do the full batch (!):

    fatcat-cli search releases doi_prefix:10.14469 type:article 'title:NSC*' author:"Imperial College High Performance Computing Service" --entity-json --limit 160000 \
        | jq 'select(.release_type == "article")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=entry --description "Correct release_type for 'Revised Cambridge NCI database' entries" --auto-accept
    # 158k 1:00:21 [43.7 /s]

Off it goes!

There are more patterns from this repository, but this is a good start.
