
              __       _            _   
             / _| __ _| |_ ___ __ _| |_ 
            | |_ / _` | __/ __/ _` | __|
            |  _| (_| | || (_| (_| | |_ 
            |_|  \__,_|\__\___\__,_|\__|

       perpetual access to the scholarly record


[![pipeline status](https://gitlab.com/bnewbold/fatcat/badges/master/pipeline.svg)](https://gitlab.com/bnewbold/fatcat/commits/master)
[![coverage report](https://gitlab.com/bnewbold/fatcat/badges/master/coverage.svg)](https://gitlab.com/bnewbold/fatcat/commits/master)

This repository contains source code for 'fatcat', an editable catalog of
published written works (mostly journal articles), with a focus on tracking
the location and status of full-text copies to ensure "perpetual access".

The [RFC](./fatcat-rfc.md) is the original design document, and the best place
to start for technical background. There is a work-in-progress "guide" at
<https://guide.fatcat.wiki>; the canonical public location of this repository
is <https://github.com/internetarchive/fatcat>.

The public production web interface is <https://fatcat.wiki>.

See the `LICENSE` file for detailed permissions and licensing of both python
and rust code. In short, the auto-generated client libraries are permissively
released, while the API server and web interface are strong copyleft (AGPLv3).

## Building and Tests

There are three main components:

- backend API server and database (in Rust)
- API client libraries and bots (in Python)
- front-end web interface (in Python; built on API and library)

Automated integration tests run on Gitlab CI (see `.gitlab-ci.yml`) on the
Internet Archive's internal (not public) infrastructure.

See `./python/README.md` and `./rust/README.md` for details on building,
running, and testing these components.

The python client library, which is automatically generated from the API
schema, lives under `./python_openapi_client/`.

## Status

- SQL and HTTP API schemas
    - [x] Basic entities
    - [x] one-to-many and many-to-many entities
    - [x] JSON(B) "extra" metadata fields
    - [x] full rev1 schema for all entities
    - [x] file sets and web captures
    - [x] editgroup review: annotations
- HTTP API Server
    - [x] base32 encoding of UUID identifiers
    - [x] inverse many-to-many helpers (files-by-release, release-by-creator)
    - [x] Authentication (eg, accounts, OAuth2, JWT)
    - [x] Authorization (aka, roles)
- Web Interface
    - [x] Migrate Python codebase
    - [x] Creation and editing of all entities
- Other
    - [x] Elasticsearch schema
    - [x] Basic logging
    - [x] Swagger-UI 
    - [x] Bulk metadata exports
    - [x] Sentry (error reporting)
    - [x] Metrics

