
# Web Capture Entity Reference

## Fields

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
- `content_scope` (string): for situations where the webcapture does not simply
  contain the full representation of a work (eg, HTML fulltext, for an
  `article-journal` release), describes what that scope of coverage is. Eg,
  `landing-page` it doesn't contain the full content. Landing pages are
  out-of-scope for fatcat, but if they were accidentally imported, should mark
  them as such so they aren't re-imported. Uses same vocabulary as File entity.
- `release_ids` (array of string identifiers): references to `release` entities
