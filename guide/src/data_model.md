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
- `release`: a specific "release" or "publicly published" version of a work.
  Contains traditional bibliographic metadata (title, date of publication,
  media type, language, etc). Has relationships to other entities:
    - child of a single `work` (required)
    - multiple `creator` entities as "contributors" (authors, editors)
    - outbound references to multiple other `release` entities
    - member of a single `container`, for example a journal or book series
- `file`: a single concrete, fixed digital artifact; a manifestation of one or
  more `releases`. Machine-verifiable metadata includes file hashes, size, and
  detected file format. Verified URLs link to locations on the open web where
  this file can be found or has been archived. Has relationships:
    - multiple `release` entities that this file is a complete manifestation of
      (almost always a single release)
- `fileset`: a list of muliple concrete files, together forming complete
  `release` manifestation. Primarily intended for datasets and supplementary
  materials; could also contain a paper "package" (source file and figures).
- `webcapture`: a single snapshot (point in time) of a webpage or small website
  (multiple pages) which are a complete manifestation of a `release`. Not a
  landing page or page referencing the release.
- `creator`: persona (pseudonym, group, or specific human name) that
  has contributed to one or more `release`. Not necessarily one-to-one with a
  human person.
- `container` (aka "venue", "serial", "title"): a grouping of releases from a
  single publisher.

Note that, compared to many similar bibliographic ontologies, the current one
does not have entities to represent:

- physical artifacts, either generically or specific copies
- funding sources
- publishing entities
- "events at a time and place"

Each entity type has it's own relations and fields (captured in a schema), but
there are are also generic operations and fields common across all entities.
The API for creating, updating, querying, and inspecting entities is roughly
the same regardless of type.

## Identifiers and Revisions

A specific version of any entity in the catalog is called a "revision".
Revisions are generally immutable (do not change and are not editable), and are
not normally referred to directly. Instead, persistent "fatcat identifiers"
(`ident`) can be created, which "point to" a single revision at a time. This
distinction means that entities referred to by an identifier can change over
time (as metadata is corrected and expanded). Revision objects do not "point"
back to specific identifiers, so they are not the same as a simple "version
number" for an identifier.

Identifiers also have the ability to be merged (by redirecting one identifier
to another) and "deleted" (by pointing the identifier to no revision at all).
All changes to identifiers are captured as an "edit" object. Edit history can
be fetched and inspected on a per-identifier basis, and any changes can easily
be reverted (even merges/redirects and "deletion").

"Work in progress" or "proposed" updates are staged as edit objects without
updating the identifiers themselves.

## Controlled Vocabularies 

Some individual fields have additional constraints, either in the form of
pattern validation ("values must be upper case, contain only certain
characters"), or membership in a fixed set of values. These may include:

- license and open access status
- work "types" (article vs. book chapter vs. proceeding, etc)
- contributor types (author, translator, illustrator, etc)
- human languages
- identifier namespaces (DOI, ISBN, ISSN, ORCID, etc; but not the identifiers
  themselves)

Other fixed-set "vocabularies" become too large to easily maintain or express
in code. These could be added to the backend databases, or be enforced by bots
(instead of the system itself). These mostly include externally-registered
identifiers or types, such as:

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

