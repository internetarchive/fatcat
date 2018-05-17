
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

Re-create database from scratch:

    diesel database reset

Build and run:

    cargo run

### Special Tricks

Regenerate API schemas:

    cargo install cargo-swagger  # uses docker
    cargo swagger fatcat-openapi2.yml fatcat-api --docker-tag=v2.3.1
    sudo chown `whoami`:`whoami` -R fatcat-api
    # edit fatcat-api/Cargo.toml, set name to "fatcat-api"
    cargo fmt
    # git commit the fatcat-api directory at this point

    diesel print-schema > src/database_schema.rs

Debugging SQL schema errors:

    psql fatcat_rs < migrations/2018-05-12-001226_init/up.sql

Creating entities via API:

    http --json post localhost:9411/v0/container name=asdf issn=1234-5678
