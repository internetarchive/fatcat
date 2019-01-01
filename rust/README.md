
Rust implementation of fatcat API server (`fatcatd`).

## Development

- rust stable, 1.29+ (eg, via "rustup", includes cargo tool)
- diesel (`cargo install diesel_cli`)
- postgres (9.6+; targetting 11.1 for production)
- postgres libs (debian: `sudo apt install libsqlite3-dev libpq-dev`)
- libsodium library and development headers (debian: `libsodium-dev`)

Create a new postgres superuser. A regular postgres user and an existing
database should also work (with up/down migrations), but it's easier to just
blow the entire database away.

Create a `.env` file with configuration:

    DATABASE_URL=postgres://fatcat:tactaf@localhost/fatcat_rs
    TEST_DATABASE_URL=postgres://fatcat:tactaf@localhost/fatcat_rs_test
    AUTH_LOCATION=dev.fatcat.wiki
    AUTH_KEY_IDENT=2018-12-31-dev
    AUTH_SECRET_KEY=VQe8kdn8laZ3MArKAzOeWWNUQgM6IjduG2jwKnSWehQ=

Re-create database from scratch:

    diesel database reset

Build and run:

    cargo run --bin fatcatd

Tests:

    cargo test -- --test-threads 1

See `HACKING` for some more advanced tips and commands.
