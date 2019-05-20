
# Fileset Entity Reference

## Fields

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
