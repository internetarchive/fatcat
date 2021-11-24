
status: planning

This proposal tracks a batch of catalog metadata cleanups planned for 2020.


## File Hash Duplication

There are at least a few dozen file entities with duplicate SHA-1.

These should simply be merged via redirect. This is probably the simplest
cleanup case, as the number of entities is low and the complexity of merging
metadata is also low.


## Release Identifier (DOI, PMID, PMCID, arxiv) Duplication

At least a few thousand DOIs (some from Datacite import due to normalization
behavior, some from previous Crossref issues), hundreds of thousands of PMIDs,
and an unknown number of PMCIDs and arxiv ids have duplicate releases. This
means, multiple releases exist with the same external identifier.

The cleanup is same as with file hashes: the duplicate releases and works
should be merged (via redirects).

TODO: It is possible that works should be deleted instead of merged.


## PDF File Metadata Completeness

All PDF files should be "complete" over {SHA1, SHA256, MD5, size, mimetype},
all of which metadata should be confirmed by calculating the values directly
from the file.

A good fraction of file entities have metadata from direct CDX imports, which
did not include (uncompressed) size, hashes other than SHA-1, or confirmed
mimetype. Additionally, the SHA-1 itself is not accurate for the "inner" file
in a fraction of cases (at least thousands of files, possibly 1% or more) due
to CDX/WARC behavior with transport compressed bodies (where the recorded SHA-1
is of the compressed body, not the actual inner file).


## File URL Cleanups

The current file URL metadata has a few warts:

- inconsistent or incorrect tagging of URL "rel" type. It is possible we should
  just strip/skip this tag and always recompute from scratch. Or target just
  those domains with >= 1% of links, or top 100 domains
- duplicate URLs (lack of normalization):
    - `http://example.com/file.pdf`
    - `http://example.com:80/file.pdf`
    - `https://example.com/file.pdf`
    - `http://www.example.com/file.pdf`
- URLs with many and long query parameters, such as `jsessionid` or AWS token
  parameters. These are necessary in wayback URLs (for replay), but meaningless
  and ugly as regular URLs
- possibly some remaining `https://web.archive.org/web/None/...` URLs, which
  at best should be replaced with the actual capture timestamp or at least
  deleted
- some year-only wayback links (`https://web.archive.org/web/2016/...`)
  basically same as `None`
- many wayback links per file

Some of these issues are partially user-interface driven. There is also a
balance between wanting many URLs (and datetimes for wayback URLs) for
diversity and as an archival signal, but there being diminishing returns for
this kind of completeness.

I would propose that one URL per host and the oldest wayback link per host and
transport (treating http/https as same transport type, but ftp as distinct) is
a reasonable constraint, but am open to other opinions. I think that all web
URLs should be normalized for issues like `jsessionid` and `:80` port
specification.

In user interface we should limit to a single wayback link, and single link per
domain.

NOTE: "host" means the fully qualified domain hostname; domain means the
"registered" part of the domain.


## Container Metadata

At some point, had many "NULL" publishers.

"NA" in ISSNe, ISSNp. Eg: <https://fatcat.wiki/container/s3gm7274mfe6fcs7e3jterqlri>

"Type" coverage should be improved.

"Publisher type" (inferred in various ways in chocula tool) could be included in
`extra` and end up in search faceting.

Overall OA status should probably be more sophisticated: gold, green, etc.


## Stub Hunting

There are a lot of release entities which should probably be marked `stub` or
in some other way indicated as unimportant or other (see also proposal to add
new `release_types`). The main priority is to change the type of releases that
are currently `published` and "paper-like", thus showing up in coverage stats.

A partial list:

- bad/weird titles
    - "[Blank page]"
    - "blank page"
    - "Temporary Empty DOI 0"
    - "ADVERTISEMENT"
    - "Full title page with Editorial board (with Elsevier tree)"
    - "Advisory Board Editorial Board"


## Very Long Titles

These are likely stubs, but the title is also "just too long". Could stash full
title in `extra`?

- https://fatcat.wiki/release/4b7swn2zsvguvkzmt
    => crossref updated

## Abstracts

Bad:

- https://qa.fatcat.wiki/release/nwd5kkilybf5vdhm3iduvhvbvq
- https://qa.fatcat.wiki/release/rkigixosmvgcvmlkb5aqeyznim

Very long:

- https://qa.fatcat.wiki/release/s2cafgwepvfqnjp4xicsx6amsa

