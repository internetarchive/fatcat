
Rust implementation of fatcat API server. Commands include:

- `fatcatd`: the API server itself
- `fatcat-auth`: privileged command to manage authentication keys, tokens, and
  accounts. Useful to generate admin accounts, new signing keys, etc.
- `fatcat-export`: high-speed JSON export tool, which talks directly to the
  database (instead of going through the API).

The `fatcat-openapi` crate is generated from the openapi/swagger spec and
contains Rust models, response types, and endpoint definitions (but not
implementations).

The SQL database schema (and migrations) are under `./migrations/`.

## Development

You need the following dependencies installed locally to build, run tests, and
do development work:

- rust stable, 2018 edition, 1.34+ (eg, via "rustup", includes cargo tool)
- diesel (`cargo install diesel_cli`)
- postgres (compatible with 9.6+; we run 11.x in production)
- postgres libs (debian/ubuntu: `libsqlite3-dev libpq-dev`)
- libsodium library and development headers (debian/ubuntu: `libsodium-dev`)
- OpenSSL

We need to create a new `fatcat` postgres user and database to run tests and
develop with. On debian/ubuntu, a UNIX account named `postgres` is
automatically created with control over the database, so we'll run setup
commands from that user. To create the database account:

    sudo su - postgres

    # this command creates a PostgreSQL user, not a UNIX/system user
    createuser -s fatcat -P

    # switch back to your regular system user
    exit

Copy `./example.env` to `./.env` and update the `DATABASE_URL` and
`TEST_DATABASE_URL` lines with the password you set above.

As your regular user, use `diesel` to create and initialize the `fatcat`
database (`diesel` and the fatcat tools will automatically use postgresql
credentials from the `.env` file):

    diesel database reset

Build and run the API server:

    cargo run --bin fatcatd

Run tests:

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
