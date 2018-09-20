# Data Model

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

## Entity States

    wip (not live; not redirect; has rev)
      activate
    active (live; not redirect; has rev)
      redirect
      delete
    redirect (live; redirect; rev or not)
      split
      delete
    deleted (live; not redirect; no rev)
      redirect
      activate

    "wip redirect" or "wip deleted" are invalid states

## Global Edit Changelog

As part of the process of "accepting" an edit group, a row would be written to
an immutable, append-only log table (which internally could be a SQL table)
documenting each identifier change. This changelog establishes a monotonically
increasing version number for the entire corpus, and should make interaction
with other systems easier (eg, search engines, replicated databases,
alternative storage backends, notification frameworks, etc.).

