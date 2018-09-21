# Entity Field Reference

All entities have:

- `extra`: free-form JSON metadata

The "extra" field is an "escape hatch" to include extra fields not in the
regular schema. It is intented to enable gradual evolution of the schema, as
well as accomodating niche or field-specific content. That being said,
reasonable limits should be adhered to.

## Containers

- `name`: (string, required). The title of the publication, as used in
  international indexing services. Eg, "Journal of Important Results". Not
  necessarily in the native language, but also not necessarily in English.
  Alternative titles (and translations) can be stored in "extra" metadata
  (TODO: what field?).
- `publisher` (string): The name of the publishing organization. Eg, "Society
  of Curious Students".
- `issnl` (string): an external identifier, with registration controlled by the
  [ISSN organization](http://www.issn.org/). Registration is relatively
  inexpensive and easy to obtain (depending on world region), so almost all
  serial publications have one. The ISSN-L ("linking ISSN") is one of either
  the print ("ISSNp") or electronic ("ISSNe") identifiers for a serial
  publication; not all publications have both types of ISSN, but many do, which
  can cause confusion. The ISSN master list is not gratis/public, but the
  ISSN-L mapping is.
- `wikidata_qid` (string): external linking identifier to a Wikidata entity.
- `abbrev` (string): a commonly used abbreviation for the publication, as used
  in citations, following the [ISO 4]() standard. Eg, "Journal of Polymer
  Science Part A" -> "J. Polym. Sci. A". Alternative abbreviations can be
  stored in "extra" metadata. (TODO: what field?)
- `coden` (string): an external identifier, the [CODEN code](). 6 characters,
  all upper-case.

[CODEN]: https://en.wikipedia.org/wiki/CODEN

## Creators

See ["Human Names"](./style_guide.index##human-names) sub-section of style
guide.

- `display_name` (string, required): Eg, "Grace Hopper".
- `given_name` (string): Eg, "Grace".
- `surname` (string): Eg, "Hooper".
- `orcid` (string): external identifier, as registered with ORCID.
- `wikidata_qid` (string): external linking identifier to a Wikidata entity.

## Files

- `size` (positive, non-zero integer): Eg: 1048576.
- `sha1` (string): Eg: "f013d66c7f6817d08b7eb2a93e6d0440c1f3e7f8".
- `md5`: Eg: "d41efcc592d1e40ac13905377399eb9b".
- `sha256`: Eg: "a77e4c11a57f1d757fca5754a8f83b5d4ece49a2d28596889127c1a2f3f28832".
- `urls`: An array of "typed" URLs. Order is not meaningful, and may not be
  preserved.
    - `url` (string, required):
            Eg: "https://example.edu/~frau/prcding.pdf".
    - `rel` (string, required):
            Eg: "webarchive".
- `mimetype` (string):
    example: "application/pdf"
- `releases` (array of identifiers): references to `release` entities that this
  file represents a manifestation of. Note that a single file can contain
  multiple release references (eg, a PDF containing a full issue with many
  articles), and that a release will often have multiple files (differing only
  by watermarks, or different digitizations of the same printed work, or
  variant MIME/media types of the same published work). See also
  "Work/Release/File Distinctions".

## Releases

- `title` (required): the title of the release.
- `work_id` (fatcat identifier; required): the (single) work that this release
  is grouped under. If not specified in a creation (`POST`) action, the API
  will auto-generate a work.
- `container_id` (fatcat identifier): a (single) container that this release is
  part of. When expanded the `container` field contains the full `container`
  entity.
- `release_type` (string, controlled set): represents the medium or form-factor
  of this release; eg, "book" versus "journal article". Not necessarily
  consistent across all releases of a work. See definitions below.
- `release_status` (string, controlled set): represents the publishing/review
  lifecycle status of this particular release of the work. See definitions
  below.
- `release_date` (string, date format): when this release was first made
  publicly available
- `doi` (string): full DOI number, lower-case. Example: "10.1234/abcde.789".
  See the "External Identifiers" section of style guide.
- `isbn13` (string): external identifer for books. ISBN-9 and other formats
  should be converted to canonical ISBN-13. See the "External Identifiers"
  section of style guide.
- `core_id` (string): external identifier for the [CORE] open access
  aggregator. These identifiers are integers, but stored in string format. See
  the "External Identifiers" section of style guide.
- `pmid` (string): external identifier for PubMed database. These are bare
  integers, but stored in a string format. See the "External Identifiers"
  section of style guide.
- `pmcid` (string): external identifier for PubMed Central database. These are
  integers prefixed with "PMC" (upper case), like "PMC4321". See the "External
  Identifiers" section of style guide.
- `wikidata_qid` (string): external identifier for Wikidata entities. These are
  integers prefixed with "Q", like "Q4321". Each `release` entity can be
  associated with at most one Wikidata entity (this field is not an array), and
  Wikidata entities should be associated with at most a single `release`. In
  the future it may be possible to associate Wikidata entities with `work`
  entities instead. See the "External Identifiers" section of style guide.
- `volume` (string): optionally, stores the specific volume of a serial
  publication this release was published in.
        type: string
- `issue` (string): optionally, stores the specific issue of a serial
  publication this release was published in.
- `pages` (string): the pages (within a volume/issue of a publication) that
  this release can be looked up under. This is a free-form string, and could
  represent the first page, a range of pages, or even prefix pages (like
  "xii-xxx").
- `publisher` (string): name of the publishing entity. This does not need to be
  populated if the associated `container` entity has the publisher field set,
  though it is acceptable to duplicate, as the publishing entity of a container
  may differ over time. Should be set for singleton releases, like books.
- `language` (string): the primary language used in this particular release of
  the work. Only a single language can be specified; additional languages can
  be stored in "extra" metadata (TODO: which field?). This field should be a
  valid RFC1766/ISO639-1 language code ("with extensions"), aka a controlled
  vocabulary, not a free-form name of the language.
- `contribs`: an array of authorship and other `creator` contributions to this
  release. Contribution fields include:
    - `index` (integer, optional): the (zero-indexed) order of this
      author. Authorship order has significance in many fields. Non-author
      contributions (illustration, translation, editorship) may or may not be
      ordered, depending on context, but index numbers should be unique per
      release (aka, there should not be "first author" and "first translator")
    - `creator_id` (identifier): if known, a reference to a specific `creator`
    - `raw_name` (string): the name of the contributor, as attributed in the
      text of this work. If the `creator_id` is linked, this may be different
      from the `display_name`; if a creator is not linked, this field is
      particularly important. Syntax and name order is not specified, but most
      often will be "display order", not index/alphabetical (in Western
      tradition, surname followed by given name).
    - `role` (string, of a set): the type of contribution, from a controlled
      vocabulary. TODO: vocabulary needs review.
    - `extra` (string): additional context can go here. For example, author
      affiliation, "this is the corresponding author", etc.
- `refs`: an array of references (aka, citations) to other releases. References
  can only be linked to a specific target release (not a work), though it may
  be ambugious which release of a work is being referenced if the citation is
  not specific enough. Reference fields include:
    - `index` (integer, optional): reference lists and bibliographies almost
      always have an implicit order. Zero-indexed. Note that this is distinct
      from the `key` field.
    - `target_release_id` (fatcat identifier): if known, and the release
      exists, a cross-reference to the fatcat entity
    - `extra` (JSON, optional): additional citation format metadata can be
      stored here, particularly if the citation schema does not align. Common
      fields might be "volume", "authors", "issue", "publisher", "url", and
      external identifers ("doi", "isbn13").
    - `key` (string): works often reference works with a short slug or index
      number, which can be captured here. For example, "[BROWN2017]". Keys
      generally supercede the `index` field, though both can/should be
      supplied.
    - `year` (integer): year of publication of the cited release.
    - `container_title` (string): if applicable, the name of the container of
      the release being cited, as written in the citation (usually an
      abbreviation).
    - `title` (string): the title of the work/release being cited, as written.
    - `locator` (string): a more specific reference into the work/release being
      cited, for example the page number(s). For web reference, store the URL
      in "extra", not here.

Controlled vocabulary for `release_type` is derived from the Crossref `type`
vocabulary:

- `journal-article`
- `proceedings-article`
- `monograph`
- `dissertation`
- `book` (and `edited-book`, `reference-book`)
- `book-chapter` (and `book-part`, `book-section`, though much rarer) is
  allowed as these are frequently referenced and read independent of the entire
  book. The data model does not currently support linking a subset of a release
  to an entity representing the entire release. The release/work/file
  distinctions should not be used to group chapters into complete work; a book
  chapter can be it's own work. A paper which is republished as a chapter (eg,
  in a collection, or "edited" book) can have both releases under one work. The
  criteria of whether to "split" a book and have release entities for each
  chapter is whether the chapter has been cited/reference as such.
- `dissertation`
- `dataset` (though representation with `file` entities is TBD).
- `monograph`
- `report`
- `standard`
- `posted-content` is allowed, but may be re-categorized. For crossref, this
  seems to imply a journal article or report which is not published (pre-print)
- `other` matches Crossref `other` works, which may (and generally should) have
  a more specific type set.
- `web-post` (custom extension) for blog posts, essays, and other individual
  works on websites
- `website` (custom extension) for entire web sites and wikis.
- `presentation` (custom extension) for, eg, slides and recorded conference
  presentations themselves, as distinct from `proceedings-article`
- `editorial` (custom extension) for columns, "in this issue", and other
  content published along peer-reviewed content in journals. Can bleed in to
  "other" or "stub"
- `book-review` (custom extension)
- `letter` for "letters to the editor", "authors respond", and
  sub-article-length published content
- `example` (custom extension) for dummy or example releases that have valid
  (registered) identifiers. Other metadata does not need to match "canonical"
  examples.
- `stub` (custom extension) for releases which have notable external
  identifiers, and thus are included "for completeness", but don't seem to
  represent a "full work". An example might be a paper that gets an extra DOI
  by accident; the primary DOI should be a full release, and the accidental DOI
  can be a `stub` release under the same work. `stub` releases shouldn't be
  considered full releases when counting or aggregating (though if technically
  difficult this may not always be implemented). Other things that can be
  categorized as stubs (which seem to often end up miscategorized as full
  articles in bibliographic databases):
    - an abstract, which is only an abstract of a larger work
    - commercial advertisements
    - "trap" or "honey pot" works, which are fakes included in databases to
      detect re-publishing without attribution
    - "This page is intentionally blank"
    - "About the author", "About the editors", "About the cover"
    - "Acknowledgements"
    - "Notices"

Other types from Crossref (such as `component`, `reference-entry`) are valid,
but are not actively solicited for inclusion, as they are not the current focus
of the database.

In the future, some types (like `journal`, `proceedings`, and `book-series`)
will probably be represented as `container` entities. How to represent other
container-like types (like `report-series` or `book-series`) is TBD.

Controlled vocabulary for `release_status`:
- `published` for any version of the work that was "formally published", or any
  variant that can be considered a "proof", "camera ready", "archival",
  "version of record" or "definitive" that have no meaningful differences from
  the "published" version. Note that "meaningful" here will need to be
  explored.
- `corrected` for a version of a work that, after formal publication, has been
  revised and updated. Could be the "version of record".
- `pre-print`, for versions of a work which have not been submitted for peer
  review or formal publication
- `post-print`, often a post-peer-review version of a work that does not have
  publisher-supplied copy-editing, typesetting, etc.
- `draft` in the context of book publication or online content (shouldn't be
  applied to journal articles), is an unpublished, but somehow notable version
  of a work.
- If blank, indicates status isn't known, and wasn't inferred at creation time.
  Can often be interpreted as `published`.

Controlled vocabulary for `role` field on `contribs`:
- `author`
- `translator`
- `illustrator`
- `editor`
- If blank, indicates that type of contribution is not known; this can often be
  interpreted as authorship.

Current "extra" fields, flags, and content:
- `crossref` (object), for extra crossref-specific metadata
- `is_retracted` (boolean flag) if this work has been retracted
- `translation_of` (release identifier) if this release is a translation of
  another (usually under the same work)
- `arxiv_id` (string) external identifier to a (version-specific) [arxiv.org]()
  work

[arxiv.org]: https://arxiv.org

### Abstracts

Abstract *contents* (in raw string form) are stored in their own table, and are
immutable (not editable), but there is release-specific metadata as part of
`release` entities.

- `sha1` (string, hex, required): reference to the abstract content (string).
  Example: "3f242a192acc258bdfdb151943419437f440c313"
- `content` (string): The abstract raw content itself. Example: `<jats:p>Some
  abstract thing goes here</jats:p>`
- `mimetype` (string): not formally required, but should effectively always get
  set. `text/plain` if the abstract doesn't have a structured format
- `lang` (string, controlled set): the human language this abstract is in. See
  the `lang` field of release for format and vocabulary.

## Works

Works have no field! They just group releases.
