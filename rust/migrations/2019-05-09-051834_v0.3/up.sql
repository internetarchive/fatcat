
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
ADD COLUMN withdrawn_status       TEXT,  -- TODO: enum?
ADD COLUMN withdrawn_date        DATE,
ADD COLUMN withdrawn_year        BIGINT;

-- create new, separate 
CREATE TABLE release_rev_extid (
    release_rev     UUID REFERENCES release_rev(id) NOT NULL,
    extid_type      TEXT NOT NULL CHECK (octet_length(extid_type) >= 1),
    value           TEXT NOT NULL CHECK (octet_length(value) >= 1),
    PRIMARY KEY(release_rev, extid_type)
);

CREATE INDEX release_rev_extid_type_value_idx ON release_rev_extid(extid_type, value);

-- remove now-unused identifier columns
DROP INDEX IF EXISTS release_rev_isbn13_idx;
DROP INDEX IF EXISTS release_rev_arxiv_idx;
DROP INDEX IF EXISTS release_rev_jstor_idx;

ALTER TABLE release_rev
DROP COLUMN isbn13,
DROP COLUMN arxiv_id,
DROP COLUMN jstor_id;

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


-------------------- Update Test Revs --------------------------------------

-- IMPORTANT: don't create new entities here, only mutate existing

BEGIN;

UPDATE release_rev SET
    subtitle = 'and here a reasonable-length subtitle',
    withdrawn_status = 'withdrawn',
    withdrawn_date = '2018-06-07',
    withdrawn_year = '2018',
    number = 'RN9594',
    version = 'v12'
WHERE id = '00000000-0000-0000-4444-FFF000000002';

INSERT INTO release_rev_extid (release_rev, extid_type, value) VALUES
    ('00000000-0000-0000-4444-FFF000000002', 'isbn13',  '978-3-16-148410-0'),
    ('00000000-0000-0000-4444-FFF000000002', 'arxiv',   '1905.03769v1'),
    ('00000000-0000-0000-4444-FFF000000002', 'jstor',   '1819117828'),
    ('00000000-0000-0000-4444-FFF000000002', 'ark',     'ark:/13030/m53r5pzm'),
    ('00000000-0000-0000-4444-FFF000000002', 'mag',     '992489213');

UPDATE release_contrib SET given_name = 'John', surname = 'Ioannidis' WHERE release_rev = '00000000-0000-0000-4444-FFF000000003';

UPDATE webcapture_rev_cdx SET size_bytes = 12345 WHERE webcapture_rev = '00000000-0000-0000-7777-FFF000000003';

commit;
