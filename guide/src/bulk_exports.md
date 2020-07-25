# Bulk Exports

There are several types of bulk exports and database dumps folks might be
interested in:

- complete database dumps
- changelog history with all entity revisions and edit metadata
- identifier snapshot tables
- entity exports

All exports and dumps get uploaded to the Internet Archive under the
["Fatcat Database Snapshots and Bulk Metadata Exports"](https://archive.org/details/fatcat_snapshots_and_exports?&sort=-publicdate)
collection.

## Complete Database Dumps

The most simple and complete bulk export. Useful for disaster recovery,
mirroring, or forking the entire service. The internal database schema is not
stable, so not as useful for longitudinal analysis. These dumps will include
edits-in-progress, deleted entities, old revisions, etc, which are potentially
difficult or impossible to fetch through the API.

Public copies may have some tables redacted (eg, API credentials).

Dumps are in PostgreSQL `pg_dump` "tar" binary format, and can be restored
locally with the `pg_restore` command. See `./extra/sql_dumps/` for commands
and details. Dumps are on the order of 100 GBytes (compressed) and will grow
over time.

## Changelog History

These are currently unimplemented; would involve "hydrating" sub-entities into
changelog exports. Useful for some mirrors, and analysis that needs to track
provenance information. Format would be the public API schema (JSON).

All information in these dumps should be possible to fetch via the public API,
including on a feed/streaming basis using the sequential changelog index. All
information is also contained in the database dumps.

## Identifier Snapshots

Many of the other dump formats are very large. To save time and bandwidth, a
few simple snapshot tables can be exported directly in TSV format. Because
these tables can be dumped in single SQL transactions, they are consistent
point-in-time snapshots.

One format is per-entity identifier/revision tables. These contain active,
deleted, and redirected identifiers, with revision and redirect references, and
are used to generate the entity dumps below.

Other tables contain external identifier mappings or file hashes.

Release abstracts can be dumped in their own table (JSON format), allowing them
to be included only by reference from other dumps.  The copyright status and
usage restrictions on abstracts are different from other catalog content; see
the [policy](./policy.md) page for more context.  Abstracts are immutable and
referenced by hash in the database, so the consistency of these dumps is not as
much of a concern as with other exports.

Unlike all other dumps and public formats, the Fatcat identifiers in these
dumps are in raw UUID format (not base32-encoded), though this may be fixed in
the future.

See `./extra/sql_dumps/` for scripts and details. Dumps are on the order of a
couple GBytes each (compressed).

## Entity Exports

Using the above identifier snapshots, the Rust `fatcat-export` program outputs
single-entity-per-line JSON files with the same schema as the HTTP API. These
might contain the default fields, or be in "expanded" format containing
sub-entities for each record.

Only "active" entities are included (not deleted, work-in-progress, or
redirected entities).

These dumps can be quite large when expanded (over 100 GBytes compressed), but
do not include history so will not grow as fast as other exports over time. Not
all entity types are dumped at the moment; if you would like specific dumps get
in touch!

