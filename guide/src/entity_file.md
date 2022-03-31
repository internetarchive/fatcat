
# File Entity Reference

## Fields

- `size` (integer, positive, non-zero): Size of file in bytes. Eg: 1048576.
- `md5` (string): MD5 hash in lower-case hex. Eg: "d41efcc592d1e40ac13905377399eb9b".
- `sha1` (string): SHA-1 hash in lower-case hex. Not technically required, but
  the most-used of the hash fields and should always be included. Eg:
  "f013d66c7f6817d08b7eb2a93e6d0440c1f3e7f8".
- `sha256`: SHA-256 hash in lower-case hex. Eg:
  "a77e4c11a57f1d757fca5754a8f83b5d4ece49a2d28596889127c1a2f3f28832".
- `urls`: An array of "typed" URLs. Order is not meaningful, and may not be
  preserved.
    - `url` (string, required): Eg: "https://example.edu/~frau/prcding.pdf".
    - `rel` (string, required): Eg: "webarchive", see vocabulary below.
- `mimetype` (string): Format of the file. If XML, specific schema can be
  included after a `+`. Example: "application/pdf"
- `content_scope` (string): for situations where the file does not simply
  contain the full representation of a work (eg, fulltext of an article, for an
  `article-journal` release), describes what that scope of coverage is. Eg,
  entire `issue`, `corrupt` file. See vocabulary below.
- `release_ids` (array of string identifiers): references to `release` entities
  that this file represents a manifestation of. Note that a single file can
  contain multiple release references (eg, a PDF containing a full issue with
  many articles), and that a release will often have multiple files (differing
  only by watermarks, or different digitizations of the same printed work, or
  variant MIME/media types of the same published work).
- `extra` (object with string keys): additional metadata about this file
    - `path`: filename, with optional path prefix. path must be "relative", not
      "absolute", and should use UNIX-style forward slashes, not Windows-style
      backward slashes

#### URL `rel` Vocabulary

- `web`: generic public web sites; for `http/https` URLs, this should be the default
- `webarchive`: full URL to a resource in a long-term web archive
- `repository`: direct URL to a resource stored in a repository (eg, an
  institutional or field-specific research data repository)
- `academicsocial`: academic social networks (such as academia.edu or ResearchGate)
- `publisher`: resources hosted on publisher's website
- `aggregator`: fulltext aggregator or search engine, like CORE or Semantic
  Scholar
- `dweb`: content hosted on distributed/decentralized web protocols, such as
  `dat://` or `ipfs://` URLs

#### `content_scope` Vocabulary

This same vocabulary is shared between file, fileset, and webcapture entities;
not all the fields make sense for each entity type.

- if not set, assume that the artifact entity is valid and represents a
  complete copy of the release
- `issue`: artifact contains an entire issue of a serial publication (eg, issue
  of a journal), representing several releases in full
- `abstract`: contains only an abstract (short description) of the release, not
  the release itself (unless the `release_type` itself is `abstract`, in which
  case it is the entire release)
- `index`: index of a journal, or series of abstracts from a conference
- `slides`: slide deck (usually in "landscape" orientation)
- `front-matter`: non-article content from a journal, such as editorial policies
- `supplement`: usually a file entity which is a supplement or appendix, not
  the entire work
- `component`: a sub-component of a release, which may or may not be associated
  with a `component` release entity. For example, a single figure or table as
  part of an article
- `poster`: digital copy of a poster, eg as displayed at conference poster sessions
- `sample`: a partial sample of the entire work. eg, just the first page of an
  article. distinct from `truncated`
- `truncated`: the file has been truncated at a binary level, and may also be
  corrupt or invalid. distinct from `sample`
- `corrupt`: broken, mangled, or corrupt file (at the binary level)
- `stub`: any other out-of-scope artifact situations, where the artifact
  represents something which would not link to any possible in-scope release in
  the catalog (except a `stub` release)
- `landing-page`: for webcapture, the landing page of a work, as opposed to the
  work itself
- `spam`: content is spam. articles, webpages, or issues which include
  incidental advertisements within them are not counted as `spam`
