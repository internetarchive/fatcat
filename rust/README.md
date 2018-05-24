
Rust implementation of fatcat API server (`fatcatd`).

## Status

- HTTP API
    - [ ] base32 encoding of UUID identifiers
- SQL Schema
    - [x] Basic entities
    - [ ] one-to-many and many-to-many entities
    - [ ] JSON(B) "extra" metadata fields
- Web Interface
    - [ ] Migrate Python codebase
- Other
    - [x] Basic logging
    - [x] Swagger-UI 
    - [ ] Sentry (error reporting)
    - [ ] Metrics
    - [ ] Authentication (eg, accounts, OAuth2, JWT)
    - [ ] Authorization (aka, roles)

## Development

- rust stable, 1.26+ (eg, via "rustup", includes cargo tool)
- diesel (`cargo install diesel_cli`)
- postgres (9.6+; targetting 10.3 for production)
- postgres libs (debian: `sudo apt install libsqlite3-dev libpq-dev`)

Create a new postgres superuser. A regular postgres user and an existing
database should also work (with up/down migrations), but it's easier to just
blow the entire database away.

Create a `.env` file with configuration:

    DATABASE_URL=postgres://fatcat:tactaf@localhost/fatcat_rs
    TEST_DATABASE_URL=postgres://fatcat:tactaf@localhost/fatcat_rs_test

Re-create database from scratch:

    diesel database reset

Build and run:

    cargo run

Tests:

    cargo test -- --test-threads 1

### Special Tricks

Regenerate API schemas:

    cargo install cargo-swagger  # uses docker
    cargo swagger fatcat-openapi2.yml fatcat-api --docker-tag=v2.3.1
    sudo chown `whoami`:`whoami` -R fatcat-api

    # usually want to keep our changes to sub-module toml
    git checkout fatcat-api/Cargo.toml

    cargo fmt
    # git commit the fatcat-api directory at this point

Regenerate SQL schema:

    diesel database reset
    diesel print-schema > src/database_schema.rs

Debugging SQL schema errors:

    psql fatcat_rs < migrations/2018-05-12-001226_init/up.sql

Creating entities via API:

    http --json post localhost:9411/v0/container name=asdf issn=1234-5678
