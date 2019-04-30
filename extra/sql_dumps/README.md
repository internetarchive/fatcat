
## HOWTO: Ident Table Snapshots

This will take somewhere around 15-25 GB of disk space on the database server
(under /tmp). It would probably be better to stream this transaction over a
network connection (saving database disk I/O), but I can't figure out how to do
that with plain SQL (multiple table dumps in a single session), so would need
to be a custom client.

    ./ident_table_snapshot.sh

Or, in production:

    sudo su postgres
    DATABASE_URL=fatcat_prod ./ident_table_snapshot.sh /tmp

## HOWTO: Dump abstracts, release identifiers, file hashes, etc

These are run as regular old commands, and can run across the network in a
couple different ways. We might not want database ports open to the network
(even cluster/VPN); on the other hand we could proabably do SSH port
forwarding anyways.

    # Locally, or client running on a remote machine
    psql fatcat < dump_abstracts.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | gzip > abstracts.json.gz

    # Run on database server, write to file on remote host
    psql fatcat < dump_abstracts.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | gzip | ssh user@host 'cat > abstracts.json.gz'

In production:

    sudo -u postgres psql fatcat_prod < dump_abstracts.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | pigz > /srv/fatcat/snapshots/abstracts.json.gz
    sudo -u postgres psql fatcat_prod < dump_file_hashes.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | pigz > /srv/fatcat/snapshots/file_hashes.tsv.gz
    sudo -u postgres psql fatcat_prod < dump_release_extid.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | pigz > /srv/fatcat/snapshots/release_extid.tsv.gz

## HOWTO: Full ("private") database backup and restore

    export DATESLUG="`date +%Y-%m-%d.%H%M%S`"
    time sudo -u postgres pg_dump --verbose --format=tar fatcat_prod | pigz > /srv/fatcat/snapshots/fatcat_full_dbdump_${DATESLUG}.tar.gz

In production (as of 2019-04-24, with a 283 GByte PostgreSQL database), this
takes 1h40m.

NOTE: by using the "directory" export (along with `--file`) instead of "tar"
export, it would be possible to use parallel dumping. However, this would put
additional load on both the database and underlying disk. Could also cause
issues with users/permissions.

To restore, CAREFULLY, run:

    sudo -u postgres pg_restore --clean --if-exists --create --exit-on-error --jobs=16 DUMP_FILE.tar

Or, in production:

    sudo su postgres
    time zcat fatcat_full_dbdump_2020-02-02.022209.tar.gz  | pg_restore --exit-on-error --clean --if-exists --dbname fatcat_prod

In QA (as of 2019-01-30 dump), this takes about 5h15m.

To just inspect a dump:

    pg_restore -l DUMP_FILE.tar.gz

## HOWTO: Public database dump

This dump will contain all tables in the backend schema, except for "private"
authentication tables. For local or non-production machines, might need to
replace the `fatcat_prod` database name.

    # TODO: for production, probably want consistent serialization mode
    export DATESLUG="`date +%Y-%m-%d.%H%M%S`"
    sudo -u postgres pg_dump --verbose --format=tar --exclude-table-data=auth_oidc fatcat_prod | pigz > /srv/fatcat/snapshots/fatcat_public_dbdump_${DATESLUG}.tar.gz

Can also run using the remote/SSH options above.

## Uploading to Internet Archive

The `./ia_item_exports_readme.md` and `sqldump` files should be included as a
`README.md` when appropriate:

    ia upload fatcat_bulk_exports_YYYY-MM-DD ia_exports_item_readme.md --remote-name=README.md
    ia upload fatcat_sqldump_full_YYYY-MM-DD ia_sqldump_item_readme.md --remote-name=README.md

Uploads should can be `--no-derive` to save cluster time.

Metadata should be set as:

- item name: `fatcat_bulk_exports_YYYY-MM-DD` or `fatcat_sqldump_public_YYYY-MM-DD` (or sometimes `fatcat_sqldump_full`)
- collection: `ia_biblio_metadata`
- creator: `Internet Archive Web Group`
- date: that the dump started (UTC)
- title: "Fatcat Bulk Metadata Exports (YYYY-MM-DD)" or "Fatcat Public Database Snapshot (YYYY-MM-DD)"

