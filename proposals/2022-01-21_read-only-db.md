
status: in-progress

Database Read-Only Mode
=======================

Fatcat should have the ability to run in "read-only" mode, where all web and
API requests work as usual, except no modifications to the database are
possible.

There are at least two specific use cases for this mode:

- during certain maintenance operations, like migrating VMs or upgrading databases
- in the currently-planned high-availability mode, the "replica" VM (database,
  API, and web interface) is read-only

This proposal is to implement this at the SQL database level (using
`default_transaction_read_only` in PostgreSQL), have the API server detect this
error and report appropriately. Additionally, the web interface will have a
configuration variable that allows a site-wide alert message which will display
on every page, which will allow indicating things like upcoming downtime, read-only
mode, and read-replica responses.

TODO: have fatcat API server handle the specific error when a write is
attempted on a read replica database (distinct from
`default_transaction_read_only`).


## Configure PostgreSQL Read-Only Mode

Connect to database with `psql` (presumably as `postgres` superuser), and run:

    ALTER DATABASE fatcat_prod SET default_transaction_read_only = true;

Then restart the API server (to force reconnect).

Revert with the same process, but setting to `false`.


## Maintenance Process

When database maintenance is in process, of a type that requires freezing
writes to the database, first a banner should be set up via
`FATCAT_ALERT_MESSAGE` and restarting `fatcat-web` (see below).

Then, `default_transaction_read_only=true` can be set via `psql` (see
PostgreSQL section of this doc) and `fatcat-api` restarted.

When maintenance is done, reverse the process.


## Read-Only Backup Web Configuration

On replica (backup) servers, add a permanent `FATCAT_ALERT_MESSAGE` environment
variable for the `fatcat-web` service, which will impose a loud banner in the
web interface. HTML is allowed, so the message can link elsewhere for more
details.

If the replica is ever promoted to primary, this environment variable should be
removed and the `fatcat-web` service restarted.
