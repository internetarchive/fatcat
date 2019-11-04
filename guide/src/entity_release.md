
# Release Entity Reference

## Fields

- `title` (string, required): the display title of the release. May include subtitle.
- `subtitle` (string): intended only to be used primarily with books, not
  journal articles. Subtitle may also be appended to the `title` instead of
  populating this field.
- `original_title` (string): the full original language title, if `title` is translated
- `work_id` (fatcat identifier; required): the (single) work that this release
  is grouped under. If not specified in a creation (`POST`) action, the API
  will auto-generate a work.
- `container_id` (fatcat identifier): a (single) container that this release is
  part of. When expanded the `container` field contains the full `container`
  entity.
- `release_type` (string, controlled set): represents the medium or form-factor
  of this release; eg, "book" versus "journal article". Not necessarily
  the same across all releases of a work. See definitions below.
- `release_state` (string, controlled set): represents the publishing/review
  lifecycle status of this particular release of the work. See definitions
  below.
- `release_date` (string, ISO date format): when this release was first made
  publicly available. Blank if only year is known.
- `release_year` (integer): year when this release was first made
  publicly available; should match `release_date` if both are known.
- `withdrawn_status` (string, controlled set):
- `release_date` (string, ISO date format): when this release was first made
  publicly available. Blank if only year is known.
- `release_year` (integer): year when this release was first made
  publicly available; should match `release_date` if both are known.
- `ext_ids` (key/value object of string-to-string mappings): external
  identifiers. At least an empty `ext_ids` object is always required for
  release entities, so individual identifiers can be accessed directly.
- `volume` (string): optionally, stores the specific volume of a serial
  publication this release was published in.
        type: string
- `issue` (string): optionally, stores the specific issue of a serial
  publication this release was published in.
- `pages` (string): the pages (within a volume/issue of a publication) that
  this release can be looked up under. This is a free-form string, and could
  represent the first page, a range of pages, or even prefix pages (like
  "xii-xxx").
- `version` (string): optionally, describes distinguishes this release version
  from others. Generally a number, software-style version, or other short/slug
  string, not a freeform description. Book "edition" descriptions can also go
  in an `edition` extra field. Often used in conjunction with external
  identifiers. If you're not certain, don't use this field!
- `number` (string): an inherent identifier for this release (or work), often
  part of the title. For example, standards numbers, technical memo numbers,
  book series number, etc. Not a book `chapter` number however (which can be
  stored in `extra`). Depending on field or series-specific norms, the number
  may be stored here, in the title, or in both fields.
- `publisher` (string): name of the publishing entity. This does not need to be
  populated if the associated `container` entity has the publisher field set,
  though it is acceptable to duplicate, as the publishing entity of a container
  may differ over time. Should be set for singleton releases, like books.
- `language` (string, slug): the primary language used in this particular release of
  the work. Only a single language can be specified; additional languages can
  be stored in "extra" metadata (TODO: which field?). This field should be a
  valid RFC1766/ISO639 language code (two letters). AKA, a controlled
  vocabulary, not a free-form name of the language.
- `license_slug` (string, slug): the license of this release. Usually a
  creative commons short code (eg, `CC-BY`), though a small number of other
  short names for publisher-specific licenses are included (TODO: list these).
- `contribs` (array of objects): an array of authorship and other `creator` contributions to this
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
- `refs` (array of ident strings): references (aka, citations) to other releases. References
  can only be linked to a specific target release (not a work), though it may
  be ambiguous which release of a work is being referenced if the citation is
  not specific enough. Reference fields include:
    - `index` (integer, optional): reference lists and bibliographies almost
      always have an implicit order. Zero-indexed. Note that this is distinct
      from the `key` field.
    - `target_release_id` (fatcat identifier): if known, and the release
      exists, a cross-reference to the Fatcat entity
    - `extra` (JSON, optional): additional citation format metadata can be
      stored here, particularly if the citation schema does not align. Common
      fields might be "volume", "authors", "issue", "publisher", "url", and
      external identifiers ("doi", "isbn13").
    - `key` (string): works often reference works with a short slug or index
      number, which can be captured here. For example, "[BROWN2017]". Keys
      generally supersede the `index` field, though both can/should be
      supplied.
    - `year` (integer): year of publication of the cited release.
    - `container_title` (string): if applicable, the name of the container of
      the release being cited, as written in the citation (usually an
      abbreviation).
    - `title` (string): the title of the work/release being cited, as written.
    - `locator` (string): a more specific reference into the work/release being
      cited, for example the page number(s). For web reference, store the URL
      in "extra", not here.
- `abstracts` (array of objects): see below
  - `sha1` (string, hex, required): reference to the abstract content (string).
    Example: "3f242a192acc258bdfdb151943419437f440c313"
  - `content` (string): The abstract raw content itself. Example: `<jats:p>Some
    abstract thing goes here</jats:p>`
  - `mimetype` (string): not formally required, but should effectively always get
    set. `text/plain` if the abstract doesn't have a structured format
  - `lang` (string, controlled set): the human language this abstract is in. See
    the `lang` field of release for format and vocabulary.

#### External Identifiers (`ext_ids`)

The `ext_ids` object name-spaces external identifiers and makes it easier to
add new identifiers to the schema in the future.

Many identifier fields must match an internal regex (string syntax constraint)
to ensure they are properly formatted, though these checks aren't always
complete or correct in more obscure cases.

- `doi` (string): full DOI number, lower-case. Example: "10.1234/abcde.789".
  See the "External Identifiers" section of style guide for more notes
  about DOIs specifically.
- `wikidata_qid` (string): external identifier for Wikidata entities. These are
  integers prefixed with "Q", like "Q4321". Each `release` entity can be
  associated with at most one Wikidata entity (this field is not an array), and
  Wikidata entities should be associated with at most a single `release`. In
  the future it may be possible to associate Wikidata entities with `work`
  entities instead.
- `isbn13` (string): external identifier for books. ISBN-9 and other formats
  should be converted to canonical ISBN-13.
- `pmid` (string): external identifier for PubMed database. These are bare
  integers, but stored in a string format.
- `pmcid` (string): external identifier for PubMed Central database. These are
  integers prefixed with "PMC" (upper case), like "PMC4321". Versioned PMCIDs
  can also be stored (eg, "PMC4321.1"; future clarification of whether versions
  should *always* be stored will be needed.
- `core` (string): external identifier for the [CORE] open access
  aggregator. These identifiers are integers, but stored in string format.
- `arxiv` (string) external identifier to a (version-specific) [arxiv.org][]
  work. For releases, must always include the `vN` suffix (eg, `v3`).
- `jstor` (string) external identifier for works in JSTOR.
- `ark` (string) ARK identifer
- `mag` (string) Microsoft Academic Graph identifier

[arxiv.org]: https://arxiv.org

#### `extra` Fields

- `crossref` (object), for extra crossref-specific metadata
    - `subject` (array of strings) for subject/category of content
    - `type` (string) raw/original Crossref type
    - `alternative-id` (array of strings)
    - `archive` (array of strings), indicating preservation services deposited
    - `funder` (object/dictionary)
- `aliases` (array of strings) for additional titles this release might be
  known by
- `container_name` (string) if not matched to a container entity
- `group-title` (string) for releases within an collection/group
- `translation_of` (release identifier) if this release is a translation of
  another (usually under the same work)
- `superceded` (boolean) if there is another release under the same work that
  should be referenced/indicated instead. Intended as a temporary hint until
  proper work-based search is implemented. As an example use, all arxiv release
  versions except for the most recent get this set.

#### `release_type` Vocabulary

This vocabulary is based on the 
[CSL types](http://docs.citationstyles.org/en/stable/specification.html#appendix-iii-types),
with a small number of (proposed) extensions:

- `article-magazine`
- `article-journal`, including pre-prints and working papers
- `book`
- `chapter` is allowed as they are frequently referenced and read independent
  of the entire book. The data model does not currently support linking a
  subset of a release to an entity representing the entire release. The
  release/work/file distinctions should not be used to group multiple chapters under
  a single work; a book chapter can be it's own work. A paper which is
  republished as a chapter (eg, in a collection, or "edited" book) can have
  both releases under one work. The criteria of whether to "split" a book and
  have release entities for each chapter is whether the chapter has been
  cited/reference as such.
- `dataset`
- `entry`, which can be used for generic web resources like question/answer
  site entries.
- `entry-encyclopedia`
- `manuscript`
- `paper-conference`
- `patent`
- `post-weblog` for blog entries
- `report`
- `review`, for things like book reviews, not the "literature review" form of
  `article-journal`, nor peer reviews (see `peer_review`)
- `speech` can be used for eg, slides and recorded conference presentations
  themselves, as distinct from `paper-conference`
- `thesis`
- `webpage`
- `peer_review` (fatcat extension)
- `software` (fatcat extension)
- `standard` (fatcat extension), for technical standards like RFCs
- `abstract` (fatcat extension), for releases that are only an abstract of a
  larger work. In particular, translations. Many are granted DOIs.
- `editorial` (custom extension) for columns, "in this issue", and other
  content published along peer-reviewed content in journals. Many are granted DOIs.
- `letter` for "letters to the editor", "authors respond", and
  sub-article-length published content. Many are granted DOIs.
- `stub` (fatcat extension) for releases which have notable external
  identifiers, and thus are included "for completeness", but don't seem to
  represent a "full work".
- `component` (fatcat extension) for sub-components of a full paper (or other
  work). Eg, figures or tables.
  
An example of a `stub` might be a paper that gets an extra DOI by accident; the
primary DOI should be a full release, and the accidental DOI can be a `stub`
release under the same work. `stub` releases shouldn't be considered full
releases when counting or aggregating (though if technically difficult this may
not always be implemented). Other things that can be categorized as stubs
(which seem to often end up mis-categorized as full articles in bibliographic
databases):

- commercial advertisements
- "trap" or "honey pot" works, which are fakes included in databases to
  detect re-publishing without attribution
- "This page is intentionally blank"
- "About the author", "About the editors", "About the cover"
- "Acknowledgments"
- "Notices"

All other CSL types are also allowed, though they are mostly out of scope:

- `article` (generic; should usually be some other type)
- `article-newspaper`
- `bill`
- `broadcast`
- `entry-dictionary`
- `figure`
- `graphic`
- `interview`
- `legislation`
- `legal_case`
- `map`
- `motion_picture`
- `musical_score`
- `pamphlet`
- `personal_communication`
- `post`
- `review-book`
- `song`
- `treaty`

For the purpose of statistics, the following release types are considered
"papers":

- `article`
- `article-journal`
- `chapter`
- `paper-conference`
- `thesis`

#### `release_state` Vocabulary

These roughly follow the [DRIVER](http://web.archive.org/web/20091109125137/http://www2.lse.ac.uk/library/versions/VERSIONS_Toolkit_v1_final.pdf) publication version guidelines, with the addition of a `retracted` status.

- `draft` is an early version of a work which is not considered for peer
  review. Sometimes these are posted to websites or repositories for early
  comments and feedback.
- `submitted` is the version that was submitted for publication. Also known as
  "pre-print", "pre-review", "under review". Note that this doesn't imply that
  the work was every actually submitted, reviewed, or accepted for publication,
  just that this is the version that "would be". Most versions in pre-print
  repositories are likely to have this status.
- `accepted` is a version that has undergone peer review and accepted for
  published, but has not gone through any publisher copy editing or
  re-formatting. Also known as "post-print", "author's manuscript",
  "publisher's proof".
- `published` is the version that the publisher distributes. May include minor
  (gramatical, typographical, broken link, aesthetic) corrections. Also known
  as "version of record", "final publication version", "archival copy".
- `updated`: post-publication significant updates (considered a separate release
  in Fatcat). Also known as "correction" (in the context of either a published
  "correction notice", or the full new version)
- `retraction` for post-publication retraction notices (should be a release
  under the same work as the `published` release)

Note that in the case of a retraction, the original publication does not get
state `retracted`, only the retraction notice does. The original publication
does get a `withdrawn_status` metadata field set.

When blank, indicates status isn't known, and wasn't inferred at creation time.
Can often be interpreted as `published`, but be careful!

#### `withdrawn_status` Vocabulary

Don't know of an existing controlled vocabulary for things like retractions or
other reasons for marking papers as removed from publication, so invented my
own. These labels should be considered experimental and subject to change.

Note that some of these will apply more to pre-print servers or publishing
accidents, and don't necessarily make sense as a formal change of status for a
print journal publication.

Any value at all indicates that the release should be considered "no longer
published by the publisher or primary host", which could mean different things
in different contexts. As some concrete examples, works are often accidentally
generated a duplicate DOI; physics papers have been taken down in reponse to
government order under national security justifications; papers have been
withdrawn for public health reasons (above and beyond any academic-style
retraction); entire journals may be found to be predatory and pulled from
circulation; individual papers may be retracted by authors if a serious mistake
or error is found; an author's entire publication history may be retracted in
cases of serious academic misconduct or fraud.

- `withdrawn` is generic: the work is no longer available from the original
  publisher. There may be no reason, or the reason may not be known yet.
- `retracted` for when a work is formally retracted, usually accompanied by a
  retraction notice (a separate release under the same work). Note that the
  retraction itself should not have a `withdrawn_status`.
- `concern` for when publishers release an "expression of concern", often
  indicating that the work is not reliable in some way, but not yet formally
  retracted. In this case the original work is probably still available, but
  should be marked as suspect. This is not the same as presence of errata.
- `safety` for works pulled for public health or human safety concerns.
- `national-security` for works pulled over national security concerns.
- `spam` for content that is considered spam (eg, bogus pre-print or repository
  submissions). Not to be confused with advertisements or product reviews in
  journals.

#### `contribs.role` Vocabulary

- `author`
- `translator`
- `illustrator`
- `editor`

All other CSL role types are also allowed, though are mostly out of scope for
Fatcat:

- `collection-editor`
- `composer`
- `container-author`
- `director`
- `editorial-director`
- `editortranslator`
- `interviewer`
- `original-author`
- `recipient`
- `reviewed-author`

If blank, indicates that type of contribution is not known; this can often be
interpreted as authorship.
