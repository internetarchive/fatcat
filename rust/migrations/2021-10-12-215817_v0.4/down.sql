-- This file should undo anything in `up.sql`

DROP INDEX IF EXISTS container_rev_issne_idx;
DROP INDEX IF EXISTS container_rev_issnp_idx;

ALTER TABLE container_rev
DROP COLUMN issne,
DROP COLUMN issnp,
DROP COLUMN publication_status;

ALTER TABLE fileset_rev_file 
DROP COLUMN mimetype;

DELETE FROM release_rev_extid WHERE extid_type = 'hdl';
