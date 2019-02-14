# Data Model

## Entity Types and Ontology

Loosely following "Functional Requirements for Bibliographic Records" (FRBR),
but removing the "manifestation" abstraction, and favoring files (digital
artifacts) over physical items, the primary bibliographic entity types are:

- `work`: representing an abstract unit of creative output. Does not contain
  any metadata itself; used only to group `release` entities. For example, a
  journal article could be posted as a pre-print, published on a journal
  website, translated into multiple languages, and then re-published (with
  minimal changes) as a book chapter; these would all be variants of the same
  `work`.
- `release`: a specific "release" or "publicly published" (in a formal or
  informal sense) version of a work. Contains traditional bibliographic
  metadata (title, date of publication, media type, language, etc). Has
  relationships to other entities:
    - "variant of" a single `work`
    - "contributed to by" multiple `creators`
    - "references to" (cites) multiple `releases`
    - "published as part of" a single `container`
- `file`: a single concrete, fixed digital artifact; a manifestation of one or
  more `releases`. Machine-verifiable metadata includes file hashes, size, and
  detected file format. Verified URLs link to locations on the open web where
  this file can be found or has been archived. Has relationships:
    - "manifestation of" multiple `releases` (though usually a single release)
- `creator`: persona (pseudonym, group, or specific human name) that
  contributions to `releases` have been attributed to. Not necessarily
  one-to-one with a human person.
- `container` (aka "venue", "serial", "title"): a grouping of releases from a
  single publisher.

Note that, compared to many similar bibliographic ontologies, the current one
does not have entities to represent:

- funding sources
- publishing entities
- "events at a time and place"
- physical artifacts, either generically or specific copies
- sets of files (eg, a dataset or webpage with media)

Each entity type has it's own relations and fields (captured in a schema), but
there are are also generic operations and fields common across all entities.
The process of creating, updating, querying, and inspecting entities is roughly
the same regardless of type.

## Identifiers and Revisions

A specific version of any entity in the catalog is called a "revision".
Revisions are generally immutable (do not change and are not editable), and are
not usually referred to directly by users. Instead, persistent identifiers can
be created, which "point to" a specific revision at a time. This distinction
means that entities referred to by an identifier can change over time (as
metadata is corrected and expanded). Revision objects do not "point" back to
specific identifiers, so they are not the same as a simple "version number" for
an identifier.

Identifiers also have the ability to be merged (by redirecting one identifier
to another) and "deleted" (by pointing the identifier to no revision at all).
All changes to identifiers are captured as an "edit" object. Edit history can
be fetched and inspected on a per-identifier basis, and any changes can easily
be reverted (even merges/redirects and "deletion").

"Staged" or "proposed" changes are captured as edit objects without updating
the identifiers themselves.

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

A 64-bit namespace would probably be large enough, and would work with
database Integer columns:

    work_rzga5b9cd7efg
    https://fatcat.wiki/work/rzga5b9cd7efg

Fatcat identifiers can used to interlink between databases, but are explicitly
*not* intended to supplant DOIs, ISBNs, handle, ARKs, and other "registered"
persistent identifiers.

### Entity States

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

## Controlled Vocabularies 

Some individual fields have additional constraints, either in the form of
pattern validation ("values must be upper case, contain only certain
characters"), or membership in a fixed set of values. These may include:

- subject categorization
- license and open access status
- work "types" (article vs. book chapter vs. proceeding, etc)
- contributor types (author, translator, illustrator, etc)
- human languages
- identifier namespaces (DOI, ISBN, ISSN, ORCID, etc; but not the identifiers
  themselves)

Other fixed-set "vocabularies" become too large to easily maintain or express
in code. These could be added to the backend databases, or be enforced by bots
(instead of the core system itself). These mostly include externally-registered identifiers or types, such as:

- file mimetypes
- identifiers themselves (DOI, ORCID, etc), by checking for registration
  against canonical APIs and databases

## Global Edit Changelog

As part of the process of "accepting" an edit group, a row is written to an
immutable, append-only table (which internally is a SQL table) documenting each
identifier change. This changelog establishes a monotonically increasing
version number for the entire corpus, and should make interaction with other
systems easier (eg, search engines, replicated databases, alternative storage
backends, notification frameworks, etc.).

