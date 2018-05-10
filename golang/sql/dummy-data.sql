
-- Fake data at the raw SQL level, for early development and testing

BEGIN;

INSERT INTO editor (id, username, is_admin) VALUES
    (1, 'admin', true),
    (2, 'claire', true),
    (3, 'doug', false);

INSERT INTO editgroup (id, editor_id, description) VALUES
    (1, 1, 'first edit ever!'),
    (2, 1, 'another one!'),
    (3, 3, 'user edit'),
    (4, 2, 'uncommited edit');

INSERT INTO editor (id, username, is_admin, active_editgroup_id) VALUES
    (4, 'bnewbold', true, 4);

INSERT INTO changelog (id, editgroup_id) VALUES
    (1, 1),
    (2, 2),
    (3, 3);

INSERT INTO creator_rev (id, name, orcid) VALUES
    (1, 'Grace Hopper', null),
    (2, 'Emily Noethe', null),
    (3, 'Christine Moran', '0000-0003-2088-7465');

INSERT INTO creator_ident (id, is_live, rev_id, redirect_id) VALUES
    ('f1f046a3-45c9-4b99-adce-000000000001', true, 1, null),
    ('f1f046a3-45c9-4b99-adce-000000000002', true, 2, null),
    ('f1f046a3-45c9-4b99-adce-000000000003', true, 3, null),
    ('f1f046a3-45c9-4b99-adce-000000000004', false, 2, null);

INSERT INTO creator_edit (id, ident_id, rev_id, redirect_id, editgroup_id) VALUES
    (1, 'f1f046a3-45c9-4b99-adce-000000000001', 1, null, 1),
    (2, 'f1f046a3-45c9-4b99-adce-000000000002', 2, null, 2),
    (3, 'f1f046a3-45c9-4b99-adce-000000000003', 3, null, 3),
    (4, 'f1f046a3-45c9-4b99-adce-000000000004', 2, null, 4);

COMMIT;
