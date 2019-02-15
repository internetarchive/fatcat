# Roadmap

Core unimplemented features (as of February 2019) include:

- rate-limiting and spam/abuse mitigation
- actual entity creation, editing, deleting through the web interface

Contributions would be helpful to implement:

- import (bulk and/or continuous updates) for more metadata sources
- better handling of work/release distinction in, eg, search results and
  citation counting
- de-duplication (via merging) for all entity types
- matching improvements, eg, for references (citations), contributions
  (authorship), work grouping, and file/release matching
- internationalization of the web interface (translation to multiple languages)
- accessibility review of user interface

Longer term projects could include:

- full-text search over release files
- bi-directional synchronization with other user-editable catalogs, such as
  Wikidata
- alternate/enhanced backend to store full edit history without overloading
  traditional relational database

## Known Issues

- changelog index may have gaps due to PostgreSQL sequence and transaction
  roll-back behavior

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
