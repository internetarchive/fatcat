
-- Set of schema additions, tweaks, and missing indices
-- Part of v0.3.0 (May 2019) backwards-incompatible changes to the Fatcat API

-------------------- Release -----------------------------------------------

-- structured contrib author names (in addition to 'raw_name')
ALTER TABLE release_contrib 
ADD COLUMN given_name          TEXT CHECK (octet_length(raw_name) >= 1),
ADD COLUMN surname             TEXT CHECK (octet_length(surname) >= 1);

-- release_status => release_stage (clarifies meaning of field)
ALTER TABLE release_rev
RENAME COLUMN release_status TO release_stage;

-- additional fields; withdrawl tracking; additional external identifiers
ALTER TABLE release_rev
ADD COLUMN number                TEXT CHECK (octet_length(number) >= 1),
ADD COLUMN version               TEXT CHECK (octet_length(version) >= 1),
ADD COLUMN subtitle              TEXT CHECK (octet_length(subtitle) >= 1),
ADD COLUMN withdrawn_state       TEXT,  -- TODO: enum?
ADD COLUMN withdrawn_date        DATE,
ADD COLUMN withdrawn_year        BIGINT,
ADD COLUMN mag_id                TEXT CHECK (octet_length(mag_id) >= 1),
ADD COLUMN ark_id                TEXT CHECK (octet_length(ark_id) >= 5);

--CREATE INDEX IF NOT EXISTS release_rev_mag_idx ON release_rev(mag_id);
--CREATE INDEX IF NOT EXISTS release_rev_ark_idx ON release_rev(mag_id);

-------------------- Web Capture -------------------------------------------

ALTER TABLE webcapture_rev_cdx
ADD COLUMN size_bytes          BIGINT;

-------------------- Edit Indices ------------------------------------------

-- there is already a UNIQ index on (editgroup_id, ident_id)
CREATE INDEX IF NOT EXISTS creator_edit_ident_idx ON creator_edit(ident_id);
CREATE INDEX IF NOT EXISTS container_edit_ident_idx ON container_edit(ident_id);
CREATE INDEX IF NOT EXISTS file_edit_ident_idx ON file_edit(ident_id);
CREATE INDEX IF NOT EXISTS fileset_edit_ident_idx ON fileset_edit(ident_id);
CREATE INDEX IF NOT EXISTS webcapture_edit_ident_idx ON webcapture_edit(ident_id);
CREATE INDEX IF NOT EXISTS release_edit_ident_idx ON release_edit(ident_id);
CREATE INDEX IF NOT EXISTS work_edit_ident_idx ON work_edit(ident_id);

-- TODO: not needed?
-- CREATE INDEX IF NOT EXISTS creator_edit_editgroup_idx ON creator_edit(editgroup_id);
-- CREATE INDEX IF NOT EXISTS container_edit_editgroup_idx ON container_edit(editgroup_id);
-- CREATE INDEX IF NOT EXISTS file_edit_editgroup_idx ON file_edit(editgroup_id);
-- CREATE INDEX IF NOT EXISTS fileset_edit_editgroup_idx ON fileset_edit(editgroup_id);
-- CREATE INDEX IF NOT EXISTS webcapture_edit_editgroup_idx ON webcapture_edit(editgroup_id);
-- CREATE INDEX IF NOT EXISTS release_edit_editgroup_idx ON release_edit(editgroup_id);
-- CREATE INDEX IF NOT EXISTS work_edit_editgroup_idx ON work_edit(editgroup_id);
