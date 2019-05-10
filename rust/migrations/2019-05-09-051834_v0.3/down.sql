-- This file should undo anything in `up.sql`

ALTER TABLE release_contrib 
DROP COLUMN given_name,
DROP COLUMN surname;

ALTER TABLE release_rev
RENAME COLUMN release_stage TO release_status;

ALTER TABLE release_rev
DROP COLUMN number,
DROP COLUMN version,
DROP COLUMN subtitle,
DROP COLUMN withdrawn_status,
DROP COLUMN withdrawn_date,
DROP COLUMN withdrawn_year;

DROP INDEX IF EXISTS release_rev_extid_type_value_idx;
DROP TABLE release_rev_extid;

ALTER TABLE webcapture_rev_cdx
DROP COLUMN size_bytes;
