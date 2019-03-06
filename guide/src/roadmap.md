# Roadmap

Core unimplemented features (as of February 2019) include:

- rate-limiting and spam/abuse mitigation
- actual entity creation, editing, deleting through the web interface
- several web interface views (eg, editor-specific changelog, recent changes)
- "work aggolomeration", merging related releases under the same work
- linking known citations (we know DOI or PMID of "target", but haven't updated
  reference to point to fatcat ident)

Contributions would be helpful to implement:

- import (bulk and/or continuous updates) for more metadata sources
- better handling of work/release distinction in, eg, search results and
  citation counting
- de-duplication (via merging) for all entity types
- matching improvements, eg, for references (citations), contributions
  (authorship), work grouping, and file/release matching
- internationalization of the web interface (translation to multiple languages)
- accessibility review of user interface

Possible breaking API and schema changes:

- move all edit endpoints under `/editgroup/<editgroup_id>/...`, instead of
  having an `editgroup_id` query parameter
- rename `release_status` to `release_stage` 
- handle retractions/withdrawls with `widthdrawn` and `withdrawn_date` release
  fields, and `retracted` status
- new entity type for research institutions, to track author affiliation. Use
  the new (2019) ROR identifier/registry
- container nesting, or some method to handle conferences (event vs. series)
  and other "series" or "group" containers
- include more author name metadata (display, sur, given) in contribs, and
  potentially references. Need this to format citations properly (CSL) when we
  don't have full author linkage

Other longer term projects could include:

- full-text search over release files
- bi-directional synchronization with other user-editable catalogs, such as
  Wikidata
- alternate/enhanced backend to store full edit history without overloading
  traditional relational database
- make external identifiers generic, instead of having a fixed (indexed) list.
  Eg, extid table for every entity rev, with string ("issn:1234-5678") or
  structure ('{type: "issn", value: "1234-5678"}')
- URLs for entities. Have avoided so far, in lieu of external identifiers or
  web captures
- "save paper now" feature in web interface
- generic tagging of entities. Needs design/scoping; a separate service?
  editor-specific? tag by slugs, free-form text, or wikidata entities?
  "delicious for papers"?. Something as an alternative to traditional
  hierarchal categorization.
- first-class support for books: additional external identifiers, metadata
  tweaks, bulk import of MARC or other metadata records, matching to DOAB and
  other open-access book collections

## Known Issues

- changelog index may have gaps due to PostgreSQL sequence and transaction
  roll-back behavior
- search is idiosyncratic: does not cover contrib names by default, and some
  queries cause errors (eg, "N/A" without quotes)

## Unresolved Questions

How to handle translations of, eg, titles and author names? To be clear, not
translations of works (which are just separate releases), these are more like
aliases or "originally known as".

Should external identifers be made generic? Eg, instead of having `arxiv_id` as
a column, have a table of arbitary identifers, with either an `extid_type` or
just use a prefix like `arxiv:someid`.

Should contributor/author affiliation and contact information be retained? It
could be very useful for disambiguation, but we don't want to build a huge
database for "marketing" and other spam.

Can general-purpose SQL databases like Postgres or MySQL scale well enough to
hold several tables with billions of entity revisions? Right from the start
there are hundreds of millions of works and releases, many of which having
dozens of citations, many authors, and many identifiers, and then we'll have
potentially dozens of edits for each of these. This multiplies out to `1e8 * 2e1
* 2e1 = 4e10`, or 40 billion rows in the citation table. If each row was 32
bytes on average (uncompressed, not including index size), that would be 1.3
TByte on its own, larger than common SSD disks. I do think a transactional SQL
datastore is the right answer. In my experience locking and index rebuild times
are usually the biggest scaling challenges; the largely-immutable architecture
here should mitigate locking. Hopefully few indexes would be needed in the
primary database, as user interfaces could rely on secondary read-only search
engines for more complex queries and views.

There is a tension between focus and scope creep. If a central database like
Fatcat doesn't support enough fields and metadata, then it will not be possible
to completely import other corpuses, and this becomes "yet another" partial
bibliographic database. On the other hand, accepting arbitrary data leads to
other problems: sparseness increases (we have more "partial" data), potential
for redundancy is high, humans will start editing content that might be
bulk-replaced, etc.

There might be a need to support "stub" references between entities. Eg, when
adding citations from PDF extraction, the cited works are likely to be
ambiguous. Could create "stub" works to be merged/resolved later, or could
leave the citation hanging. Same with authors, containers (journals), etc.
