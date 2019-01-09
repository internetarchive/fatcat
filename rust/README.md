
Rust implementation of fatcat API server. Commands include:

- `fatcatd`: the API server itself
- `fatcat-auth`: privileged command to manage authentication keys, tokens, and
  accounts. Useful to generate admin accounts, new signing keys, etc.
- `fatcat-export`: high-speed JSON export tool, which talks directly to the
  database (instead of going through the API). See `README.export.md`.

The `fatcat-api-spec` crate is generated from the openapi/swagger spec and
contains Rust models, response types, and endpoint definitions (but not
implementations).

The SQL database schema (and migrations) are under `./migrations/`.

## Development

You need the following dependencies installed locally to build, run tests, and
do development work:

- rust stable, 1.29+ (eg, via "rustup", includes cargo tool)
- diesel (`cargo install diesel_cli`)
- postgres (9.6+; targetting 11.1 for production)
- postgres libs (debian: `sudo apt install libsqlite3-dev libpq-dev`)
- libsodium library and development headers (debian: `libsodium-dev`)

Copying commands out of `../.gitlab-ci.yml` file may be the fastest way to get
started.

Create a new postgres superuser. A regular postgres user and an existing
database should also work (with up/down migrations), but it's easier to just
blow the entire database away.

Copy `env.example` to `.env`, update if needed, then re-create database from
scratch:

    diesel database reset

Build and run:

    cargo run --bin fatcatd

Tests:

    cargo test

    # if you get weird postgres errors, try running only one test at a time
    # (slow and conservative)
    cargo test -- --test-threads 1

Note that most "integration" level tests are written in python and run by
`pytest`; see `../python/README.md`.

See `HACKING` for some more advanced tips and commands.

## Configuration

All configuration goes through environment variables, the notable ones being:

- `DATABASE_URL`: postgres connection details (username, password, host, and database)
- `TEST_DATABASE_URL`: used when running `cargo test`
- `AUTH_LOCATION`: the domain authentication tokens should be valid over
- `AUTH_KEY_IDENT`: a unique name for the primary auth signing key (used to
  find the correct key after key rotation has occured)
- `AUTH_SECRET_KEY`: base64-encoded secret key used to both sign and verify
  authentication tokens (symmetric encryption)
- `AUTH_ALT_KEYS`: additional ident/key pairs that can be used to verify tokens
  (to enable key rotation). Syntax is like `<ident1>:<key1>,<ident2>:key2,...`.

To setup authentication with a new secret authentication key, run:

    cargo run --bin fatcat-auth create-key

then copy the last line as `AUTH_SECRET_KEY` in `.env`, and update
`AUTH_KEY_IDENT` with a unique name for this new key (eg, including the date).
