
Things!

    sudo apt install libsqlite3-dev libpq-dev

    wget https://oss.sonatype.org/content/repositories/snapshots/io/swagger/swagger-codegen-cli/3.0.0-SNAPSHOT/swagger-codegen-cli-3.0.0-20180411.134218-60.jar

    cargo swagger ../golang/fatcat-openapi2.yml gen-out

    diesel print-schema > src/database_schema.rs

Debugging SQL errors:

    psql fatcat_rs < migrations/2018-05-12-001226_init/up.sql

Creating entities:

    http --json post localhost:9411/v0/container name=asdf issn=1234-5678
