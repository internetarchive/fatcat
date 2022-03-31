
# Fileset Entity Reference

## Fields

- `manifest` (array of objects): each entry represents a file
  - `path` (string, required): relative path to file (including filename)
  - `size` (integer, required): in bytes
  - `md5` (string): MD5 hash in lower-case hex
  - `sha1` (string): SHA-1 hash in lower-case hex
  - `sha256` (string): SHA-256 hash in lower-case hex
  - `mimetype` (string): Content type in MIME type schema
  - `extra` (object): any extra metadata about this specific file. all are
    optional
    - `original_url`: live web canonical URL to download this file
    - `webarchive_url`: web archive capture of this file
- `urls`: An array of "typed" URLs. Order is not meaningful, and may not be
  preserved. These are URLs for the entire fileset, not individual files.
    - `url` (string, required):
            Eg: "https://example.edu/~frau/prcding.pdf".
    - `rel` (string, required):
            Eg: "archive-base", "webarchive".

- `release_ids` (array of string identifiers): references to `release` entities
- `content_scope` (string): for situations where the fileset does not simply
  contain the full representation of a work (eg, all files in dataset, for a
  `dataset` release), describes what that scope of coverage is. Uses same
  vocabulary as File entity.
- `extra` (object with string keys): additional metadata about this group of
  files, including upstream platform-specific metadata and identifiers
  - `platform_id`: platform-specific identifier for this fileset

#### URL `rel` types

Any ending in "-base" implies that a file path (from the manifest) can be
appended to the "base" URL to get a file download URL. Any "bundle" implies a
direct link to an archive or "bundle" (like `.zip` or `.tar`) which contains
all the files in this fileset

- `repository` or `platform`: URL of a live-web landing page or other location
  where content can be found. May or may not be machine-reachable.
- `webarchive`: web archive version of `repository`
- `repository-bundle`: direct URL to a live-web "archive" file, such as `.zip`,
  which contains all of the individual files in this fileset
- `webarchive-bundle`: web archive version of `repository-bundle`
- `archive-bundle`: file archive version of `repository-bundle`
- `repository-base`: live-web base URL/directory from which file `path` can be
  appended to fetch individual files
- `archive-base`: base URL/directory from which file `path` can be appended to fetch
  individual files

