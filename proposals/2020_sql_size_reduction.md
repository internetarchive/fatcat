
status: brainstorming

This document is tracking ideas to reduce fatcat catalog database size (postgresql).

As of January 2020, the prod database is over 470 GBytes, on DB hosts with 2
TByte disks. We are probably fine doubling (up to ~1000 GBytes) with no serious
issues, and there are lots of ideas beyond that (4 TByte SSDs disks, multiple
SSDs using RAID or postgres table spaces, some spinning disk usage, etc), but
in the meanwhile it might be worth trying to slim down the existing schema.

See also `20191018_bigger_db.md` about far future database size. Scale-out
(horizontal) and other larger refactors are out of scope for this document.


## Current Database Usage and Growth

`extra/stats/2020-01-19-prod-table-sizes.txt` shows table data and index sizes.

To categorize these in a few different ways:

- release entities (incl. edits, ident, revs)
    - 72%   337 GB total, 215 GB data, 122 GB index
- file entities (incl. edits, ident, revs):
    -  7%    34 GB total,  18 GB data,  16 GB index
- release entity revs (no edits, ident)
    - 61%   287 GB total, 200 GB data,  87 GB index
- entity edits only
    - 15%    71 GB total,  30 GB data,  40 GB index

`refs_blob` is large, but is "content addressed", so as long as we don't
*update* the *non-linkage* reference metadata, will not grow. We are also
planning on holding off on dumping all references (citation graph) directly
into the catalog at this time.

`release_contrib`, `release_rev`, and `release_edit` are more concerning, as
these are expected to grow linearly (or faster). Just those three tables are
46% of the database.

We expect to both update almost all files and quadruple the number of files in
the near future (roughly 25m currently; expect 80m+20m edits soon, so 5x
database size). Will also be filling in additional hashes and other metadata.
All that together would be only some 80 GB additional disk size; maybe less if
we de-dupe URLs.

Releases we are likely to want to do something like 20 million entity updates (as
part of cleanups and merging) and add another 20 million entities (from CORE,
MAG, longtail OA, etc). That would be about a 30% increase in rev count, or
another 100 GB of data+index.

Other growth is expected to be much smaller, let's say a few GB of disk.

This works out to a bit over 600 GByte total disk size.


## Idea: finish `ext_id` migration and drop columns+index from `release_rev`

Every `release_rev` table has DOI, CORE, PMID, PMCID, and Wikidata columns,
*and* indices on all of these. For all but DOI, the majority of release revs
have these columns NULL. This doesn't waste much disk, but it does waste index:

    fatcat_prod=# \di+ release_rev*
                                                  List of relations
     Schema |               Name               | Type  |  Owner   |        Table         |  Size   | Description 
    --------+----------------------------------+-------+----------+----------------------+---------+-------------
     public | release_rev_abstract_pkey        | index | postgres | release_rev_abstract | 469 MB  | 
     public | release_rev_abstract_rev_idx     | index | postgres | release_rev_abstract | 658 MB  | 
     public | release_rev_abstract_sha1_idx    | index | postgres | release_rev_abstract | 1416 MB | 
     public | release_rev_core_idx             | index | postgres | release_rev          | 3156 MB | 
     public | release_rev_doi_idx              | index | postgres | release_rev          | 6124 MB | 
     public | release_rev_extid_pkey           | index | fatcat   | release_rev_extid    | 119 MB  | 
     public | release_rev_extid_type_value_idx | index | fatcat   | release_rev_extid    | 117 MB  | 
     public | release_rev_pkey                 | index | postgres | release_rev          | 4344 MB | 
     public | release_rev_pmcid_idx            | index | postgres | release_rev          | 3146 MB | 
     public | release_rev_pmid_idx             | index | postgres | release_rev          | 3271 MB | 
     public | release_rev_wikidata_idx         | index | postgres | release_rev          | 3223 MB | 
     public | release_rev_work_idx             | index | postgres | release_rev          | 4344 MB | 
    (12 rows)

That's 3+ GByte for indices with relatively few values. Potentially 12-15 GByte
of savings, even accounting for the fact that the extid table would grow by
several GB.

To do this, would need to do something like:

- have fatcatd to write into both rows and extid table on insert/update
- iterate over all all `release_rev` rows, and for each of these columns insert
  into the `ext_id` table if the value is set
- update fatcatd to read from extid table, and stop inserting these columns in
  rev table
- update schema to drop old indices and columns

This is non-trivial development and operational work; probably a good week for
bnewbold if all goes well?


## Idea: drop contrib and reference entity indices

The contribs table has a foreign key to the creator table. The release table as
a foreign key to the release table. Both of these have "reverse" indices,
allowing things like "all papers for creator" and "all releases referencing
this one". Neither of these are *really* necessary; they could be offloaded to
the search index (would, of course, increase that index size).

    fatcat_prod=# \di+ release_contrib*
                                        List of relations
     Schema |            Name             | Type  |  Owner   |      Table      | Size  | Description 
    --------+-----------------------------+-------+----------+-----------------+-------+-------------
     public | release_contrib_creator_idx | index | postgres | release_contrib | 11 GB | 
     public | release_contrib_pkey        | index | postgres | release_contrib | 11 GB | 
     public | release_contrib_rev_idx     | index | postgres | release_contrib | 15 GB | 
    (3 rows)


    fatcat_prod=# \di+ release_ref*
                                            List of relations
     Schema |              Name              | Type  |  Owner   |    Table    |  Size   | Description 
    --------+--------------------------------+-------+----------+-------------+---------+-------------
     public | release_ref_pkey               | index | postgres | release_ref | 2373 MB | 
     public | release_ref_target_release_idx | index | postgres | release_ref | 1843 MB | 
    (2 rows)

Looks like about 13 GByte of index could be saved here, or around 4% of total
release disk utilization.

## Idea: re-order table columns

I've read that there can be some data storage savings by moving fields which
are often null to the end of table schemas.

Need to research/calculate what savings might be. Probably small, but maybe
significant for edit tables.

Not sure how implementation would work... dump data values (plain or custom
`pg_dump`?) then reload, probably.

## Idea: `release_rev` and `file_rev` BIGINT for sub-tables

`_rev` table primary keys are UUIDs (16 bytes). All child tables (eg,
`release_contrib`, `file_rev_url`) have both foreign keys and indices on those
foreign keys to this table.

We could add new BIGINT secondary identifiers to the `_rev` tables (no index
needed), and switch all the child tables to use that. Because the child tables
are usually much larger than the `_rev` tables, this could save a lot of data
and index space.

It is a complex change, especially if it was only for `release`/`file`.

## Idea: Refactor `_edit` tables

Feels like there might be a bunch of waste here. Could make `{editgroup,
ident}` the primary key, which would remove need for a separate identifier and
index (we already have a UNIQ constraint on this).

    fatcat_prod=# \di+ work*
                                              List of relations
     Schema |                Name                 | Type  |  Owner   |   Table    |  Size   | Desc
    --------+-------------------------------------+-------+----------+------------+---------+-----
     public | work_edit_editgroup_id_ident_id_key | index | postgres | work_edit  | 6674 MB | 
     public | work_edit_ident_idx                 | index | postgres | work_edit  | 4233 MB | 
     public | work_edit_pkey                      | index | postgres | work_edit  | 4233 MB | 
     public | work_ident_pkey                     | index | postgres | work_ident | 4233 MB | 
     public | work_ident_redirect_idx             | index | postgres | work_ident | 3014 MB | 
     public | work_ident_rev_idx                  | index | postgres | work_ident | 4233 MB | 
     public | work_rev_pkey                       | index | postgres | work_rev   | 4233 MB | 
    (7 rows)

For the example of work edits (13 GB data, 20 GB index, 33 GB total), this
would drop ~20% of data size and ~20% of index size.

Would it make more sense to use {ident, editgroup} as the primary key and UNIQ,
then have a separate index on `editgroup`? On the assumption that `editgroup`
cardinality is much smaller, thus the index disk usage would be smaller.
