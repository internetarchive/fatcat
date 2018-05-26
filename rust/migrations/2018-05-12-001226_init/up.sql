-- written for Postgres 9.6 with OSSP extension for UUIDs

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


-- uuid_generate_v1mc: timestamp ordered, random MAC address
-- uuid_generate_v4:   totally random

-- NB: could use LIKE clause, or "composite types"

CREATE TABLE editor (
    id                  BIGSERIAL PRIMARY KEY,
    username            TEXT NOT NULL UNIQUE,
    is_admin            BOOLEAN NOT NULL DEFAULT false,
    active_editgroup_id BIGINT -- REFERENCES( editgroup(id) via ALTER below
);

CREATE TABLE editgroup (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,
    editor_id           BIGSERIAL REFERENCES editor(id) NOT NULL,
    description         TEXT
);

ALTER TABLE editor
    ADD CONSTRAINT editor_editgroupid_fkey FOREIGN KEY (active_editgroup_id)
        REFERENCES editgroup(id);

CREATE TABLE changelog (
    id                  BIGSERIAL PRIMARY KEY,
    editgroup_id        BIGINT REFERENCES editgroup(id) NOT NULL,
    timestamp           TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL
);

-------------------- Creators -----------------------------------------------
CREATE TABLE creator_rev (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,

    name                TEXT NOT NULL,
    orcid               TEXT
);

-- Could denormalize a "is_live" flag into revision tables, to make indices
-- more efficient
CREATE INDEX creator_rev_orcid_idx ON creator_rev(orcid) WHERE orcid IS NOT NULL;

CREATE TABLE creator_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              BIGINT REFERENCES creator_rev(id),
    redirect_id         UUID REFERENCES creator_ident(id)
);

CREATE TABLE creator_edit (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,
    ident_id            UUID REFERENCES creator_ident(id) NOT NULL,
    rev_id              BIGINT REFERENCES creator_rev(id),
    redirect_id         UUID REFERENCES creator_ident(id),
    editgroup_id        BIGINT REFERENCES editgroup(id) NOT NULL
);

-------------------- Containers --------------------------------------------
CREATE TABLE container_rev (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,

    name                TEXT NOT NULL,
    publisher           TEXT,
    issn                TEXT -- TODO: varchar
);

CREATE INDEX container_rev_issn_idx ON container_rev(issn) WHERE issn IS NOT NULL;

CREATE TABLE container_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              BIGINT REFERENCES container_rev(id),
    redirect_id         UUID REFERENCES container_ident(id)
);

CREATE TABLE container_edit (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,
    ident_id            UUID REFERENCES container_ident(id) NOT NULL,
    rev_id              BIGINT REFERENCES container_rev(id),
    redirect_id         UUID REFERENCES container_ident(id),
    editgroup_id        BIGINT REFERENCES editgroup(id) NOT NULL
);

-------------------- Files -------------------------------------------------
CREATE TABLE file_rev (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,

    size                BIGINT,
    sha1                TEXT, -- TODO: varchar or bytes
    url                 TEXT  -- TODO: URL table
);

CREATE TABLE file_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              BIGINT REFERENCES file_rev(id),
    redirect_id         UUID REFERENCES file_ident(id)
);

CREATE TABLE file_edit (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,
    ident_id            UUID REFERENCES file_ident(id) NOT NULL,
    rev_id              BIGINT REFERENCES file_rev(id),
    redirect_id         UUID REFERENCES file_ident(id),
    editgroup_id        BIGINT REFERENCES editgroup(id) NOT NULL
);

-------------------- Release -----------------------------------------------
CREATE TABLE release_rev (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,

    work_ident_id       UUID NOT NULL, -- FOREIGN KEY; see ALRTER below
    container_ident_id  UUID REFERENCES container_ident(id),
    title               TEXT NOT NULL,
    release_type        TEXT, -- TODO: enum
    date                DATE,
    doi                 TEXT,
    volume              TEXT,
    pages               TEXT,
    issue               TEXT
    -- TODO: identifier table?
);

CREATE INDEX release_rev_doi_idx ON release_rev(doi) WHERE doi IS NOT NULL;

CREATE TABLE release_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              BIGINT REFERENCES release_rev(id),
    redirect_id         UUID REFERENCES release_ident(id)
);

CREATE TABLE release_edit (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,
    ident_id            UUID REFERENCES release_ident(id) NOT NULL,
    rev_id              BIGINT REFERENCES release_rev(id),
    redirect_id         UUID REFERENCES release_ident(id),
    editgroup_id        BIGINT REFERENCES editgroup(id) NOT NULL
);

-------------------- Works --------------------------------------------------
CREATE TABLE work_rev (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,

    -- not even a work, for now
    work_type           TEXT, -- TODO: enum?
    primary_release_id  UUID REFERENCES release_ident(id)
);

CREATE TABLE work_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              BIGINT REFERENCES work_rev(id),
    redirect_id         UUID REFERENCES work_ident(id)
);

CREATE TABLE work_edit (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,
    ident_id            UUID REFERENCES work_ident(id) NOT NULL,
    rev_id              BIGINT REFERENCES work_rev(id),
    redirect_id         UUID REFERENCES work_ident(id),
    editgroup_id        BIGINT REFERENCES editgroup(id) NOT NULL
);


ALTER TABLE release_rev
    ADD CONSTRAINT release_containeridentid_fkey FOREIGN KEY (work_ident_id)
            REFERENCES work_ident(id);

-------------------- Inter-Entity Relations

CREATE TABLE release_contrib (
    id                  BIGSERIAL PRIMARY KEY,
    release_rev         BIGSERIAL REFERENCES release_rev(id) NOT NULL,
    creator_ident_id    UUID REFERENCES creator_ident(id),
    contrib_type        TEXT,
    index               BIGINT,
    stub                TEXT
);

CREATE TABLE release_ref (
    id                      BIGSERIAL PRIMARY KEY,
    release_rev             BIGSERIAL REFERENCES release_rev(id) NOT NULL,
    target_release_ident_id UUID REFERENCES release_ident(id), -- or work?
    index                   BIGINT,
    stub                    TEXT
);

CREATE TABLE file_release (
    file_rev                BIGSERIAL REFERENCES file_rev(id) NOT NULL,
    target_release_ident_id UUID REFERENCES release_ident(id) NOT NULL,
    PRIMARY KEY (file_rev, target_release_ident_id)
);

---------------------------------------------------------------------------
---------------------------------------------------------------------------
---------------------------------------------------------------------------

-- Fake data at the raw SQL level, for early development and testing

BEGIN;

INSERT INTO editor (username, is_admin) VALUES
    ('admin', true),
    ('claire', true),
    ('doug', false);

INSERT INTO editgroup (editor_id, description) VALUES
    (1, 'first edit ever!'),
    (1, 'another one!'),
    (3, 'user edit'),
    (2, 'uncommited edit'),
    (1, 'journal edit'),
    (1, 'another journal edit');

INSERT INTO editor (username, is_admin, active_editgroup_id) VALUES
    ('bnewbold', true, 4);

INSERT INTO changelog (editgroup_id) VALUES
    (1),
    (2),
    (3),
    (4),
    (5);

INSERT INTO container_rev (name, issn) VALUES
    ('Journal of Trivial Results', '1234-5678'),
    ('MySpace Blog', null);

INSERT INTO container_ident (id, is_live, rev_id, redirect_id) VALUES
    ('f1f046a3-45c9-4b99-cccc-000000000001', true, 1, null),
    ('f1f046a3-45c9-4b99-cccc-000000000002', true, 2, null);

INSERT INTO container_edit (ident_id, rev_id, redirect_id, editgroup_id) VALUES
    ('f1f046a3-45c9-4b99-cccc-000000000001', 1, null, 4),
    ('f1f046a3-45c9-4b99-cccc-000000000002', 2, null, 5);

INSERT INTO creator_rev (name, orcid) VALUES
    ('Grace Hopper', null),
    ('Emily Noethe', null),
    ('Christine Moran', '0000-0003-2088-7465');

INSERT INTO creator_ident (id, is_live, rev_id, redirect_id) VALUES
    ('f1f046a3-45c9-4b99-adce-000000000001', true, 1, null),
    ('f1f046a3-45c9-4b99-adce-000000000002', true, 2, null),
    ('f1f046a3-45c9-4b99-adce-000000000003', true, 3, null),
    ('f1f046a3-45c9-4b99-adce-000000000004', false, 2, null);

INSERT INTO creator_edit (ident_id, rev_id, redirect_id, editgroup_id) VALUES
    ('f1f046a3-45c9-4b99-adce-000000000001', 1, null, 1),
    ('f1f046a3-45c9-4b99-adce-000000000002', 2, null, 2),
    ('f1f046a3-45c9-4b99-adce-000000000003', 3, null, 3),
    ('f1f046a3-45c9-4b99-adce-000000000004', 2, null, 4);

INSERT INTO file_rev (size, sha1, url) VALUES
    (null, null, null),
    (4321, '7d97e98f8af710c7e7fe703abc8f639e0ee507c4', 'http://archive.org/robots.txt');

INSERT INTO file_ident (id, is_live, rev_id, redirect_id) VALUES
    ('f1f046a3-45c9-4b99-ffff-000000000001', true, 1, null),
    ('f1f046a3-45c9-4b99-ffff-000000000002', true, 2, null);

INSERT INTO file_edit (ident_id, rev_id, redirect_id, editgroup_id) VALUES
    ('f1f046a3-45c9-4b99-ffff-000000000001', 1, null, 4),
    ('f1f046a3-45c9-4b99-ffff-000000000002', 2, null, 5);

INSERT INTO work_rev (work_type, primary_release_id) VALUES
    (null, null),
    ('journal-article', null);

INSERT INTO work_ident (id, is_live, rev_id, redirect_id) VALUES
    ('f1f046a3-45c9-4b99-3333-000000000001', true, 1, null),
    ('f1f046a3-45c9-4b99-3333-000000000002', true, 2, null);

INSERT INTO work_edit (ident_id, rev_id, redirect_id, editgroup_id) VALUES
    ('f1f046a3-45c9-4b99-3333-000000000001', 1, null, 4),
    ('f1f046a3-45c9-4b99-3333-000000000002', 2, null, 5);

INSERT INTO release_rev (work_ident_id, container_ident_id, title, release_type, date, doi, volume, pages, issue) VALUES
    ('f1f046a3-45c9-4b99-3333-000000000001',                                   null,  'example title',              null,         null,         null,  null,  null, null),
    ('f1f046a3-45c9-4b99-3333-000000000002', 'f1f046a3-45c9-4b99-cccc-000000000001', 'bigger example', 'journal-article', '2018-01-01', '10.123/abc',  '12', '5-9', 'IV');

INSERT INTO release_ident (id, is_live, rev_id, redirect_id) VALUES
    ('f1f046a3-45c9-4b99-4444-000000000001', true, 1, null),
    ('f1f046a3-45c9-4b99-4444-000000000002', true, 2, null);

INSERT INTO release_edit (ident_id, rev_id, redirect_id, editgroup_id) VALUES
    ('f1f046a3-45c9-4b99-4444-000000000001', 1, null, 4),
    ('f1f046a3-45c9-4b99-4444-000000000002', 2, null, 5);

INSERT INTO release_contrib (release_rev, creator_ident_id, stub, contrib_type, index) VALUES
    (2, null, null, null, null),
    (2, 'f1f046a3-45c9-4b99-adce-000000000002', 'some contrib', 'editor', 4);

INSERT INTO release_ref (release_rev, target_release_ident_id, index, stub) VALUES
    (2, null, null, null),
    (2, 'f1f046a3-45c9-4b99-4444-000000000001', 4, 'citation note');

INSERT INTO file_release (file_rev, target_release_ident_id) VALUES
    (2, 'f1f046a3-45c9-4b99-4444-000000000002');

COMMIT;
