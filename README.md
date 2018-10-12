
      __       _            _   
     / _| __ _| |_ ___ __ _| |_ 
    | |_ / _` | __/ __/ _` | __|
    |  _| (_| | || (_| (_| | |_ 
    |_|  \__,_|\__\___\__,_|\__|

                                        ... catalog all the things!


The [RFC](./fatcat-rfc.md) is the original design document, and the best place
to start for background. There is a work-in-progress "guide" at
<https://guide.fatcat.wiki>; the canonical public location of this repository
is <https://github.com/internetarchive/fatcat>.

There are four main components:

- backend API server and database
- elasticsearch index
- API client libraries and bots (eg, ingesters)
- front-end web interface (built on API and library)

The API server was prototyped in python. "Real" implementation started in
golang, but shifted to Rust, and is work-in-progress. The beginings of a client
library, web interface, and data ingesters exist in python. Elasticsearch index
is currently just a Crossref metadata dump and doesn't match entities in the
database/API (but is useful for paper lookups).

See the LICENSE file for details permissions and licensing of both python and
rust code. In short, the auto-generated client libraries are permissively
released, while the API server and web interface are strong copyleft (AGPLv3).

## Status

- HTTP API
    - [x] base32 encoding of UUID identifiers
    - [x] inverse many-to-many helpers (files-by-release, release-by-creator)
- SQL Schema
    - [x] Basic entities
    - [x] one-to-many and many-to-many entities
    - [x] JSON(B) "extra" metadata fields
    - [x] full rev1 schema for all entities
    - [ ] editgroup review: comments? actions?
- Web Interface
    - [x] Migrate Python codebase
    - [ ] Creation and editing of all entities
- Other
    - [x] Basic logging
    - [x] Swagger-UI 
    - [ ] Sentry (error reporting)
    - [ ] Metrics
    - [ ] Authentication (eg, accounts, OAuth2, JWT)
    - [ ] Authorization (aka, roles)
    - [ ] bot vs. editor

## Identifiers

Fatcat entity identifiers are 128-bit UUIDs encoded in base32 format. Revision
ids are also UUIDs, and encoded in normal UUID fashion, to disambiguate from
edity identifiers.

Python helpers for conversion:

    import base64
    import uuid

    def fcid2uuid(s):
        s = s.split('_')[-1].upper().encode('utf-8')
        assert len(s) == 26
        raw = base64.b32decode(s + b"======")
        return str(uuid.UUID(bytes=raw)).lower()

    def uuid2fcid(s):
        raw = uuid.UUID(s).bytes
        return base64.b32encode(raw)[:26].lower().decode('utf-8')

    test_uuid = '00000000-0000-0000-3333-000000000001'
    assert test_uuid == fcid2uuid(uuid2fcid(test_uuid))
