
-- written for Postgres 9.6 with OSSP extension for UUIDs

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

DROP TABLE IF EXISTS editor CASCADE;
DROP TABLE IF EXISTS editgroup CASCADE;
DROP TABLE IF EXISTS changelog CASCADE;
DROP TABLE IF EXISTS creator_rev CASCADE;
DROP TABLE IF EXISTS creator_ident CASCADE;
DROP TABLE IF EXISTS creator_edit CASCADE;

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
