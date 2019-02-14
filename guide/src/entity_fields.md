# Entity Field Reference

All entities have:

- `extra`: free-form JSON metadata

The "extra" field is an "escape hatch" to include extra fields not in the
regular schema. It is intended to enable gradual evolution of the schema, as
well as accommodating niche or field-specific content. Reasonable care should
be taken with this extra metadata: don't include large text or binary fields,
hundreds of fields, duplicate metadata, etc.

## Containers

- `name` (string, required): The title of the publication, as used in
  international indexing services. Eg, "Journal of Important Results". Not
  necessarily in the native language, but also not necessarily in English.
  Alternative titles (and translations) can be stored in "extra" metadata (see
  below)
- `container_type` (string): eg, journal vs. conference vs. book series.
  Controlled vocabulary is TODO.
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

#### `extra` Fields

- `abbrev` (string): a commonly used abbreviation for the publication, as used
  in citations, following the [ISO 4]() standard. Eg, "Journal of Polymer
  Science Part A" -> "J. Polym. Sci. A"
- `coden` (string): an external identifier, the [CODEN code](). 6 characters,
  all upper-case.
- `issnp` (string): Print ISSN
- `issne` (string): Electronic ISSN
- `default_license` (string, slug): short name (eg, "CC-BY-SA") for the
  default/recommended license for works published in this container
- `original_name` (string): native name (if `name` is translated)
- `platform` (string): hosting platform: OJS, wordpress, scielo, etc
- `mimetypes` (array of string): formats that this container publishes all works
  under (eg, 'application/pdf', 'text/html')
- `first_year` (integer): first year of publication
- `last_year` (integer): final year of publication (implies that container is no longer active)
- `languages` (array of strings): ISO codes; the first entry is considered the
  "primary" language (if that makes sense)
- `country` (string): ISO abbreviation (two characters) for the country this
  container is published in
- `aliases` (array of strings): significant alternative names or abbreviations
  for this container (not just capitalization/punctuation)
- `region` (string, slug): continent/world-region (vocabulary is TODO)
- `discipline` (string, slug): highest-level subject aread (vocabulary is TODO)
- `urls` (array of strings): known homepage URLs for this container (first in array is default)

Additional fields used in analytics and "curration" tracking:

- `doaj` (object)
  - `as_of` (string, ISO datetime): datetime of most recent check; if not set,
    not actually in DOAJ
  - `seal` (bool): has DOAJ seal
  - `work_level` (bool): whether work-level publications are registered with DOAJ
  - `archive` (array of strings): preservation archives
- `road` (object)
  - `as_of` (string, ISO datetime): datetime of most recent check; if not set,
    not actually in ROAD
- `kbart` (object)
  - `lockss`, `clockss`, `portico`, `jstor` etc (object)
    - `year_spans` (array of arrays of integers (pairs)): year spans (inclusive)
      for which the given archive has preserved this container
    - `volume_spans` (array of arrays of integers (pairs)): volume spans (inclusive)
      for which the given archive has preserved this container
- `sherpa_romeo` (object):
    - `color` (string): the SHERPA/RoMEO "color" of the publisher of this container
- `doi`: TODO: include list of prefixes and which (if any) DOI registrar is used
- `dblp` (object):
  - `id` (string)
- `ia` (object): Internet Archive specific fields
  - `sim` (object): same format as `kbart` preservation above; coverage in microfilm collection
  - `longtail` (bool): is this considered a "long-tail" open access venue


[CODEN]: https://en.wikipedia.org/wiki/CODEN

## Creators

- `display_name` (string, required): Full name, as will be displayed in user
  interfaces. Eg, "Grace Hopper"
- `given_name` (string): Also known as "first name". Eg, "Grace".
- `surname` (string): Also known as "last name". Eg, "Hooper".
- `orcid` (string): external identifier, as registered with ORCID.
- `wikidata_qid` (string): external linking identifier to a Wikidata entity.
  
See also ["Human Names"](./style_guide.md##human-names) sub-section of style guide.

## Files

- `size` (integer, positive, non-zero): Size of file in bytes. Eg: 1048576.
- `md5` (string): MD5 hash in lower-case hex. Eg: "d41efcc592d1e40ac13905377399eb9b".
- `sha1` (string): SHA-1 hash in lower-case hex. Not required, but the most-used of the hashes and should always be included. Eg: "f013d66c7f6817d08b7eb2a93e6d0440c1f3e7f8".
- `sha256`: SHA-256 hash in lower-case hex. Eg: "a77e4c11a57f1d757fca5754a8f83b5d4ece49a2d28596889127c1a2f3f28832".
- `urls`: An array of "typed" URLs. Order is not meaningful, and may not be
  preserved.
    - `url` (string, required):
            Eg: "https://example.edu/~frau/prcding.pdf".
    - `rel` (string, required):
            Eg: "webarchive".
- `mimetype` (string): Format of the file. If XML, specific schema can be
  included after a `+`. Example: "application/pdf"
- `release_ids` (array of string identifiers): references to `release` entities that this
  file represents a manifestation of. Note that a single file can contain
  multiple release references (eg, a PDF containing a full issue with many
  articles), and that a release will often have multiple files (differing only
  by watermarks, or different digitizations of the same printed work, or
  variant MIME/media types of the same published work).

## Filesets

Warning: This schema is not yet stable.

- `manifest` (array of objects): each entry represents a file
  - `path` (string, required): relative path to file (including filename)
  - `size` (integer, required): in bytes
  - `md5` (string): MD5 hash in lower-case hex
  - `sha1` (string): SHA-1 hash in lower-case hex
  - `sha256` (string): SHA-256 hash in lower-case hex
  - `extra` (object): any extra metadata about this specific file
- `urls`: An array of "typed" URLs. Order is not meaningful, and may not be
  preserved.
    - `url` (string, required):
            Eg: "https://example.edu/~frau/prcding.pdf".
    - `rel` (string, required):
            Eg: "webarchive".
- `release_ids` (array of string identifiers): references to `release` entities

## Webcaptures

Warning: This schema is not yet stable.

- `cdx` (array of objects): each entry represents a distinct web resource
  (URL). First is considered the primary/entry. Roughly aligns with CDXJ schema.
  - `surt` (string, required): sortable URL format
  - `timestamp` (string, datetime, required): ISO format, UTC timezone, with
    `Z` prefix required, with second (or finer) precision. Eg,
    "2016-09-19T17:20:24Z". Wayback timestamps (like "20160919172024") should
    be converted naively.
  - `url` (string, required): full URL
  - `mimetype` (string): content type of the resource
  - `status_code` (integer, signed): HTTP status code
  - `sha1` (string, required): SHA-1 hash in lower-case hex
  - `sha256` (string): SHA-256 hash in lower-case hex
- `archive_urls`: An array of "typed" URLs where this snapshot can be found.
  Can be wayback/memento instances, or direct links to a WARC file containing
  all the capture resources.  Often will only be a single archive. Order is not
  meaningful, and may not be preserved.
    - `url` (string, required):
            Eg: "https://example.edu/~frau/prcding.pdf".
    - `rel` (string, required): Eg: "wayback" or "warc"
- `original_url` (string): base URL of the resource. May reference a specific
  CDX entry, or maybe in normalized form.
- `timestamp` (string, datetime): same format as CDX line timestamp (UTC, etc).
  Corresponds to the overall capture timestamp. Can be the earliest of CDX
  timestamps if that makes sense
- `release_ids` (array of string identifiers): references to `release` entities

## Releases

- `title` (string, required): the display title of the release. May include subtitle.
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
- `release_status` (string, controlled set): represents the publishing/review
  lifecycle status of this particular release of the work. See definitions
  below.
- `release_date` (string, ISO date format): when this release was first made
  publicly available. Blank if only year is known.
- `release_year` (integer): year when this release was first made
  publicly available; should match `release_date` if both are known.
- `doi` (string): full DOI number, lower-case. Example: "10.1234/abcde.789".
  See the "External Identifiers" section of style guide.
- `wikidata_qid` (string): external identifier for Wikidata entities. These are
  integers prefixed with "Q", like "Q4321". Each `release` entity can be
  associated with at most one Wikidata entity (this field is not an array), and
  Wikidata entities should be associated with at most a single `release`. In
  the future it may be possible to associate Wikidata entities with `work`
  entities instead. See the "External Identifiers" section of style guide.
- `isbn13` (string): external identifier for books. ISBN-9 and other formats
  should be converted to canonical ISBN-13. See the "External Identifiers"
  section of style guide.
- `pmid` (string): external identifier for PubMed database. These are bare
  integers, but stored in a string format. See the "External Identifiers"
  section of style guide.
- `pmcid` (string): external identifier for PubMed Central database. These are
  integers prefixed with "PMC" (upper case), like "PMC4321". See the "External
  Identifiers" section of style guide.
- `core_id` (string): external identifier for the [CORE] open access
  aggregator. These identifiers are integers, but stored in string format. See
  the "External Identifiers" section of style guide.
- `arxiv_id` (string) external identifier to a (version-specific) [arxiv.org]()
  work
- `jstor_id` (string) external identifier for works in JSTOR
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
      exists, a cross-reference to the fatcat entity
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
- `subtitle` (string)
- `group-title` (string) for releases within an collection/group
  `release_status` getting updated)
- `translation_of` (release identifier) if this release is a translation of
  another (usually under the same work)
- `withdrawn_data` (string, ISO date format): if this release has been
  retracted (post-publication) or withdrawn (pre- or post-publication), this is
  the datetime of that event. Retractions also result in a `retraction` release
  under the same `work` entity. This is intended to migrate from "extra" to a
  full release entity field.

#### `release_type` Vocabulary

This vocabulary is based on the 
[CSL types](http://docs.citationstyles.org/en/stable/specification.html#appendix-iii-types),
with a small number of (proposed) extensions:

- `article-magazine`
- `article-newspaper`
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
- `review`, for things like book reviews, not the "literature review" form of `article-journal`
- `speech` can be used for eg, slides and recorded conference presentations
  themselves, as distinct from `paper-conference`
- `thesis`
- `webpage`
- `peer_review` (fatcat extension)
- `software` (fatcat extension)
- `standard` (fatcat extension)
- `abstract` (fatcat extension)
- `editorial` (custom extension) for columns, "in this issue", and other
  content published along peer-reviewed content in journals.
- `letter` for "letters to the editor", "authors respond", and
  sub-article-length published content
- `example` (custom extension) for dummy or example releases that have valid
  (registered) identifiers. Other metadata does not need to match "canonical"
  examples.
- `stub` (fatcat extension) for releases which have notable external
  identifiers, and thus are included "for completeness", but don't seem to
  represent a "full work". An example might be a paper that gets an extra DOI
  by accident; the primary DOI should be a full release, and the accidental DOI
  can be a `stub` release under the same work. `stub` releases shouldn't be
  considered full releases when counting or aggregating (though if technically
  difficult this may not always be implemented). Other things that can be
  categorized as stubs (which seem to often end up mis-categorized as full
  articles in bibliographic databases):
    - commercial advertisements
    - "trap" or "honey pot" works, which are fakes included in databases to
      detect re-publishing without attribution
    - "This page is intentionally blank"
    - "About the author", "About the editors", "About the cover"
    - "Acknowledgments"
    - "Notices"

All other CSL types are also allowed, though they are mostly out of scope:

- `article` (generic; should usually be some other type)
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

- `article-journal`
- `chapter`
- `paper-conference`
- `thesis`

#### `release_status` Vocabulary

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
status `retracted`, only the retraction notice does. The original publication
does get a `widthdrawn_date` metadata field set.

When blank, indicates status isn't known, and wasn't inferred at creation time.
Can often be interpreted as `published`, but be careful!

#### `contribs.role` Vocabulary

- `author`
- `translator`
- `illustrator`
- `editor`

If blank, indicates that type of contribution is not known; this can often be
interpreted as authorship.

## Works

Works have no fields! They just group releases.

