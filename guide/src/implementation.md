# Implementation

The canonical backend datastore exposes a microservice-like HTTP API, which
could be extended with gRPC or GraphQL interfaces. The initial datastore is a
transactional SQL database, but this implementation detail is abstracted by the
API.

As little "application logic" as possible should be embedded in this back-end;
as much as possible would be pushed to bots which could be authored and
operated by anybody. A separate web interface project talks to the API backend
and can be developed more rapidly with less concern about data loss or
corruption.

A cronjob will create periodic database dumps, both in "full" form (all tables
and all edit history, removing only authentication credentials) and "flattened"
form (with only the most recent version of each entity).

One design goal is to be linked-data/RDF/JSON-LD/semantic-web "compatible", but
not necessarily "first". It should be possible to export the database in a
relatively clean RDF form, and to fetch data in a variety of formats, but
internally Fatcat is not backed by a triple-store, and is not tied to any
specific third-party ontology or schema.

Microservice daemons should be able to proxy between the primary API and
standard protocols like ResourceSync and OAI-PMH, and third party bots can
ingest or synchronize the database in those formats.

### Fatcat Identifiers

Fatcat identifiers are semantically meaningless fixed-length random numbers,
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

and 64-bit:

    work_rzga5b9cd7efg
    https://fatcat.wiki/work/rzga5b9cd7efg

Fatcat identifiers can used to interlink between databases, but are explicitly
*not* intended to supplant DOIs, ISBNs, handle, ARKs, and other "registered"
persistent identifiers for general use.

### Internal Schema

Internally, identifiers are lightweight pointers to "revisions" of an entity.
Revisions are stored in their complete form, not as a patch or difference; if
comparing to distributed version control systems (for managing changes to
source code), this follows the git model, not the mercurial model.

The entity revisions are immutable once accepted; the editing process involves
the creation of new entity revisions and, if the edit is approved, pointing the
identifier to the new revision. Entities cross-reference between themselves by
*identifier* not *revision number*. Identifier pointers also support
(versioned) deletion and redirects (for merging entities).

Edit objects represent a change to a single entity; edits get batched together
into edit groups (like "commits" and "pull requests" in git parlance).

SQL tables look something like this (with separate tables for entity type a la
`work_revision` and `work_edit`):

    entity_ident
        id (uuid)
        current_revision (entity_revision foreign key)
        redirect_id (optional; points to another entity_ident)
        is_live (boolean; whether newly created entity has been accepted)

    entity_revision
        revision_id
        <all entity-style-specific fields>
        extra: json blob for schema evolution

    entity_edit
        timestamp
        editgroup_id (editgroup foreign key)
        ident (entity_ident foreign key)
        new_revision (entity_revision foreign key)
        new_redirect (optional; points to entity_ident table)
        previous_revision (optional; points to entity_revision)
        extra: json blob for provenance metadata

    editgroup
        editor_id (editor table foreign key)
        description
        extra: json blob for provenance metadata

An individual entity can be in the following "states", from which the given
actions (transition) can be made:

- `wip` (not live; not redirect; has rev)
    - activate (to `active`)
- `active` (live; not redirect; has rev)
    - redirect (to `redirect`)
    - delete (to `deleted`)
- `redirect` (live; redirect; rev or not)
    - split (to `active`)
    - delete (to `delete`)
- `deleted` (live; not redirect; no rev)
    - redirect (to `redirect`)
    - activate (to `active`)

"WIP, redirect" or "WIP, deleted" are invalid states.

Additional entity-specific columns hold actual metadata. Additional
tables (which reference both `entity_revision` and `entity_id` foreign
keys as appropriate) represent things like authorship relationships
(creator/release), citations between works, etc. Every revision of an entity
requires duplicating all of these associated rows, which could end up
being a large source of inefficiency, but is necessary to represent the full
history of an object.
