# Roadmap

## Unresolved Questions

How to handle translations of, eg, titles and author names? To be clear, not
translations of works (which are just separate releases), these are more like
aliases or "originally known as".

Are bi-directional links a schema anti-pattern? Eg, should "work" point to a
"primary release" (which itself points back to the work)?

Should `identifier` and `citation` be their own entities, referencing other
entities by UUID instead of by revision? Not sure if this would increase or
decrease database resource utilization.

Should contributor/author affiliation and contact information be retained? It
could be very useful for disambiguation, but we don't want to build a huge
database for spammers or "innovative" start-up marketing.

Can general-purpose SQL databases like Postgres or MySQL scale well enough to
hold several tables with billions of entity revisions? Right from the start
there are hundreds of millions of works and releases, many of which having
dozens of citations, many authors, and many identifiers, and then we'll have
potentially dozens of edits for each of these, which multiply out to `1e8 * 2e1
* 2e1 = 4e10`, or 40 billion rows in the citation table. If each row was 32
bytes on average (uncompressed, not including index size), that would be 1.3
TByte on its own, larger than common SSD disks. I do think a transactional SQL
datastore is the right answer. In my experience locking and index rebuild times
are usually the biggest scaling challenges; the largely-immutable architecture
here should mitigate locking. Hopefully few indexes would be needed in the
primary database, as user interfaces could rely on secondary read-only search
engines for more complex queries and views.

I see a tension between focus and scope creep. If a central database like
fatcat doesn't support enough fields and metadata, then it will not be possible
to completely import other corpuses, and this becomes "yet another" partial
bibliographic database. On the other hand, accepting arbitrary data leads to
other problems: sparseness increases (we have more "partial" data), potential
for redundancy is high, humans will start editing content that might be
bulk-replaced, etc.

There might be a need to support "stub" references between entities. Eg, when
adding citations from PDF extraction, the cited works are likely to be
ambiguous. Could create "stub" works to be merged/resolved later, or could
leave the citation hanging. Same with authors, containers (journals), etc.
