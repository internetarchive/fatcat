-- Small SQL changes
-- Part of v0.4.0 (October 2021) changes to the Fatcat API

-------------------- Container ---------------------------------------------

ALTER TABLE container_rev
-- fixed size identifiers
ADD COLUMN issne               TEXT CHECK (octet_length(issne) = 9),
ADD COLUMN issnp               TEXT CHECK (octet_length(issnp) = 9),
ADD COLUMN publication_status  TEXT CHECK (octet_length(publication_status) >= 1);

CREATE INDEX container_rev_issne_idx ON container_rev(issne);
CREATE INDEX container_rev_issnp_idx ON container_rev(issnp);


-------------------- Fileset -----------------------------------------------

ALTER TABLE fileset_rev_file
ADD COLUMN mimetype            TEXT CHECK (octet_length(mimetype) >= 1);


-------------------- Update Test Revs --------------------------------------
-- IMPORTANT: don't create new entities here, only mutate existing

BEGIN;

UPDATE container_rev SET
    issne = '1234-3333',
    issnp = '1234-6666',
    publication_status = 'active'
WHERE id = '00000000-0000-0000-1111-FFF000000002';

UPDATE container_rev SET
    issne = '1549-1676',
    issnp = '1549-1277',
    publication_status = 'active'
WHERE id = '00000000-0000-0000-1111-FFF000000003';

INSERT INTO release_rev_extid (release_rev, extid_type, value) VALUES
    ('00000000-0000-0000-4444-FFF000000002', 'hdl',     '20.500.23456/abc/dummy');

UPDATE fileset_rev_file SET mimetype = 'application/gzip' WHERE fileset_rev = '00000000-0000-0000-6666-fff000000003' and md5 = 'f4de91152c7ab9fdc2a128f962faebff';

COMMIT;
