
Grouped Release Exports
=======================

Want to have rich "work" entity dumps, without duplicating with enriched
release dumps. The motivation for this is to do work-level processing of
releases. For example, transform into scholar.archive.org (indexed at work
level), bulk downloading fulltext (only need "best copy for entire work"),
citation analysis, etc.

One elegant way to do this would be having release exports be sorted by
`work_id`, with all release entities with the same `work_id` contiguous (but
not all on the same line).

Plan is to update fatcat-export tool (rust) to read in an ident list with
`work_id` included (and sorted by the `work_id`).

## Rust Implementation

Remembering that `fatcat-export` operates on lines, and uses channels and
worker threads for processing and serialization, we can't just assume that the
output lines will be in the same order as the input lines (eg, worker threads
can shift order). To preserve order, we will add a new dump mode which operates
on batches (`Vec`) of rows instead, and write the batches contiguously. The
batches themselves may end up out of order, but that is fine.

## SQL Ident Dump


Database timing (running 2020-08-04 on fatcat-qa):

    COPY (SELECT id, rev_id, redirect_id FROM release_ident      WHERE is_live=true) TO '/tmp/fatcat_ident_releases.tsv'            WITH NULL '';
    => COPY 142635153
    => Time: 143264.483 ms (02:23.264)

    COPY (
      SELECT
        release_ident.id,
        release_ident.rev_id,
        release_ident.redirect_id,
        release_rev.work_ident_id
      FROM
        release_ident
        LEFT JOIN release_rev ON release_ident.rev_id = release_rev.id
      WHERE
        release_ident.is_live=true
        AND release_ident.redirect_id IS NULL
        AND release_ident.rev_id IS NOT NULL
      ORDER BY
        release_rev.work_ident_id ASC NULLS LAST
    ) TO '/tmp/fatcat_ident_releases_by_work.tsv'            WITH NULL '';
    => COPY 142635147
    => Time: 610540.112 ms (10:10.540)

Much slower, but not a blocking issue. Apparently postgresql will do a full
rowscan instead of using the existing `work_id` sorted index on `release_rev`
when going of a large fraction of the table (here almost the entire table)
because it thinks the read will be so much faster. I don't think this is true
with SSDs? Haven't actually confirmed this is the behavior, just assuming based
on postgresql docs.
