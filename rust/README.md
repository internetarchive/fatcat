
Rust implementation of fatcat API server (`fatcatd`).

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

## Simple Deployment

Canonical ansible scripts are in the journal-infra repo. To install manually,
on a bare server, as root:

    adduser fatcat
    apt install postgresql-9.6 postgresql-contrib postgresql-client-9.6 \
        nginx build-essential git pkg-config libssl-dev libpq-dev \
        htop screen
    mkdir -p /srv/fatcat
    chown fatcat:fatcat /srv/fatcat

    # setup new postgres user
    su - postgres
    createuser -P -s fatcat     # strong random password
    # DELETE: createdb fatcat

    # as fatcat user
    su - fatcat
    ssh-keygen
    curl https://sh.rustup.rs -sSf | sh
    source $HOME/.cargo/env
    cargo install diesel_cli --no-default-features --features "postgres"
    cd /srv/fatcat
    git clone git@git.archive.org:webgroup/fatcat
    cd rust
    cargo build
    echo "DATABASE_URL=postgres://fatcat@localhost/fatcat" > .env
    diesel database reset

    # as fatcat, in a screen or something
    cd /srv/fatcat/fatcat/rust
    cargo run

### Special Tricks

Regenerate API schemas (this will, as a side-effect, also run `cargo fmt` on
the whole project, so don't run it with your editor open):

    cargo install cargo-swagger  # uses docker
    ./codegen_openapi2.sh

Regenerate SQL schema:

    diesel database reset
    diesel print-schema > src/database_schema.rs

Debugging SQL schema errors:

    psql fatcat_rs < migrations/2018-05-12-001226_init/up.sql

Creating entities via API:

    http --json post localhost:9411/v0/container name=asdf issn=1234-5678
