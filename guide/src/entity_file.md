
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
    - `rel` (string, required): Eg: "webarchive".
- `mimetype` (string): Format of the file. If XML, specific schema can be
  included after a `+`. Example: "application/pdf"
- `release_ids` (array of string identifiers): references to `release` entities
  that this file represents a manifestation of. Note that a single file can
  contain multiple release references (eg, a PDF containing a full issue with
  many articles), and that a release will often have multiple files (differing
  only by watermarks, or different digitizations of the same printed work, or
  variant MIME/media types of the same published work).

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
