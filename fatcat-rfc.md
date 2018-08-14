
fatcat Design Document (RFC)
------------------------------

*Contact: Bryan Newbold <bnewbold@archive.org>. Last updated 2018-08-10*

fatcat is a proposed open bibliographic catalog of written works.  The scope of
works is somewhat flexible, with a focus on published research outputs like
journal articles, pre-prints, and conference proceedings. Records are
collaboratively editable, versioned, available in bulk form, and include
URL-agnostic file-level metadata.

fatcat is currently used internally at the Internet Archive, but interested
folks are welcome to contribute to design and development.

## Goals and Ecosystem Niche

For the Internet Archive use case, fatcat has two primary use cases:

- Track the "completeness" of our holdings against all known published works.
  In particular, allow us to monitor and prioritize further collection work.
- Be a public-facing catalog and access mechanism for our open access holdings.

In the larger ecosystem, fatcat could also provide:

- A work-level (as opposed to title-level) archival dashboard: what fraction of
  all published works are preserved in archives? KBART, CLOCKSS, Portico, and
  other preservations don't provide granular metadata
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

## Technical Architecture

The canonical backend datastore exposes a microservice-like HTTP API, which
could be extended with gRPC or GraphQL interfaces. The initial datastore is a
transactional SQL database, but this implementation detail is abstracted by the
API.

As little "application logic" as possible should be embedded in this back-end;
as much as possible would be pushed to bots which could be authored and
operated by anybody. A separate web interface project talks to the API backend
and can be developed more rapidly with less concern about data loss or
corruption.

A cronjob will creae periodic database dumps, both in "full" form (all tables
and all edit history, removing only authentication credentials) and "flattened"
form (with only the most recent version of each entity).

A goal is to be linked-data/RDF/JSON-LD/semantic-web "compatible", but not
necessarily "first". It should be possible to export the database in a
relatively clean RDF form, and to fetch data in a variety of formats, but
internally fatcat will not be backed by a triple-store, and will not be bound
to a rigid third-party ontology or schema.

Microservice daemons should be able to proxy between the primary API and
standard protocols like ResourceSync and OAI-PMH, and third party bots could
ingest or synchronize the databse in those formats.

## Licensing

The core fatcat database should only contain verifiable factual statements
(which isn't to say that all statements are "true"), not creative or derived
content.

The goal is to have a very permissively licensed database: CC-0 (no rights
reserved) if possible. Under US law, it should be possible to scrape and pull
in factual data from other corpuses without adopting their licenses. The goal
here isn't to avoid attribution (progeny information will be included, and a
large sources and acknowledgments statement should be maintained and shipped
with bulk exports), but trying to manage the intersection of all upstream
source licenses seems untenable, and creates burdens for downstream users and
developers.

Special care will need to be taken around copyright, "original work" by
editors, and contributions that raise privacy concerns.  If abstracts are
stored at all, they should be in a partitioned database table to prevent
copyright contamination. Likewise, even simple user-created content like lists,
reviews, ratings, comments, discussion, documentation, etc., should live in
separate services.

## Basic Editing Workflow and Bots

Both human editors and bots should have edits go through the same API, with
humans using either the default web interface, integrations, or client
software.

The normal workflow is to create edits (or updates, merges, deletions) on
individual entities. Individual changes are bundled into an "edit group" of
related edits (eg, correcting authorship info for multiple works related to a
single author). When ready, the editor would "submit" the edit group for
review. During the review period, human editors vote and bots can perform
automated checks. During this period the editor can make tweaks if necessary.
After some fixed time period (72 hours?) with no changes and no blocking
issues, the edit group would be auto-accepted if no merge conflicts have
be created by other edits to the same entities. This process balances editing
labor (reviews are easy, but optional) against quality (cool-down period makes
it easier to detect and prevent spam or out-of-control bots). More
sophisticated roles and permissions could allow some certain humans and bots to
push through edits more rapidly (eg, importing new works from a publisher API).

Bots need to be tuned to have appropriate edit group sizes (eg, daily batches,
instead of millions of works in a single edit) to make human QA review and
reverts managable.

Data progeny and source references are captured in the edit metadata, instead
of being encoded in the entity data model itself. In the case of importing
external databases, the expectation is that special-purpose bot accounts
are be used, and tag timestamps and external identifiers in the edit metadata.
Human editors would leave edit messages to clarify their sources.

A style guide (wiki) and discussion forum would be hosted as separate
stand-alone services for editors to propose projects and debate process or
scope changes. These services should have unified accounts and logins (oauth?)
to have consistent account IDs across all mediums.

## Global Edit Changelog

As part of the process of "accepting" an edit group, a row would be written to
an immutable, append-only log table (which internally could be a SQL table)
documenting each identifier change. This changelog establishes a monotonically
increasing version number for the entire corpus, and should make interaction
with other systems easier (eg, search engines, replicated databases,
alternative storage backends, notification frameworks, etc.).

## Identifiers

A fixed number of first-class "entities" are defined, with common behavior and
schema layouts. These are all be semantic entities like "work", "release",
"container", and "creator".

fatcat identifiers are semantically meaningless fixed-length random numbers,
usually represented in case-insensitive base32 format. Each entity type has its
own identifier namespace.

128-bit (UUID size) identifiers encode as 26 characters (but note that not all
such strings decode to valid UUIDs), and in the backend can be serialized in
UUID columns:

    work_rzga5b9cd7efgh04iljk8f3jvz
    https://fatcat.wiki/work/rzga5b9cd7efgh04iljk8f3jvz

In comparison, 96-bit identifiers would have 20 characters and look like:

    work_rzga5b9cd7efgh04iljk
    https://fatcat.wiki/work/rzga5b9cd7efgh04iljk

A 64-bit namespace would probably be large enought, and would work with
database Integer columns:

    work_rzga5b9cd7efg
    https://fatcat.wiki/work/rzga5b9cd7efg

The idea would be to only have fatcat identifiers be used to interlink between
databases, *not* to supplant DOIs, ISBNs, handle, ARKs, and other "registered"
persistent identifiers.

## Entities and Internal Schema

Internally, identifiers would be lightweight pointers to "revisions" of an
entity. Revisions are stored in their complete form, not as a patch or
difference; if comparing to distributed version control systems, this is the
git model, not the mercurial model.

The entity revisions are immutable once accepted; the editting process involves
the creation of new entity revisions and, if the edit is approved, pointing the
identifier to the new revision. Entities cross-reference between themselves by
*identifier* not *revision number*. Identifier pointers also support
(versioned) deletion and redirects (for merging entities).

Edit objects represent a change to a single entity; edits get batched together
into edit groups (like "commits" and "pull requests" in git parlance).

SQL tables would probably look something like the (but specific to each entity
type, with tables like `work_revision` not `entity_revision`):

    entity_ident
        id (uuid)
        current_revision (entity_revision foreign key)
        redirect_id (optional; points to another entity_ident)

    entity_revision
        revision_id
        <entity-specific fields>
        extra: json blob for schema evolution

    entity_edit
        timestamp
        editgroup_id
        ident (entity_ident foreign key)
        new_revision (entity_revision foreign key)
        previous_revision (optional; points to entity_revision)
        extra: json blob for progeny metadata

    editgroup
        editor_id
        description
        extra: json blob for progeny metadata

Additional entity-specific columns would hold actual metadata. Additional
tables (which would reference both `entity_revision` and `entity_id` foreign
keys as appropriate) would represent things like authorship relationships
(creator/release), citations between works, etc. Every revision of an entity
would require duplicating all of these associated rows, which could end up
being a large source of inefficiency, but is necessary to represent the full
history of an object.

## Scope

The goal is to capture the "scholarly web": the graph of written works that
cite other works. Any work that is both cited more than once and cites more
than one other work in the catalog is very likely to be in scope. "Leaf nodes"
and small islands of intra-cited works may or may not be in scope.

Overall focus is on written works, with some exceptions. The expected core
focus (for which we would pursue "completeness") is:

    journal articles
    academic books
    conference proceedings
    technical memos
    dissertations
    monographs
    well-researched blog posts
    web pages (that have citations)
    "white papers"

Possibly in scope:

    reports
    magazine articles
    essays
    notable mailing list postings
    government documents
    presentations (slides, video)
    datasets
    well-researched wiki pages
    patents

Probably not:

    court cases and legal documents
    newspaper articles
    social media
    manuals
    datasheets
    courses
    published poetry

Definitely not:

    audio recordings
    tv show episodes
    musical scores
    advertisements

Author, citation, and work disambiguation would be core tasks. Linking
pre-prints to final publication is in scope.

I'm much less interested in altmetrics, funding, and grant relationships than
most existing databases in this space.

fatcat would not include any fulltext content itself, even for cleanly licensed
(open access) works, but would have "strong" (verified) links to fulltext
content, and would include file-level metadata (like hashes and fingerprints)
to help discovery and identify content from any source. File-level URLs with
context ("repository", "author-homepage", "web-archive") should make fatcat
more useful for both humans and machines to quickly access fulltext content of
a given mimetype than existing redirect or landing page systems. So another
factor in deciding scope is whether a work has "digital fixity" and can be
contained in a single immutable file.

## Ontology

Loosely following FRBR (Functional Requirements for Bibliographic Records), but
removing the "manifestation" abstraction, and favoring files (digital
artifacts) over physical items, the primary entities are:

    work
        <a stub, for grouping releases>

    release (aka "edition", "variant")
        title
        volume/pages/issue/chapter
        media/formfactor
        publication/peer-review status
        language
        <published> date
        <variant-of> work
        <published-in> container
        <has-contributors> creator
        <citation-to> release
        <has> identifier

    file (aka "digital artifact")
        <instantiates> release
        hashes/checksums
        mimetype
        <found-at> URLs

    creator (aka "author")
        name
        identifiers
        aliases

    container (aka "venue", "serial", "title")
        name
        open-access policy
        peer-review policy
        <has> aliases, acronyms
        <about> subject/category
        <has> identifier
        <published-in> container
        <published-by> publisher

## Controlled Vocabularies

Some special namespace tables and enums would probably be helpful; these could
live in the database (not requiring a database migration to update), but should
have more controlled editing workflow... perhaps versioned in the codebase:

- identifier namespaces (DOI, ISBN, ISSN, ORCID, etc; but not the identifers
  themselves)
- subject categorization
- license and open access status
- work "types" (article vs. book chapter vs. proceeding, etc)
- contributor types (author, translator, illustrator, etc)
- human languages
- file mimetypes

These could also be enforced by QA bots that review all editgroups.

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
(statements) to specific sources unambiguously. Potential advantages fatcat
would have would be a focus on a specific scope (not a general-purpose database
of entities) and a goal of completeness (capturing as many works and
relationships as rapidly as possible). However, it might be better to just
pitch in to the wikidata efforts.

The technical design of fatcat is loosely inspired by the git
branch/tag/commit/tree architecture, and specifically inspired by Oliver
Charles' "New Edit System" [blog posts][nes-blog] from 2012.

There are a whole bunch of proprietary, for-profit bibliographic databases,
including Web of Science, Google Scholar, Microsoft Academic Graph, aminer,
Scopus, and Dimensions. There are excellent field-limited databases like dblp,
MEDLINE, and Semantic Scholar. There are some large general-purpose databases
that are not directly user-editable, including the OpenCitation corpus, CORE,
BASE, and CrossRef. I don't know of any large (more than 60 million works),
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

## RFC Changelog

- **2017-12-16**: early notes
- **2018-01-17**: initial RFC document
- **2018-08-10**: updates from implementation work
