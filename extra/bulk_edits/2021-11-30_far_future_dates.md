
This is an experiment in using `fatcat-cli` for bulk updates of metadata.

As a simple initial cleanup, looking at far-future release entities, which have
dates beyond year 2100 (or even beyond year 5000). These are the result of not
checking for reasonable date values during initial catalog bootstrapping,
combined with poor upstream metadata.

    fatcat-cli search release --count 'year:>2100'
    # 279622

    fatcat-cli search release --count 'year:>5000'
    # 123035

That is a lot of bad releases!

## Source of Bad Metadata

Where were these imported from?

9999 seems to be pretty much just JALC:

    fatcat-cli search release --count 'year:>5000' doi_registrar:jalc
    # 122279

    fatcat-cli search release --count 'year:9999' doi_registrar:jalc
    # 122279

    fatcat-cli search release --count 'year:9999' '!doi_registrar:jalc'
    # 380

A handful from longtail OA import:

    fatcat-cli search release --count 'year:>5000' is_longtail_oa:true
    84

    fatcat-cli search release --count 'year:>2100' is_longtail_oa:true
    669

A significant number from datacite:

    fatcat-cli search release --count 'year:>5000' doi_registrar:datacite
    672

    fatcat-cli search release --count 'year:>2100' doi_registrar:datacite
    156359

And a handful from crossref:

    fatcat-cli search release --count 'year:>2100' '!doi_registrar:datacite' '!doi_registrar:jalc' '!is_longtail_oa:true'
    314

    fatcat-cli search release 'year:>2100' '!doi_registrar:datacite' '!doi_registrar:jalc' '!is_longtail_oa:true' --index-json --limit 0 | jq .doi_registrar | sort | uniq -c
    Got 314 hits in 81ms
        314 "crossref"

How about no DOI and not longtail OA?

    fatcat-cli search release --count 'year:>2100' is_longtail_oa:false '!doi:*'

## QA (2021-11-17)

    export FATCAT_API_HOST=https://api.qa.fatcat.wiki
    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP
    # using prod search index

    fatcat-cli --version
    fatcat-cli 0.1.6


    bnewbold@orithena$ fatcat-cli status
         API Version: 0.5.0 (local)
            API host: https://api.qa.fatcat.wiki [successfully connected]
      Last changelog: 5326062
      API auth token: [configured]
             Account: cleanup-bot [bot] [admin] [active]
                      editor_vvnmtzskhngxnicockn4iavyxq

Year `9999` seems popular and is probably a place-holder; skip those for later re-import/update.

The QA catalog is out of date from prod, so need to filter to only older
releases using `doc_index_ts` (as a hack).

We want to re-filter locally to ensure that the fields are actually bogus. `jq` works for this.

    # NOTE: in update chain, important to *NOT* use `--hide` here
    fatcat-cli search releases 'year:>5000' '!year:9999' 'doc_index_ts:<2021-05-01' --entity-json --limit 20 \
        | jq 'select(.release_year > 5000)' -c \
        | fatcat-cli batch update release release_year= release_date=
    # Got 376 hits in 28ms
    # editgroup_q2rqwoaa4zdmhmlmyrtcc5tnia

Editgroup came through with agent, `agent_version` correct. Only 5 edits. Newer 
presumably because entities have been updated in prod. Description was "part of
a fatcat-cli batch operation", should be able to modify this (CLI arg?  env
variable?).

Trying again with `--limit 200`. Got 2x editgroups.

Trying different search parameters:

    fatcat-cli search releases 'year:>2100' is_longtail_oa:true 'doc_index_ts:<2021-05-01' --entity-json --limit 100 \
        | jq 'select(.release_year > 2100)' -c \
        | wc -l
    # Got 669 hits in 312ms
    # 100

Try doing 300 updates, with auto-accept:

    fatcat-cli search releases 'year:>2100' is_longtail_oa:true 'doc_index_ts:<2021-05-01' --entity-json --limit 300 \
        | jq 'select(.release_year > 2100)' -c \
        | pv -l \
        | fatcat-cli batch update release release_year= release_date= --auto-accept

## Production (2021-11-30)

Scope for this initial update:

- longtail releases
- Crossref
- other non-DOI releases (?)
- possibly JALC releases (more than 100k)


    export FATCAT_API_HOST=https://api.fatcat.wiki
    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    fatcat-cli --version
    # fatcat-cli 0.1.6

    fatcat-cli status
         API Version: 0.5.0 (local)
            API host: https://api.fatcat.wiki [successfully connected]
      Last changelog: 5639457
      API auth token: [configured]
             Account: cleanup-bot [bot] [admin] [active]
                      editor_vvnmtzskhngxnicockn4iavyxq

Start with longtail releases:

    fatcat-cli search releases 'year:>2100' is_longtail_oa:true 'doc_index_ts:<2021-05-01' --entity-json --limit 100 \
        | jq 'select(.release_year > 2100)' -c \
        | wc -l
    # Got 669 hits in 375ms
    # 100

    fatcat-cli search releases 'year:>2100' is_longtail_oa:true 'doc_index_ts:<2021-05-01' '!doi:*' --entity-json --limit 100 \
        | jq 'select(.release_year > 2100)' -c \
        | head -n50 \
        | pv -l \
        | fatcat-cli batch update release release_year= release_date= --description "Cleanup of 'far-future' release dates"
    # editgroup_6sjgeun22bexlc5ujxhy6ui3sm

No dates in this batch, only years. Diff looks good. Accepted.

Run all these (few hundred, auto-accept):

    fatcat-cli search releases 'year:>2100' is_longtail_oa:true 'doc_index_ts:<2021-11-20' '!doi:*' --entity-json --limit 0 \
        | jq 'select(.release_year > 2100)' -c \
        | pv -l \
        | fatcat-cli batch update release release_year= release_date= --description "Cleanup of 'far-future' release dates" --auto-accept
    # Got 579 hits in 101ms
    # 579  0:00:15 [38.0 /s]

Next, Crossref:

    fatcat-cli search releases 'year:>2100' doi_registrar:crossref 'doc_index_ts:<2021-11-20' --count
    # 314

Small batch, no auto-accept:

    fatcat-cli search releases 'year:>2100' doi_registrar:crossref 'doc_index_ts:<2021-11-20' --entity-json --limit 20 \
        | jq 'select(.release_year > 2100)' -c \
        | pv -l \
        | fatcat-cli batch update release release_year= release_date= --description "Cleanup of 'far-future' release dates"
    # editgroup_u75a4jdgyra4pbqixt344uus4e

Looks good. Accept, then run full batch:

    fatcat-cli search releases 'year:>2100' doi_registrar:crossref 'doc_index_ts:<2021-11-20' --entity-json --limit 0 \
        | jq 'select(.release_year > 2100)' -c \
        | pv -l \
        | fatcat-cli batch update release release_year= release_date= --description "Cleanup of 'far-future' release dates" --auto-accept
    # Got 294 hits in 133ms
    # 294  0:00:05 [49.5 /s]

Any other non-DOI releases?

    fatcat-cli search releases 'year:>2100' '!doi:*' --count
    # 0


    fatcat-cli search releases 'year:>2100' 'doi_registrar:jalc' --count
    # 122280

Most of the remaining are JALC or datacite, and are large updates. Let's do
just the subset which are `in_ia`:

    fatcat-cli search releases 'year:>2100' in_ia:true --count
    # 23

    fatcat-cli search releases 'year:>2100' in_ia:true 'doc_index_ts:<2021-11-20' --entity-json --limit 50 \
        | jq 'select(.release_year > 2100)' -c \
        | pv -l \
        | fatcat-cli batch update release release_year= release_date= --description "Cleanup of 'far-future' release dates"
    # Got 23 hits in 54ms
    # 13
    # editgroup_jg35r4tr7bfavcvksh4k7k3siq

Accepted.
