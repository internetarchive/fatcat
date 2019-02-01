
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

    sudo -u postgres psql fatcat_prod < dump_abstracts.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | gzip > /srv/fatcat/snapshots/abstracts.json.gz
    sudo -u postgres psql fatcat_prod < dump_file_hashes.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | gzip > /srv/fatcat/snapshots/file_hashes.tsv.gz
    sudo -u postgres psql fatcat_prod < dump_release_extid.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | gzip > /srv/fatcat/snapshots/release_extid.tsv.gz

## HOWTO: Full private database backup and restore

    export DATESLUG="`date +%Y-%m-%d.%H%M%S`"
    time sudo -u postgres pg_dump --verbose --format=tar fatcat_prod | gzip > /srv/fatcat/snapshots/fatcat_private_dbdump_${DATESLUG}.tar.gz

NOTE: by using the "directory" export (along with `--file`) instead of "tar"
export, it would be possible to use parallel dumping. However, this would put
additional load on both the database and underlying disk. Could also cause
issues with users/permissions.

To restore, CAREFULLY, run:

    sudo -u postgres pg_restore --clean --if-exists --create -exit-on-error --jobs=16 DUMP_FILE.tar.gz

To just inspect a dump:

    pg_restore -l DUMP_FILE.tar.gz

## HOWTO: Public database dump

This dump will contain all tables in the backend schema, except for "private"
authentication tables. For local or non-production machines, might need to
replace the `fatcat_prod` database name.

    # TODO: for production, probably want consistent serialization mode
    export DATESLUG="`date +%Y-%m-%d.%H%M%S`"
    sudo -u postgres pg_dump --verbose --format=tar --exclude-table-data=auth_oidc fatcat_prod | gzip > /srv/fatcat/snapshots/fatcat_public_dbdump_${DATESLUG}.tar.gz

Can also run using the remote/SSH options above.
