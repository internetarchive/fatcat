
## Project Goals and Ecosystem Niche

The Internet Archive has two primary use cases for fatcat:

- Tracking the "completeness" of our holdings against all known published
  works.  In particular, allow us to monitor progress, identify gaps, and
  prioritize further collection work.
- Be a public-facing catalog and access mechanism for our open access holdings.

In the larger ecosystem, fatcat could also provide:

- A work-level (as opposed to title-level) archival dashboard: what fraction of
  all published works are preserved in archives? [KBART](), [CLOCKSS](),
  [Portico](), and other preservations don't provide granular metadata
- A collaborative, independent, non-commercial, fully-open, field-agnostic,
  "completeness"-oriented catalog of scholarly metadata
- Unified (centralized) foundation for discovery and access across repositories
  and archives: discovery projects can focus on user experience instead of
  building their own catalog from scratch
- Research corpus for meta-science, with an emphasis on availability and
  reproducibility (metadata corpus itself is open access, and file-level hashes
  control for content drift)
- Foundational infrastructure for distributed digital preservation
- On-ramp for non-traditional digital works ("grey literature") into the
  scholarly web

[KBART]: https://thekeepers.org/
[CLOCKSS]: https://clockss.org
[Portico]: http://www.portico.org

## Scope

What types of works should be included in the catalog?

The goal is to capture the "scholarly web": the graph of written works that
cite other works. Any work that is both cited more than once and cites more
than one other work in the catalog is very likely to be in scope. "Leaf nodes"
and small islands of intra-cited works may or may not be in scope.

Fatcat does not include any fulltext content itself, even for cleanly licensed
(open access) works, but does have "strong" (verified) links to fulltext
content, and includes file-level metadata (like hashes and fingerprints)
to help discovery and identify content from any source. File-level URLs with
context ("repository", "author-homepage", "web-archive") should make fatcat
more useful for both humans and machines to quickly access fulltext content of
a given mimetype than existing redirect or landing page systems. So another
factor in deciding scope is whether a work has "digital fixity" and can be
contained in a single immutable file.

## References and Previous Work

The closest overall analog of fatcat is [MusicBrainz][mb], a collaboratively
edited music database. [Open Library][ol] is a very similar existing service,
which exclusively contains book metadata.

[Wikidata][wd] seems to be the most successful and actively edited/developed
open bibliographic database at this time (early 2018), including the
[wikicite][wikicite] conference and related Wikimedia/Wikipedia projects.
Wikidata is a general purpose semantic database of entities, facts, and
relationships; bibliographic metadata has become a large fraction of all
content in recent years. The focus there seems to be linking knowledge
(statements) to specific sources unambiguously. Potential advantages fatcat has
are a focus on a specific scope (not a general-purpose database of entities)
and a goal of completeness (capturing as many works and relationships as
rapidly as possible). With so much overlap, the two efforts might merge in the
future.

The technical design of fatcat is loosely inspired by the git
branch/tag/commit/tree architecture, and specifically inspired by Oliver
Charles' "New Edit System" [blog posts][nes-blog] from 2012.

There are a whole bunch of proprietary, for-profit bibliographic databases,
including Web of Science, Google Scholar, Microsoft Academic Graph, aminer,
Scopus, and Dimensions. There are excellent field-limited databases like dblp,
MEDLINE, and Semantic Scholar. There are some large general-purpose databases
that are not directly user-editable, including the OpenCitation corpus, CORE,
BASE, and CrossRef. We do not know of any large (more than 60 million works),
open (bulk-downloadable with permissive or no license), field agnostic,
user-editable corpus of scholarly publication bibliographic metadata.

[nes-blog]: https://ocharles.org.uk/blog/posts/2012-07-10-nes-does-it-better-1.html
[mb]: https://musicbrainz.org
[ol]: https://openlibrary.org
[wd]: https://wikidata.org
[wikicite]: https://meta.wikimedia.org/wiki/WikiCite_2017

## Further Reading

"From ISIS to CouchDB: Databases and Data Models for Bibliographic Records" by Luciano G. Ramalho. code4lib, 2013. <https://journal.code4lib.org/articles/4893>

"Representing bibliographic data in JSON". github README file, 2017. <https://github.com/rdmpage/bibliographic-metadata-json>

"Citation Style Language", <https://citationstyles.org/>

"Functional Requirements for Bibliographic Records", Wikipedia article, <https://en.wikipedia.org/wiki/Functional_Requirements_for_Bibliographic_Records>

OpenCitations and I40C <http://opencitations.net/>, <https://i4oc.org/>

