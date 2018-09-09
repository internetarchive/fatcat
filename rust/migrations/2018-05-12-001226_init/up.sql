-- written for Postgres 9.6 with OSSP extension for UUIDs -- ... but actually runs on Postgres 10 in qa/production

-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


-- uuid_generate_v1mc: timestamp ordered, random MAC address
-- gen_random_uuid:   totally random

-- NB: could use LIKE clause, or "composite types"

CREATE TABLE editor (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username            TEXT NOT NULL UNIQUE,
    is_admin            BOOLEAN NOT NULL DEFAULT false,
    registered          TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL,
    active_editgroup_id UUID -- REFERENCES( editgroup(id) via ALTER below
);

CREATE INDEX active_editgroup_idx ON editor(active_editgroup_id);

CREATE TABLE editgroup (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    editor_id           UUID REFERENCES editor(id) NOT NULL,
    created             TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL,
    extra_json          JSONB,
    description         TEXT
);

ALTER TABLE editor
    ADD CONSTRAINT editor_editgroupid_fkey FOREIGN KEY (active_editgroup_id)
        REFERENCES editgroup(id);

CREATE TABLE changelog (
    id                  BIGSERIAL PRIMARY KEY,
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    timestamp           TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL
);

-- for "is this editgroup merged" queries
CREATE INDEX changelog_editgroup_idx ON changelog(editgroup_id);

CREATE TABLE abstracts (
    sha1                CHAR(40) PRIMARY KEY,
    content             TEXT NOT NULL
);

-------------------- Creators -----------------------------------------------
CREATE TABLE creator_rev (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    extra_json          JSONB,

    display_name        TEXT NOT NULL,
    given_name          TEXT,
    surname             TEXT,
    orcid               CHAR(19),
    wikidata_qid        TEXT
    -- TODO: viaf                TEXT,
    -- TODO: aliases/alternatives
    -- TODO: sortable name?
);

-- Could denormalize a "is_live" flag into revision tables, to make indices
-- more efficient
CREATE INDEX creator_rev_orcid_idx ON creator_rev(orcid); -- WHERE orcid IS NOT NULL;
CREATE INDEX creator_rev_wikidata_idx ON creator_rev(wikidata_qid); -- WHERE wikidata_qid IS NOT NULL;

CREATE TABLE creator_ident (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES creator_rev(id),
    redirect_id         UUID REFERENCES creator_ident(id)
);

CREATE INDEX creator_ident_rev_idx ON creator_ident(rev_id);

CREATE TABLE creator_edit (
    id                  BIGSERIAL PRIMARY KEY,
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES creator_ident(id) NOT NULL,
    rev_id              UUID REFERENCES creator_rev(id),
    redirect_id         UUID REFERENCES creator_ident(id),
    prev_rev            UUID REFERENCES creator_rev(id),
    extra_json          JSONB
);

CREATE INDEX creator_edit_idx ON creator_edit(editgroup_id);

-------------------- Containers --------------------------------------------
CREATE TABLE container_rev (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    extra_json          JSONB,

    name                TEXT NOT NULL,
    publisher           TEXT,
    issnl               CHAR(9),
    wikidata_qid        TEXT,
    abbrev              TEXT,
    coden               TEXT
);

CREATE INDEX container_rev_issnl_idx ON container_rev(issnl); -- WHERE issnl IS NOT NULL;
CREATE INDEX container_rev_wikidata_idx ON container_rev(wikidata_qid); -- WHERE wikidata_qid IS NOT NULL;

CREATE TABLE container_ident (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES container_rev(id),
    redirect_id         UUID REFERENCES container_ident(id)
);

CREATE INDEX container_ident_rev_idx ON container_ident(rev_id);

CREATE TABLE container_edit (
    id                  BIGSERIAL PRIMARY KEY,
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES container_ident(id) NOT NULL,
    rev_id              UUID REFERENCES container_rev(id),
    redirect_id         UUID REFERENCES container_ident(id),
    prev_rev            UUID REFERENCES container_rev(id),
    extra_json          JSONB
);

CREATE INDEX container_edit_idx ON container_edit(editgroup_id);

-------------------- Files -------------------------------------------------
CREATE TABLE file_rev (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    extra_json          JSONB,

    size                BIGINT,
    sha1                CHAR(40),
    sha256              CHAR(64),
    md5                 CHAR(32),
    mimetype            TEXT
);

CREATE INDEX file_rev_sha1_idx ON file_rev(sha1); -- WHERE sha1 IS NOT NULL;
CREATE INDEX file_rev_md5_idx ON file_rev(md5); -- WHERE md5 IS NOT NULL;
CREATE INDEX file_rev_sha256_idx ON file_rev(sha256); -- WHERE sha256 IS NOT NULL;

CREATE TABLE file_rev_url (
    id                  BIGSERIAL PRIMARY KEY,
    file_rev            UUID REFERENCES file_rev(id) NOT NULL,
    rel                 TEXT NOT NULL, -- TODO: enum? web, webarchive, repo, etc TODO: default web?
    url                 TEXT NOT NULL
);

CREATE INDEX file_rev_url_rev_idx ON file_rev_url(file_rev);

CREATE TABLE file_ident (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES file_rev(id),
    redirect_id         UUID REFERENCES file_ident(id)
);

CREATE INDEX file_ident_rev_idx ON file_ident(rev_id);

CREATE TABLE file_edit (
    id                  BIGSERIAL PRIMARY KEY,
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES file_ident(id) NOT NULL,
    rev_id              UUID REFERENCES file_rev(id),
    redirect_id         UUID REFERENCES file_ident(id),
    prev_rev            UUID REFERENCES file_rev(id),
    extra_json          JSONB
);

CREATE INDEX file_edit_idx ON file_edit(editgroup_id);

-------------------- Release -----------------------------------------------
CREATE TABLE release_rev (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    extra_json          JSONB,

    work_ident_id       UUID NOT NULL, -- FOREIGN KEY; see ALRTER below
    container_ident_id  UUID REFERENCES container_ident(id),
    title               TEXT NOT NULL,
    release_type        TEXT, -- TODO: enum
    release_status      TEXT, -- TODO: enum
    release_date        DATE,
    doi                 TEXT,
    pmid                TEXT, -- TODO: BIGINT?
    pmcid               TEXT, -- TODO: BIGINT?
    wikidata_qid        TEXT, -- TODO: BIGINT?
    isbn13              TEXT, -- TODO: fixed-length
    core_id             TEXT, -- TODO: BIGINT?
    volume              TEXT,
    issue               TEXT,
    pages               TEXT,
    publisher           TEXT, -- for books, NOT if container exists
    language            TEXT  -- primary language of the work's fulltext; RFC1766/ISO639-1
    -- TODO: oclc_ocn (TEXT or BIGINT)
    -- TODO: identifier table?
);

CREATE INDEX release_rev_doi_idx ON release_rev(doi); -- WHERE doi IS NOT NULL;
CREATE INDEX release_rev_pmid_idx ON release_rev(pmid); -- WHERE pmid IS NOT NULL;
CREATE INDEX release_rev_pmcid_idx ON release_rev(pmcid); -- WHERE pmcid IS NOT NULL;
CREATE INDEX release_rev_wikidata_idx ON release_rev(wikidata_qid); -- WHERE wikidata_qid IS NOT NULL;
CREATE INDEX release_rev_isbn13_idx ON release_rev(isbn13); -- WHERE isbn13 IS NOT NULL;
CREATE INDEX release_rev_core_idx ON release_rev(core_id); -- WHERE core_id IS NOT NULL;
CREATE INDEX release_rev_work_idx ON release_rev(work_ident_id); -- WHERE work_ident_id IS NOT NULL;

CREATE TABLE release_rev_abstract (
    id              BIGSERIAL PRIMARY KEY,
    release_rev     UUID REFERENCES release_rev(id) NOT NULL,
    abstract_sha1   CHAR(40) REFERENCES abstracts(sha1) NOT NULL,
    mimetype        TEXT,
    lang            TEXT
);

CREATE TABLE release_ident (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES release_rev(id),
    redirect_id         UUID REFERENCES release_ident(id)
);

CREATE INDEX release_ident_rev_idx ON release_ident(rev_id);

CREATE TABLE release_edit (
    id                  BIGSERIAL PRIMARY KEY,
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES release_ident(id) NOT NULL,
    rev_id              UUID REFERENCES release_rev(id),
    redirect_id         UUID REFERENCES release_ident(id),
    prev_rev            UUID REFERENCES release_rev(id),
    extra_json          JSONB
);

CREATE INDEX release_edit_idx ON release_edit(editgroup_id);

-------------------- Works --------------------------------------------------
CREATE TABLE work_rev (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    extra_json          JSONB
);

CREATE TABLE work_ident (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES work_rev(id),
    redirect_id         UUID REFERENCES work_ident(id)
);

CREATE INDEX work_ident_rev_idx ON work_ident(rev_id);

CREATE TABLE work_edit (
    id                  BIGSERIAL PRIMARY KEY,
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES work_ident(id) NOT NULL,
    rev_id              UUID REFERENCES work_rev(id),
    redirect_id         UUID REFERENCES work_ident(id),
    prev_rev            UUID REFERENCES work_rev(id),
    extra_json          JSONB
);

CREATE INDEX work_edit_idx ON work_edit(editgroup_id);

ALTER TABLE release_rev
    ADD CONSTRAINT release_containeridentid_fkey FOREIGN KEY (work_ident_id)
            REFERENCES work_ident(id);

-------------------- Inter-Entity Relations

CREATE TABLE release_contrib (
    id                  BIGSERIAL PRIMARY KEY,
    release_rev         UUID REFERENCES release_rev(id) NOT NULL,
    creator_ident_id    UUID REFERENCES creator_ident(id),
    raw_name            TEXT,
    role                TEXT, -- TODO: enum?
    index_val           BIGINT,
    extra_json          JSONB
);

CREATE INDEX release_contrib_rev_idx ON release_contrib(release_rev);
CREATE INDEX release_contrib_creator_idx ON release_contrib(creator_ident_id);

CREATE TABLE release_ref (
    id                      BIGSERIAL PRIMARY KEY,
    release_rev             UUID REFERENCES release_rev(id) NOT NULL,
    target_release_ident_id UUID REFERENCES release_ident(id), -- or work?
    index_val               BIGINT,
    key                     TEXT,
    extra_json              JSONB, -- title, year, container_title, locator (aka, page), oci_id
    container_title         TEXT,
    year                    BIGINT,
    title                   TEXT,
    locator                 TEXT
    -- TODO: oci_id (TEXT)
);

CREATE INDEX release_ref_rev_idx ON release_ref(release_rev);
CREATE INDEX release_ref_target_release_idx ON release_ref(target_release_ident_id);

CREATE TABLE file_release (
    file_rev                UUID REFERENCES file_rev(id) NOT NULL,
    target_release_ident_id UUID REFERENCES release_ident(id) NOT NULL,
    PRIMARY KEY (file_rev, target_release_ident_id)
);

CREATE INDEX file_release_target_release_idx ON file_release(target_release_ident_id);

---------------------------------------------------------------------------
---------------------------------------------------------------------------
---------------------------------------------------------------------------

-- Fake data at the raw SQL level, for early development and testing
-- Convention:
--  * first entity is smallest possible (mostly null)
--  * second entity is rich (all fields/relations designed) but artificial
--  * third entity (and above) are realistic (real DOI, etc)

BEGIN;

INSERT INTO editor (id, username, is_admin) VALUES
    ('00000000-0000-0000-AAAA-000000000001', 'admin', true),        -- aaaaaaaaaaaabkvkaaaaaaaaae
    ('00000000-0000-0000-AAAA-000000000002', 'demo-user', true),    -- aaaaaaaaaaaabkvkaaaaaaaaai
    ('00000000-0000-0000-AAAA-000000000003', 'claire', false);      -- aaaaaaaaaaaabkvkaaaaaaaaam

INSERT INTO editgroup (id, editor_id, description) VALUES
    ('00000000-0000-0000-BBBB-000000000001', '00000000-0000-0000-AAAA-000000000001', 'first edit ever!'),       -- aaaaaaaaaaaabo53aaaaaaaaae
    ('00000000-0000-0000-BBBB-000000000002', '00000000-0000-0000-AAAA-000000000001', 'another one!'),           -- aaaaaaaaaaaabo53aaaaaaaaai
    ('00000000-0000-0000-BBBB-000000000003', '00000000-0000-0000-AAAA-000000000003', 'user edit'),              -- aaaaaaaaaaaabo53aaaaaaaaam
    ('00000000-0000-0000-BBBB-000000000004', '00000000-0000-0000-AAAA-000000000002', 'uncommited edit'),        -- aaaaaaaaaaaabo53aaaaaaaaaq
    ('00000000-0000-0000-BBBB-000000000005', '00000000-0000-0000-AAAA-000000000001', 'journal edit'),           -- aaaaaaaaaaaabo53aaaaaaaaau
    ('00000000-0000-0000-BBBB-000000000006', '00000000-0000-0000-AAAA-000000000001', 'another journal edit');   -- aaaaaaaaaaaabo53aaaaaaaaay

INSERT INTO editor (id, username, is_admin, active_editgroup_id) VALUES
    ('00000000-0000-0000-AAAA-000000000004', 'bnewbold', true, '00000000-0000-0000-BBBB-000000000004');

INSERT INTO changelog (editgroup_id) VALUES
    ('00000000-0000-0000-BBBB-000000000001'),
    ('00000000-0000-0000-BBBB-000000000002'),
    ('00000000-0000-0000-BBBB-000000000003'),
    ('00000000-0000-0000-BBBB-000000000004'),
    ('00000000-0000-0000-BBBB-000000000005');

INSERT INTO abstracts (sha1, content) VALUES
    ('1ba86bf8c2979a62d29b18b537e50b2b093be27e', 'some long abstract in plain text'),
    ('0da908ab584b5e445a06beb172e3fab8cb5169e3', '<jats>A longer, more correct abstract should in theory go here</jats>');

INSERT INTO container_rev (id, name, publisher, issnl, abbrev, coden, extra_json) VALUES
    ('00000000-0000-0000-1111-FFF000000001', 'MySpace Blog', null, null, null, null, null),
    ('00000000-0000-0000-1111-FFF000000002', 'Journal of Trivial Results', 'bogus publishing group', '1234-5678', 'Triv. Res.', 'CDNXYZ', '{"is_oa": false, "in_doaj": false}'),
    ('00000000-0000-0000-1111-FFF000000003', 'PLOS Medicine', 'Public Library of Science', '1549-1277', 'PLoS med.', null, '{"is_oa": true, "in_doaj": true}');

INSERT INTO container_ident (id, is_live, rev_id, redirect_id) VALUES
    ('00000000-0000-0000-1111-000000000001', true, '00000000-0000-0000-1111-FFF000000001', null), -- aaaaaaaaaaaaaeiraaaaaaaaae
    ('00000000-0000-0000-1111-000000000002', true, '00000000-0000-0000-1111-FFF000000002', null), -- aaaaaaaaaaaaaeiraaaaaaaaai
    ('00000000-0000-0000-1111-000000000003', true, '00000000-0000-0000-1111-FFF000000003', null); -- aaaaaaaaaaaaaeiraaaaaaaaam

INSERT INTO container_edit (ident_id, rev_id, redirect_id, editgroup_id, prev_rev) VALUES
    ('00000000-0000-0000-1111-000000000001', '00000000-0000-0000-1111-FFF000000001', null, '00000000-0000-0000-BBBB-000000000003', null),
    ('00000000-0000-0000-1111-000000000002', '00000000-0000-0000-1111-FFF000000002', null, '00000000-0000-0000-BBBB-000000000004', null),
    ('00000000-0000-0000-1111-000000000003', '00000000-0000-0000-1111-FFF000000003', null, '00000000-0000-0000-BBBB-000000000005', '00000000-0000-0000-1111-FFF000000002');

INSERT INTO creator_rev (id, display_name, given_name, surname, orcid, wikidata_qid) VALUES
    ('00000000-0000-0000-2222-FFF000000001', 'Grace Hopper', null, null, null, null),
    ('00000000-0000-0000-2222-FFF000000002', 'Christine Moran', 'Christine', 'Moran', '0000-0003-2088-7465', 'Q1234'),
    ('00000000-0000-0000-2222-FFF000000003', 'John P. A. Ioannidis', 'John', 'Ioannidis', '0000-0003-3118-6859', 'Q5678');

INSERT INTO creator_ident (id, is_live, rev_id, redirect_id) VALUES
    ('00000000-0000-0000-2222-000000000001', true,  '00000000-0000-0000-2222-FFF000000001', null), -- aaaaaaaaaaaaaircaaaaaaaaae
    ('00000000-0000-0000-2222-000000000002', true,  '00000000-0000-0000-2222-FFF000000002', null), -- aaaaaaaaaaaaaircaaaaaaaaai
    ('00000000-0000-0000-2222-000000000003', true,  '00000000-0000-0000-2222-FFF000000003', null), -- aaaaaaaaaaaaaircaaaaaaaaam
    ('00000000-0000-0000-2222-000000000004', false, '00000000-0000-0000-2222-FFF000000002', null); -- aaaaaaaaaaaaaircaaaaaaaaaq

INSERT INTO creator_edit (ident_id, rev_id, redirect_id, editgroup_id, prev_rev) VALUES
    ('00000000-0000-0000-2222-000000000001', '00000000-0000-0000-2222-FFF000000001', null, '00000000-0000-0000-BBBB-000000000001', null),
    ('00000000-0000-0000-2222-000000000002', '00000000-0000-0000-2222-FFF000000002', null, '00000000-0000-0000-BBBB-000000000002', null),
    ('00000000-0000-0000-2222-000000000003', '00000000-0000-0000-2222-FFF000000003', null, '00000000-0000-0000-BBBB-000000000003', null),
    ('00000000-0000-0000-2222-000000000004', '00000000-0000-0000-2222-FFF000000002', null, '00000000-0000-0000-BBBB-000000000004', '00000000-0000-0000-2222-FFF000000003');

INSERT INTO file_rev (id, size, sha1, sha256, md5, mimetype) VALUES
    ('00000000-0000-0000-3333-FFF000000001', null, null, null, null, null),
    ('00000000-0000-0000-3333-FFF000000002', 4321, '7d97e98f8af710c7e7fe703abc8f639e0ee507c4', null, null, 'text/plain'),
    ('00000000-0000-0000-3333-FFF000000003', 255629, '3f242a192acc258bdfdb151943419437f440c313', 'ffc1005680cb620eec4c913437dfabbf311b535cfe16cbaeb2faec1f92afc362', 'f4de91152c7ab9fdc2a128f962faebff', 'application/pdf');

INSERT INTO file_rev_url (file_rev, rel, url) VALUES
    ('00000000-0000-0000-3333-FFF000000002', 'web', 'http://archive.org/robots.txt'),
    ('00000000-0000-0000-3333-FFF000000003', 'publisher', 'http://journals.plos.org/plosmedicine/article/file?id=10.1371/journal.pmed.0020124&type=printable');

INSERT INTO file_ident (id, is_live, rev_id, redirect_id) VALUES
    ('00000000-0000-0000-3333-000000000001', true, '00000000-0000-0000-3333-FFF000000001', null), -- aaaaaaaaaaaaamztaaaaaaaaae
    ('00000000-0000-0000-3333-000000000002', true, '00000000-0000-0000-3333-FFF000000002', null), -- aaaaaaaaaaaaamztaaaaaaaaai
    ('00000000-0000-0000-3333-000000000003', true, '00000000-0000-0000-3333-FFF000000003', null); -- aaaaaaaaaaaaamztaaaaaaaaam

INSERT INTO file_edit (ident_id, rev_id, redirect_id, editgroup_id, prev_rev) VALUES
    ('00000000-0000-0000-3333-000000000001', '00000000-0000-0000-3333-FFF000000001', null, '00000000-0000-0000-BBBB-000000000003', null),
    ('00000000-0000-0000-3333-000000000002', '00000000-0000-0000-3333-FFF000000002', null, '00000000-0000-0000-BBBB-000000000004', null),
    ('00000000-0000-0000-3333-000000000003', '00000000-0000-0000-3333-FFF000000003', null, '00000000-0000-0000-BBBB-000000000005', '00000000-0000-0000-3333-FFF000000002');

INSERT INTO work_rev (id) VALUES
    ('00000000-0000-0000-5555-FFF000000001'),
    ('00000000-0000-0000-5555-FFF000000002'),
    ('00000000-0000-0000-5555-FFF000000003');

INSERT INTO work_ident (id, is_live, rev_id, redirect_id) VALUES
    ('00000000-0000-0000-5555-000000000001', true, '00000000-0000-0000-5555-FFF000000001', null), -- aaaaaaaaaaaaavkvaaaaaaaaae
    ('00000000-0000-0000-5555-000000000002', true, '00000000-0000-0000-5555-FFF000000002', null), -- aaaaaaaaaaaaavkvaaaaaaaaai
    ('00000000-0000-0000-5555-000000000003', true, '00000000-0000-0000-5555-FFF000000003', null); -- aaaaaaaaaaaaavkvaaaaaaaaam

INSERT INTO work_edit (ident_id, rev_id, redirect_id, editgroup_id, prev_rev) VALUES
    ('00000000-0000-0000-5555-000000000001', '00000000-0000-0000-5555-FFF000000001', null, '00000000-0000-0000-BBBB-000000000003', null),
    ('00000000-0000-0000-5555-000000000002', '00000000-0000-0000-5555-FFF000000002', null, '00000000-0000-0000-BBBB-000000000004', null),
    ('00000000-0000-0000-5555-000000000002', '00000000-0000-0000-5555-FFF000000003', null, '00000000-0000-0000-BBBB-000000000005', '00000000-0000-0000-5555-FFF000000002');

INSERT INTO release_rev (id, work_ident_id, container_ident_id, title, release_type, release_status, release_date, doi, isbn13, core_id, volume, issue, pages, publisher, language) VALUES
    ('00000000-0000-0000-4444-FFF000000001', '00000000-0000-0000-5555-000000000001',                                   null,  'example title',              null, null,        null,         null,            null,       null, null, null,  null,                     null, null),
    ('00000000-0000-0000-4444-FFF000000002', '00000000-0000-0000-5555-000000000002', '00000000-0000-0000-1111-000000000001', 'bigger example', 'journal-article', null,'2018-01-01', '10.123/abc', '99999-999-9-X', '42022773', '12', 'IV', '5-9', 'bogus publishing group', 'cn'),
    ('00000000-0000-0000-4444-FFF000000003', '00000000-0000-0000-5555-000000000003', '00000000-0000-0000-1111-000000000003', 'Why Most Published Research Findings Are False', 'journal-article', 'published', '2005-08-30', '10.1371/journal.pmed.0020124',  null, null, '2', '8', 'e124', 'Public Library of Science', 'en');

INSERT INTO release_ident (id, is_live, rev_id, redirect_id) VALUES
    ('00000000-0000-0000-4444-000000000001', true, '00000000-0000-0000-4444-FFF000000001', null), -- aaaaaaaaaaaaarceaaaaaaaaae
    ('00000000-0000-0000-4444-000000000002', true, '00000000-0000-0000-4444-FFF000000002', null), -- aaaaaaaaaaaaarceaaaaaaaaai
    ('00000000-0000-0000-4444-000000000003', true, '00000000-0000-0000-4444-FFF000000003', null); -- aaaaaaaaaaaaarceaaaaaaaaam

INSERT INTO release_edit (ident_id, rev_id, redirect_id, editgroup_id, prev_rev) VALUES
    ('00000000-0000-0000-4444-000000000001', '00000000-0000-0000-4444-FFF000000001', null, '00000000-0000-0000-BBBB-000000000003', null),
    ('00000000-0000-0000-4444-000000000002', '00000000-0000-0000-4444-FFF000000002', null, '00000000-0000-0000-BBBB-000000000004', null),
    ('00000000-0000-0000-4444-000000000003', '00000000-0000-0000-4444-FFF000000003', null, '00000000-0000-0000-BBBB-000000000005', '00000000-0000-0000-4444-FFF000000002');

INSERT INTO release_rev_abstract (release_rev, abstract_sha1, mimetype, lang) VALUES
    ('00000000-0000-0000-4444-FFF000000001', '1ba86bf8c2979a62d29b18b537e50b2b093be27e', 'text/plain', 'en'),
    ('00000000-0000-0000-4444-FFF000000002', '0da908ab584b5e445a06beb172e3fab8cb5169e3', 'application/xml+jats', 'en');

INSERT INTO release_contrib (release_rev, creator_ident_id, raw_name, role, index_val) VALUES
    ('00000000-0000-0000-4444-FFF000000002', null, null, null, null),
    ('00000000-0000-0000-4444-FFF000000002', '00000000-0000-0000-2222-000000000002', 'some contrib', 'editor', 4),
    ('00000000-0000-0000-4444-FFF000000003', '00000000-0000-0000-2222-000000000003', 'John P. A. Ioannidis', 'author', 0);

INSERT INTO release_ref (release_rev, target_release_ident_id, index_val, extra_json) VALUES
    ('00000000-0000-0000-4444-FFF000000002', null, null, null),
    ('00000000-0000-0000-4444-FFF000000002', '00000000-0000-0000-4444-000000000001', 4, '{"unstructured":"citation note"}'),
    ('00000000-0000-0000-4444-FFF000000003', null, 0, '{"unstructured": "Ioannidis JP, Haidich AB, Lau J. Any casualties in the clash of randomised and observational evidence? BMJ. 2001;322:879–880"}'),
    ('00000000-0000-0000-4444-FFF000000003', null, 1, '{"unstructured":"Lawlor DA, Davey Smith G, Kundu D, Bruckdorfer KR, Ebrahim S. Those confounded vitamins: What can we learn from the differences between observational versus randomised trial evidence? Lancet. 2004;363:1724–1727."}'),
    ('00000000-0000-0000-4444-FFF000000003', null, 2, '{"unstructured":"Vandenbroucke JP. When are observational studies as credible as randomised trials? Lancet. 2004;363:1728–1731."}'),
    ('00000000-0000-0000-4444-FFF000000003', null, 3, '{"unstructured":"Michiels S, Koscielny S, Hill C. Prediction of cancer outcome with microarrays: A multiple random validation strategy. Lancet. 2005;365:488–492."}'),
    ('00000000-0000-0000-4444-FFF000000003', null, 4, '{"unstructured":"Ioannidis JPA, Ntzani EE, Trikalinos TA, Contopoulos-Ioannidis DG. Replication validity of genetic association studies. Nat Genet. 2001;29:306–309."}'),
    ('00000000-0000-0000-4444-FFF000000003', null, 5, '{"unstructured":"Colhoun HM, McKeigue PM, Davey Smith G. Problems of reporting genetic associations with complex outcomes. Lancet. 2003;361:865–872."}');

INSERT INTO file_release (file_rev, target_release_ident_id) VALUES
    ('00000000-0000-0000-3333-FFF000000002', '00000000-0000-0000-4444-000000000002'),
    ('00000000-0000-0000-3333-FFF000000003', '00000000-0000-0000-4444-000000000003');

commit;
