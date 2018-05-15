-- written for Postgres 9.6 with OSSP extension for UUIDs

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


-- uuid_generate_v1mc: timestamp ordered, random MAC address
-- uuid_generate_v4:   totally random

-- NB: could use LIKE clause, or "composite types"

CREATE TABLE editor (
    id                  BIGSERIAL PRIMARY KEY,
    username            TEXT NOT NULL,
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
    timestamp           TIMESTAMP WITHOUT TIME ZONE DEFAULT now()
);

-------------------- Creators -----------------------------------------------
CREATE TABLE creator_rev (
    id                  BIGSERIAL PRIMARY KEY,
    extra_json          JSON,

    name                TEXT,
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

    name                TEXT,
    parent_ident_id     BIGINT REFERENCES container_rev(id),
    publisher           TEXT,
    issn                TEXT -- TODO: varchar
);

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

    size                INTEGER, -- TODO: uint64
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

    work_ident_id       UUID, -- FOREIGN KEY; see ALRTER below
    container_ident_id  UUID REFERENCES container_ident(id),
    title               TEXT,
    license             TEXT, -- TODO: ?
    release_type        TEXT, -- TODO: enum
    date                TEXT, -- XXX: datetime
    doi                 TEXT, -- TODO: identifier table?
    volume              TEXT,
    pages               TEXT,
    issue               TEXT
);

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
    release_rev         BIGSERIAL REFERENCES release_rev(id) NOT NULL,
    creator_ident_id    UUID REFERENCES creator_ident(id),
    stub                TEXT,
    contrib_type        TEXT
);

CREATE TABLE release_ref (
    id                      BIGSERIAL PRIMARY KEY,
    release_rev             BIGSERIAL REFERENCES release_rev(id) NOT NULL,
    target_release_ident_id UUID REFERENCES creator_ident(id),
    index                   INTEGER,
    stub                    TEXT
);

CREATE TABLE file_release (
    file_rev                BIGSERIAL REFERENCES file_rev(id) NOT NULL,
    target_release_ident_id UUID REFERENCES creator_ident(id) NOT NULL
);
