
## Updating Schemas

Regenerate API schemas after editing the fatcat-openapi2 schema. This will, as
a side-effect, also run `cargo fmt` on the whole project, so don't run it with
your editor open!

    cargo install cargo-swagger  # uses docker
    ./codegen_openapi2.sh

Update Rust database schema (after changing raw SQL schema):

    diesel database reset
    diesel print-schema > src/database_schema.rs

Debug SQL schema errors (if diesel commands fail):

    psql fatcat_test < migrations/2018-05-12-001226_init/up.sql

## Direct API Interaction

Creating entities via API:

    http --json post localhost:9411/v0/container name=asdf issn=1234-5678
