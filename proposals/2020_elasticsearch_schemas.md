
status: planning

This document tracks "easy" elasticsearch schema and behavior changes that
could be made while being backwards compatible with the current v0.3 schema and
not requiring any API/database schema changes.

## Release Field Additions

Simple additions:

- volume
- issue
- pages
- `first_page` (parsed from pages) (?)
- number
- `in_shadow`
- OA license slug (?)
- `doi_prefix`
- `doi_registrar` (based on extra)

"Array" keyword types for reverse lookups:

- referenced releases idents
- contrib creator idents


## Preservation Summary Field

To make facet/aggregate queries easier, propose summarizing the preservation
status (from `in_kbart`, `in_ia`, etc) to a `preservation_status` flag which
is:

- `bright`
- `dark_only`
- `shadow_only`
- `none`

Note that these don't align with OA color or work-level preservation (aka, no
"green"), it is a release-level status.

Filters like "papers only", "published only", "not stub", "single container"
would be overlaid in queries.


## OA Color Summary Field

Might not be ready for this yet, but for both releases and containers may be
able to do a better job of indicating OA status/policy for published works.

Not clear if this should be for "published" only, or whether we should try to
handle embargo time spans and dates.


## Release Merged Default Field

A current issue with searches is that term queries only match on a single
field, unless alternative fields are explicitly indicated. This breaks obvious
queries like "principa newton" which include both title terms and author terms,
or "coffee death bmj" which include a release title and journal title.

A partial solution to this is to index a field with multiple fields "copied"
into it, and have that be the default for term queries.

Fields to copy in include at least:

- `title`
- `subtitle`
- `original_title`
- `container_name`
- names of all authors (maybe not acronyms?)

May also want to include volume, issue, year, and any container acronyms or
aliases. If we did that, users could paste in citations and there is a better
chance the best match would be the exact cited paper.

This should be a pretty simple change. The biggest downside will be larger (up
to double?) index size.


## Partial Query Parsing

At some point we may want to build a proper query parser (see separate
proposal), but in the short term there is some low-hanging fruit simple token
parsing and re-writing we could do.

- strings like `N/A` which are parse bugs; auto-quote these
- pasting/searching for entire titles which include a word then colon ("Yummy
  Food: A Review"). We can detect that "food" is not a valid facet, and quote
  that single token
- ability to do an empty search (to get total count) (?)

This would require at least a simple escaped quotes tokenizer.


## Basic Filtering

This would be in the user interface, not schema.

At simple google-style filtering in release searches like:

- time span (last year, last 5, last 20, all)
- fulltext availability
- release type; stage; withdrawn
- language
- country/region

For containers:

- is OA
- stub (zero releases)

## Work Grouping

Release searches can be "grouped by" work identifier in the default responses,
to prevent the common situation where there are multiple release which are just
different versions of the same work.

Need to ensure this is performant.

Would need to update query UI/UX to display another line under hits ("also XYZ
other copies {including retraction or update} {having fulltext if this
hit does not}").


## Container Fields

- `all_issns`
- `release_count`

The `release_count` would not be indexed (left null) by default, and would be
"patched" in to entities by a separate script (periodically?).


## Container Copied Fields

Like releases, container entities could have a merged biblio field to use as
default in term queries:

- `name`
- `original_name`
- `aliases` (in extra?)
- `publisher`

Maybe also language and country names?


## Background Reading

"Which Academic Search Systems are Suitable for Systematic Reviews or
Meta-Analyses?  Evaluating Retrieval Qualities of Google Scholar, PubMed and 26
other Resources"

https://musingsaboutlibrarianship.blogspot.com/2019/12/the-rise-of-open-discovery-indexes.html

"Scholarly Search Engine Comparison"
https://docs.google.com/spreadsheets/d/1ZiCUuKNse8dwHRFAyhFsZsl6kG0Fkgaj5gttdwdVZEM/edit#gid=1016151070
