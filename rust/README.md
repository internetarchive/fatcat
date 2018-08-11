
Rust implementation of fatcat API server (`fatcatd`).

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

### Dumps and Backups

There are a few different databaase dump formats folks might want:

- raw native database backups, for disaster recovery (would include
  volatile/unsupported schema details, user API credentials, full history,
  in-process edits, comments, etc)
- a sanitized version of the above: roughly per-table dumps of the full state
  of the database. Could use per-table SQL expressions with sub-queries to pull
  in small tables ("partial transform") and export JSON for each table; would
  be extra work to maintain, so not pursuing for now.
- full history, full public schema exports, in a form that might be used to
  mirror or enitrely fork the project. Propose supplying the full "changelog"
  in API schema format, in a single file to capture all entity history, without
  "hydrating" any inter-entity references. Rely on separate dumps of
  non-entity, non-versioned tables (editors, abstracts, etc). Note that a
  variant of this could use the public interface, in particular to do
  incremental updates (though that wouldn't capture schema changes).
- transformed exports of the current state of the database (aka, without
  history). Useful for data analysis, search engines, etc. Propose supplying
  just the Release table in a fully "hydrated" state to start. Unclear if
  should be on a work or release basis; will go with release for now. Harder to
  do using public interface because of the need for transaction locking.

Backing up the entire database using `pg_dump`, with parallelism 1 (use more on
larger machine with fast disks; try 4 or 8?), assuming the database name is
'fatcat', and the current user has access:

    pg_dump -j1 -Fd -f test-dump fatcat

### Special Tricks

Regenerate API schemas (this will, as a side-effect, also run `cargo fmt` on
the whole project, so don't run it with your editor open):

    cargo install cargo-swagger  # uses docker
    ./codegen_openapi2.sh

Regenerate SQL schema:

    diesel database reset
    diesel print-schema > src/database_schema.rs

Debugging SQL schema errors:

    psql fatcat_test < migrations/2018-05-12-001226_init/up.sql

Creating entities via API:

    http --json post localhost:9411/v0/container name=asdf issn=1234-5678
