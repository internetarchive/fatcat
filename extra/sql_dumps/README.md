
## HOWTO: Ident Table Snapshots

How to take a consistent (single transaction) snapshot of 

This will take somewhere around 15-25 GB of disk space on the database server
(under /tmp). It would probably be better to stream this transaction over a
network connection (saving database disk I/O), but I can't figure out how to do
that with plain SQL (multiple table dumps in a single session), so would need
to be a custom client.

    ./ident_table_snapshot.sh

## HOWTO: Dump abstracts, release identifiers, file hashes, etc

These are run as regular old commands, and can run across the network in a
couple different ways. We might not want database ports open to the network
(even cluster/VPN); on the other hand we could proabably do SSH port
forwarding anyways.

    # Locally, or client running on a remote machine
    psql fatcat < dump_abstracts.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | gzip > abstracts.json.gz

    # Run on database server, write to file on remote host
    psql fatcat < dump_abstracts.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | gzip | ssh user@host 'cat > abstracts.json.gz'

