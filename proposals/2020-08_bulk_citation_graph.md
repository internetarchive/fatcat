
status: mostly implemented (refcat, mostly)

Bulk Citation Graph
===================

This is one design proposal for how to scale up citation graph potential-match
generation, as well as for doing fuzzy matching of other types at scale (eg,
self-matches to group works within fatcat). Not proposiing that we have to do
things this way, this is just one possible option.

This current proposal has the following assumptions:

- 100-200 million "target" works
- 1-3 billion structured references to match
- references mostly coming from GROBID, Crossref, and PUBMED
- paper-like works (could include books; references to web pages etc to be
  handled separately)
- static snapshots are sufficient
- python fuzzy match code works and is performant enough to verify matches
  within small buckets/pools

Additional major "source" and "target" works to think about:

- wikipedia articles as special "source" works. should work fine with this
  system. also most wikipedia paper references already have persistent
  identifiers
- webpages as special "target" works, where we would want to do a CDX lookup or
  something. normalize URL (SURT?) to generate reverse index ("all works citing
  a given URL")
- openlibrary books as "target" works. also should work fine with this system

The high-level prosposal is:

- transform and normalize basic metadata for both citations and reference (eg,
  sufficient fields for fuzzy verification), and store only this minimal subset
- as a first pass, if external identifiers exist in the "source" reference set,
  do lookups against fatcat API and verify match on any resulting hits. remove
  these "source" matches from the next stages.
- generate one or more fixed-size hash identifiers (~64 bit) for each citation
  and target, and use these as a key to bucket works. this would not be hashes
  over the entire record metadata, only small subsets
- sort the "target" works into an index for self-grouping, lookups, and
  iteration. each record may appear multiple times if there are multiple hash
  types
- sort the "source" references into an index and run a merge-sort on bucket
  keys against the "target" index to generate candidate match buckets
- run python fuzzy match code against the candidate buckets, outputting a status
  for each reference input and a list of all strong matches
- resort successful matches and index by both source and target identifiers as
  output citation graph

## Record Schema

Imaginging a subset of fatcat release entity fields, perhaps stored in a binary
format like protobuf for size efficiency. Or a SQL table or columnar
datastore. If we used JSON we would want to use short key names to reduce key
storage overhead. Total data set size will impact performance because of disk
I/O, caching, etc. I think this may hold even with on-disk compression?

Would do a pass of normalization ahead of time, like aggressive string
cleaning, so we don't need to do this per-fuzzy-verify attempt.

Metadata subset might include:

- `title`
- `subtitle`
- `authors` (surnames? structured/full names? original/alternate/aliases?)
- `year`
- `container_name` (and abbreviation?)
- `volume`, `issue`, `pages`
- `doi`, `pmid`, `arxiv_id` (only ext ids used in citations?)
- `release_ident` (source or target)
- `work_ident` (source or target)
- `release_stage` (target only?)
- `release_type` (target only?)

Plus any other fields helpful in fuzzy verification.

These records can be transformed into python release entities with partial
metadata, then passed to the existing fuzzy verification code path.

## Hashing Schemes

Hashing schemes could be flexible. Multiple could be used at the same time, and
we can change schemes over time. Each record could be transformed to one or
more hashes. Ideally we could use the top couple bits of the hash to indicate
the hash type.

An initial proposal would be to use first and last N tokens of just the title.
In this scheme would normalize and tokenize the title, remove a few stopwords
(eg, tokens sometimes omitted in citation or indexing). If the title is shorter
than 3 tokens pad with blank tokens. Perhaps do a filter here against
inordinately popular titles or other bad data. Then use some fast hash
non-cryptographic hash with fixed size output (64-bits). Do this for both the
first and last three tokens; set the top bit to "0" for hash of the first three
tokens, or "1" for the hash of the last three tokens. Emit two key/value rows
(eg, TSV?), with the same values but different hashes.

Alternatively, in SQL, index a single row on the two different hash types.

Possible alternative hash variations we could experiment with:

- take the first 10 normalized characters, removing whitespace, and hash that
- include first 3 title tokens, then 1 token of the first author's surname
- normalize and hash entire title
- concatenate subtitle to title or not

Two advantages of hashing are:

- we can shard/partition based on the key. this would not be the case if the
  keys were raw natural language tokens
- advantages from fixed-size datatypes (eg, uint64)

## Bulk Joining

"Target" index could include all hash types in a single index. "Source" index
in bulk mode could be either all hash types concatenated together and run
together, then re-sort and uniq the output (eg, by release-to-release pairings)
to remove dupes. In many cases this would have the overhead of computing the
fuzzy verification multiple times redundantly (but only a small fixed maximum
number of duplicates). Alternatively, with greater pipeline complexity, could
do an initial match on one hash type, then attempt matching (eg, recompute and
sort and join) for the other hash types only for those which did not match.

## Citation Graph Index Output

Imagining successful match rows to look like:

- `match_status` (eg, strong/weak)
- `source_release_ident`
- `source_release_stage`
- `ref_key` (optional? or `ref_index`?)
- `source_work_ident`
- `target_release_ident`
- `target_release_stage`
- `target_work_ident`

Would run a sort/uniq on `(source_release_ident,target_release_ident)`.

Could filter by stages, then sort/uniq work-to-work counts to generate simple
inbound citation counts for each target work.

Could sort `target_work_ident` and generate groups of inbound works ("best
release per work") citing that work. Then do fast key lookups to show
"works/releases citing this work/release".

## To Be Decided

- bulk datastore/schema: just TSV files sorted by key column? if protobuf, how
  to encode? what about SQL? parquet/arrow?
- what datastores would allow fast merge sorts? do SQL engines (parquet)
  actually do this?
- would we need to make changes to be more compatible with something like
  opencitations? Eg, I think they want DOI-to-DOI citations; having to look
  those up again from fatcat API would be slow
- should we do this in a large distributed system like spark (maybe pyspark for
  fuzzy verification) or stick to simple UNIX/CLI tools?
- wikipedia articles as sources?
- openlibrary identifiers?
- archive.org as additional identifiers?

