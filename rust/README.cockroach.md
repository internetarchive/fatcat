
Setup a user and database on a local, single-host node:

    # using v2.0.5 (2018/08/13)
    cockroach start --insecure --store=fatcat-dev --host=localhost
    cockroach user set maxroach --insecure
    cockroach sql --insecure -e 'CREATE DATABASE fatcat'
    cockroach sql --insecure -e 'GRANT ALL ON DATABASE fatcat TO maxroach'
    cockroach sql --insecure -e 'CREATE DATABASE fatcat_test'
    cockroach sql --insecure -e 'GRANT ALL ON DATABASE fatcat_test TO maxroach'

Create a .env file:

    DATABASE_URL=postgres://maxroach@localhost:26257/fatcat
    TEST_DATABASE_URL=postgres://maxroach@localhost:26257/fatcat_test
    RUST_TEST_THREADS=1

Create:

    cockroach sql --insecure -d fatcat < migrations/2018-05-12-001226_init/down.sql
    cockroach sql --insecure -d fatcat < migrations/2018-05-12-001226_init/up.sql

    cockroach sql --insecure -d fatcat_test < migrations/2018-05-12-001226_init/down.sql
    cockroach sql --insecure -d fatcat_test < migrations/2018-05-12-001226_init/up.sql
