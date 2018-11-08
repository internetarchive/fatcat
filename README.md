
      __       _            _   
     / _| __ _| |_ ___ __ _| |_ 
    | |_ / _` | __/ __/ _` | __|
    |  _| (_| | || (_| (_| | |_ 
    |_|  \__,_|\__\___\__,_|\__|

                                        ... catalog all the things!


This repository contains source code for 'fatcat', an editable catalog of
published written works (mostly journal articles), with a focus on tracking
the location and status of full-text copies to ensure "perpetual access".

The [RFC](./fatcat-rfc.md) is the original design document, and the best place
to start for background. There is a work-in-progress "guide" at
<https://guide.fatcat.wiki>; the canonical public location of this repository
is <https://github.com/internetarchive/fatcat>.

There are three main components:

- backend API server and database (in Rust)
- API client libraries and bots (in Python)
- front-end web interface (in Python; built on API and library)

See the LICENSE file for details permissions and licensing of both python and
rust code. In short, the auto-generated client libraries are permissively
released, while the API server and web interface are strong copyleft (AGPLv3).

## Building and Tests

Automated integration tests run on Gitlab CI (see `.gitlab-ci.yml`) on the
Internet Archive's internal (not public) infrastructure.

## Status

- SQL and HTTP API schemas
    - [x] Basic entities
    - [x] one-to-many and many-to-many entities
    - [x] JSON(B) "extra" metadata fields
    - [x] full rev1 schema for all entities
    - [ ] editgroup review: comments? actions?
    - [ ] file sets and web captures
- HTTP API Server
    - [x] base32 encoding of UUID identifiers
    - [x] inverse many-to-many helpers (files-by-release, release-by-creator)
    - [ ] Authentication (eg, accounts, OAuth2, JWT)
    - [ ] Authorization (aka, roles)
- Web Interface
    - [x] Migrate Python codebase
    - [ ] Creation and editing of all entities
- Other
    - [x] Elasticsearch schema
    - [x] Basic logging
    - [x] Swagger-UI 
    - [x] Bulk metadata exports
    - [ ] Sentry (error reporting)
    - [ ] Metrics

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
