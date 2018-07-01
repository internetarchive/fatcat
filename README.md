
      __       _            _   
     / _| __ _| |_ ___ __ _| |_ 
    | |_ / _` | __/ __/ _` | __|
    |  _| (_| | || (_| (_| | |_ 
    |_|  \__,_|\__\___\__,_|\__|

                                        ... catalog all the things!


The [RFC](./farcat-rfc.md) is the original design document, and the best place
to start for background.

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

## Status

- HTTP API
    - [ ] base32 encoding of UUID identifiers
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
