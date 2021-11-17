-- This file should undo anything in `up.sql`

ALTER TABLE file_rev
DROP COLUMN content_scope;

ALTER TABLE fileset_rev
DROP COLUMN content_scope;

ALTER TABLE webcapture_rev
DROP COLUMN content_scope;
