
fatcat is a half-baked idea to build an open, independent, collaboratively
editable bibliographic database of most written works, with a focus on
published research outputs like journal articles, pre-prints, and conference
proceedings.

## Technical Architecture

The canonical backend datastore would be a very large transactional SQL server.
A relatively simple and stable back-end daemon would expose an API (could be
REST, GraphQL, gRPC, etc). As little "application logic" as possible would be
embedded in this back-end; as much as possible would be pushed to bots which
could be authored and operated by anybody. A separate web interface project
would talk to the API backend and could be developed more rapidly.

A cronjob would make periodic database dumps, both in "full" form (all tables
and all edit history, removing only authentication credentials) and "flat" form
(with only the most recent version of each entity, using only persistent IDs
between entities).

A goal is to be linked-data/RDF/JSON-LD/semantic-web "compatible", but not
necessarily "first". It should be possible to export the database in a
relatively clean RDF form, and to fetch data in a variety of formats, but
internally fatcat would not be backed by a triple-store, and would not be
bound to a specific third party ontology or schema.

Microservice daemons should be able to proxy between the primary API and
standard protocols like ResourceSync and OAI-PMH, and bots could consume
external databases in those formats.

## Licensing

The core fatcat database should only contain verifyable factual statements
(which isn't to say that all statements are "true"), not creative or derived
content.

The goal is to have a very permissively licensed database: CC-0 (no rights
reserved) if possible. Under US law, it should be possible to scrape and pull
in factual data from other corpuses without adopting their licenses. The goal
here isn't to avoid all attibution (progeny information will be included, and a
large sources and acknowledgements statement should be maintained), but trying
to manage the intersection of all upstream source licenses seems untenable, and
creates burdens for downstream users.

Special care will need to be taken around copyright and original works. I would
propose either not accepting abstracts at all, or including them in a
partitioned database to prevent copyright contamination. Likewise, even simple
user-created content like lists, reviews, ratings, comments, discussion,
documentation, etc should go in separate services.

## Basic Editing Workflow and Bots

Both human editors and bots would have edits go through the same API, with
humans using either the default web interface or arbitrary integrations or
client software.

The usual workflow would be to create edits (or creations, merges, deletions)
to individual entities one at a time, all under a single "edit group" of
related edits (eg, correcting authorship info for multiple works related to a
single author). When ready, the editor would "submit" the edit group for
review. During the review period, humans could vote (or veto/approve if they
have higher permissions), and bots can perform automated checks. During this
period the editor can make tweaks if necessary. After some fixed time period
(72 hours?) with no changes and no blocking issues, the edit group would be
auto-accepted, if no auto-resolvable merge-conflicts have arisen. This process
balances editing labor (reviews are easy, but optional) against quality
(cool-down period makes it easier to detect and prevent spam or out-of-control
bots). Advanced permissions could allow some trusted human and bot editors to
push through edits more rapidly.

Bots would need to be tuned to have appropriate edit group sizes (eg, daily
batches, instead of millions of works in a single edit) to make human QA and
reverts possible.

Data progeny and citation would be left to the edit history. In the case of
importing external databases, the expectation would be that special-purpose
bot accounts would be used. Human editors would leave edit messages to clarify
their sources.

A style guide (wiki), chat room, and discussion forum would be hosted as
separate stand-alone services for editors to propose projects and debate
process or scope changes. It would be best if these could use federated account
authorization (oauth?) to have consistent account IDs across mediums.

## Edit Log

As part of the process of "accepting" an edit group, a row would be written to
an immutable, append-only log table (which internally could be a SQL table)
documenting each identifier change. This log establishes a monotonically
increasing version number for the entire corpus, and should make interaction
with other systems easier (eg, search engines, replicated databases,
alternative storage backends, notification frameworks, etc).

## Itentifiers

A fixed number of first class "entities" would be definied, with common
behavior and schema layouts. These would all be semantic entities like "work",
"release", "container", and "person".

fatcat identifiers would be semanticly meaningless fixed length random numbers,
usually represented in case-insensitive base32 format. Each entity type would
have it's own identifier namespace. Eg, 96 bit identifiers would have 26
characters and look like:

    fcwork_rzga5b9cd7efgh04iljk

As a URL:

    https://fatcat.org/work/rzga5b9cd7efgh04iljk

A 64 bit namespace is probably plenty though, and would work with most databse
Integer columns:

    fcwork_rzga5b9cd7efg
    https://fatcat.org/work/rzga5b9cd7efg

The idea would be to only have fatcat identifiers be used to interlink between
databases, *not* to supplant DOIs, ISBNs, handle, ARKs, and other "registered"
persistant identifiers.

## Entities and Internal Schema

Internally, identifiers would be lightweight pointers to actual metadata
objects, which can be thought of as "versions". The metadata objects themselves
would be immutable once commited; the edit process is one of creating new
objects and, if the edit is approved, pointing the identifier to the new
version. Entities would reference between themselves by identifier.

Edit objects represent a change to a single entity; edits get batched together
into edit groups (like "commits" and "pull requests" in git parlance).

SQL tables would probably look something like the following, though be specific
to each entity type (eg, there would be an actual `work_revision` table, but
not an actual `entity_revision` table):

    entity_id
        uuid
        current_revision

    entity_revision
        entity_id (bi-directional?)
        previous: entity_revision or none
        state: normal, redirect, deletion
        redirect_entity_id: optional
        extra: json blob
        edit_id

    edit
        mutable: boolean
        edit_group
        editor

    edit_group

Additional type-specific columns would hold actual metadata. Additional tables
(which would reference both `entity_revision` and `entity_id` foreign keys as
appropriate) would represent things like external identifiers, ordered
author/work relationships, citations between works, etc. Every revision of an
entity would require duplicating all of these associated rows, which could end
up being a large source of inefficiency, but is necessary to represent the full
history of an object.

## Scope

Want the "scholarly web": the graph of works that cite other works. Certainly
every work that is cited more than once and every work that both cites and is
cited; "leaf nodes" and small islands might not be in scope.

Focusing on written works, with some exceptions. Expect core media (for which we would pursue "completeness") to be:

    journal articles
    books
    conference proceedings
    technical memos
    dissertations

Probably in scope:

    reports
    magazine articles
    published poetry
    essays
    government documents
    conference
    presentations (slides, video)
    datasets

Probably not:

    patents
    court cases and legal documents
    manuals
    datasheets
    courses

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
to help discovery and identify content from any source. Typed file-level links
should make fatcat more useful for both humans and machines to quickly access
fulltext content of a given mimetype than existing redirect or landing page
systems.

## Ontology

Loosely following FRBR, but removing the "manifestation" abstraction, and
favoring files (digital artifacts) over physical items, the primary entities
are:

    work
        type
        <has> contributors
        <about> subject/category
        <has-primary> release

    release (aka "edition", "variant")
        title
        volume/pages/issue/chapter
        open-access status
        <published> date
        <of a> work
        <published-by> publisher
        <published in> container
        <has> contributors
        <citation> citetext <to> release
        <has> identifier

    file (aka "digital artifact")
        <of a> release
        <has> hashes
        <found at> URLs
        <held-at> institution <with> accession

    contributor
        name
        <has> aliases
        <has> affiliation <for> date span
        <has> identifier

    container
        name
        open-access policy
        peer-review policy
        <has> aliases, acronyms
        <about> subject/category
        <has> identifier
        <published in> container
        <published-by> publisher

    publisher
        name
        <has> aliases, acronyms
        <has> identifier

## Controlled Vocabularies

Some special namespace tables and enums would probably be helpful; these should
live in the database (not requiring a database migration to update), but should
have more controlled editing workflow... perhaps versioned in the codebase:

- identifier namespaces (DOI, ISBN, ISSN, ORCID, etc)
- subject categorization
- license and open access status
- work "types" (article vs. book chapter vs. proceeding, etc)
- contributor types (author, translator, illustrator, etc)
- human languages
- file mimetypes

## Unresolved Questions

How to handle translations of, eg, titles and author names? To be clear, not
translations of works (which are just separate releases).

Are bi-directional links a schema anti-pattern? Eg, should "work" point to a
primary "release" (which itself points back to the work), or should "release"
have a "is-primary" flag?

Should `identifier` and `citation` be their own entities, referencing other
entities by UUID instead of by revision? This could save a ton of database
space and chunder.

Should contributor/author contact information be retained? It could be very
useful for disambiguation, but we don't want to build a huge database for
spammers or "innovative" start-up marketing.

Would general purpose SQL databases like Postgres or MySQL scale well enough
told hold several tables with billions of entries? Right from the start there
are hundreds of millions of works and releases, many of which having dozens of
citations, many authors, and many identifiers, and then we'll have potentially
dozens of edits for each of these, which multiply out to `1e8 * 2e1 * 2e1 =
4e10`, or 40 billion rows in the citation table. If each row was 32 bytes on
average (uncompressed, not including index size), that would be 1.3 TByte on
it's own, larger than common SSD disk. I think a transactional SQL datastore is
the right answer. In my experience locking and index rebuild times are usually
the biggest scaling challenges; the largely-immutable architecture here should
mitigate locking. Hopefully few indexes would be needed in the primary
database, as user interfaces could rely on secondary read-only search engines
for more complex queries and views.

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
(statements) to specific sources unambigiously. Potential advantages fatcat
would have would be a focus on a specific scope (not a general purpose database
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

