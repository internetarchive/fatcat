-- written for Postgres 9.6 with OSSP extension for UUIDs
-- ... but actually runs on Postgres 11 in qa/production/tests
-- ... and now Postgres 13

-- Previously VARCHAR and fixed-size CHAR was used in this schema for specific
-- columns (especially fixed-size external identifiers, and hashes). This was
-- found to cause lookup problems, so switched to TEXT with CHECK constraints.

-- Default timezone (of clients) is expected to be UTC.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


-- uuid_generate_v1mc: timestamp ordered, random MAC address
-- uuid_generate_v4:   totally random

-- NB: could use LIKE clause, or "composite types"

CREATE TABLE editor (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username            TEXT NOT NULL CHECK (username ~* '^[A-Za-z][A-Za-z0-9._-]{2,24}$'), -- UNIQ below
    is_superuser        BOOLEAN NOT NULL DEFAULT false,
    is_admin            BOOLEAN NOT NULL DEFAULT false,
    is_bot              BOOLEAN NOT NULL DEFAULT false,
    is_active           BOOLEAN NOT NULL DEFAULT true,
    registered          TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    auth_epoch          TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    wrangler_id         UUID REFERENCES editor(id)
);

-- case-insensitive UNIQ index on username
CREATE UNIQUE INDEX editor_username_uniq_idx on editor(lower(username));
CREATE INDEX editor_username_idx ON editor(username);

CREATE TABLE auth_oidc (
    id                  BIGSERIAL PRIMARY KEY,
    created             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    editor_id           UUID REFERENCES editor(id) NOT NULL,
    provider            TEXT NOT NULL,
    oidc_iss            TEXT NOT NULL,
    oidc_sub            TEXT NOT NULL,
    UNIQUE (editor_id, provider),
    UNIQUE (oidc_iss, oidc_sub)
);

CREATE TABLE editgroup (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    editor_id           UUID REFERENCES editor(id) NOT NULL,
    created             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    submitted           TIMESTAMP WITH TIME ZONE,
    is_accepted         BOOLEAN DEFAULT false NOT NULL,
    description         TEXT CHECK (octet_length(description) >= 1),
    extra_json          JSONB
);

-- for fast "recent, reviewable" and "work in progress" queries
CREATE INDEX editgroup_submitted_idx ON editgroup(is_accepted, submitted);
CREATE INDEX editgroup_editor_idx ON editgroup(is_accepted, editor_id);

CREATE TABLE editgroup_annotation (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    editor_id           UUID REFERENCES editor(id) NOT NULL,
    created             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    comment_markdown    TEXT CHECK (octet_length(comment_markdown) >= 1),
    extra_json          JSONB
);

CREATE INDEX editgroup_annotation_editgroup_created_idx ON editgroup_annotation(editgroup_id, created);
CREATE INDEX editgroup_annotation_editor_created_idx ON editgroup_annotation(editor_id, created);

CREATE TABLE changelog (
    id                  BIGSERIAL PRIMARY KEY,
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    timestamp           TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

-- for "is this editgroup merged" queries
CREATE INDEX changelog_editgroup_idx ON changelog(editgroup_id);

CREATE TABLE abstracts (
    -- fixed size hash (in hex). TODO: switch to bytes
    sha1                TEXT PRIMARY KEY CHECK (octet_length(sha1) = 40),
    content             TEXT NOT NULL CHECK (octet_length(content) >= 1)
);

CREATE TABLE refs_blob (
    -- fixed size hash (in hex). TODO: switch to bytes
    sha1                TEXT PRIMARY KEY CHECK (octet_length(sha1) = 40),
    refs_json           JSONB NOT NULL
);

-------------------- Creators -----------------------------------------------
CREATE TABLE creator_rev (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    extra_json          JSONB,

    display_name        TEXT NOT NULL CHECK (octet_length(display_name) >= 1),
    given_name          TEXT CHECK (octet_length(given_name) >= 1),
    surname             TEXT CHECK (octet_length(surname) >= 1),
    -- fixed size identifier
    orcid               TEXT CHECK(octet_length(orcid) = 19),
    -- limited size for data quality
    wikidata_qid        TEXT CHECK(octet_length(wikidata_qid) <= 12)

    -- TODO: viaf                TEXT,
    -- TODO: aliases/alternatives
    -- TODO: sortable name?
);

-- Could denormalize a "is_live" flag into revision tables, to make indices
-- more efficient
CREATE INDEX creator_rev_orcid_idx ON creator_rev(orcid);
CREATE INDEX creator_rev_wikidata_idx ON creator_rev(wikidata_qid);

CREATE TABLE creator_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES creator_rev(id),
    redirect_id         UUID REFERENCES creator_ident(id)
);

CREATE INDEX creator_ident_rev_idx ON creator_ident(rev_id);
CREATE INDEX creator_ident_redirect_idx ON creator_ident(redirect_id);

CREATE TABLE creator_edit (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES creator_ident(id) NOT NULL,
    rev_id              UUID REFERENCES creator_rev(id),
    redirect_id         UUID REFERENCES creator_ident(id),
    prev_rev            UUID REFERENCES creator_rev(id),
    extra_json          JSONB,
    UNIQUE (editgroup_id, ident_id)
);

-------------------- Containers --------------------------------------------
CREATE TABLE container_rev (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    extra_json          JSONB,

    name                TEXT NOT NULL CHECK (octet_length(name) >= 1),
    container_type      TEXT,
    publisher           TEXT CHECK (octet_length(publisher) >= 1),
    -- fixed size identifier
    issnl               TEXT CHECK(octet_length(issnl) = 9),
    -- limited size for data quality
    wikidata_qid        TEXT CHECK(octet_length(wikidata_qid) <= 12)
);

CREATE INDEX container_rev_issnl_idx ON container_rev(issnl);
CREATE INDEX container_rev_wikidata_idx ON container_rev(wikidata_qid);

CREATE TABLE container_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES container_rev(id),
    redirect_id         UUID REFERENCES container_ident(id)
);

CREATE INDEX container_ident_rev_idx ON container_ident(rev_id);
CREATE INDEX container_ident_redirect_idx ON container_ident(redirect_id);

CREATE TABLE container_edit (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES container_ident(id) NOT NULL,
    rev_id              UUID REFERENCES container_rev(id),
    redirect_id         UUID REFERENCES container_ident(id),
    prev_rev            UUID REFERENCES container_rev(id),
    extra_json          JSONB,
    UNIQUE (editgroup_id, ident_id)
);

-------------------- Files -------------------------------------------------
CREATE TABLE file_rev (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    extra_json          JSONB,

    size_bytes          BIGINT,
    -- fixed size hashes (in hex). TODO: switch to binary type type
    sha1                TEXT CHECK (octet_length(sha1) = 40),
    sha256              TEXT CHECK (octet_length(sha256) = 64),
    md5                 TEXT CHECK (octet_length(md5) = 32),
    mimetype            TEXT CHECK (octet_length(mimetype) >= 1)
);

CREATE INDEX file_rev_sha1_idx ON file_rev(sha1);
CREATE INDEX file_rev_md5_idx ON file_rev(md5);
CREATE INDEX file_rev_sha256_idx ON file_rev(sha256);

CREATE TABLE file_rev_url (
    id                  BIGSERIAL PRIMARY KEY,
    file_rev            UUID REFERENCES file_rev(id) NOT NULL,
    rel                 TEXT NOT NULL CHECK (octet_length(rel) >= 1), -- TODO: enum? web, webarchive, repo, etc
    url                 TEXT NOT NULL CHECK (octet_length(url) >= 1)
);

CREATE INDEX file_rev_url_rev_idx ON file_rev_url(file_rev);

CREATE TABLE file_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES file_rev(id),
    redirect_id         UUID REFERENCES file_ident(id)
);

CREATE INDEX file_ident_rev_idx ON file_ident(rev_id);
CREATE INDEX file_ident_redirect_idx ON file_ident(redirect_id);

CREATE TABLE file_edit (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES file_ident(id) NOT NULL,
    rev_id              UUID REFERENCES file_rev(id),
    redirect_id         UUID REFERENCES file_ident(id),
    prev_rev            UUID REFERENCES file_rev(id),
    extra_json          JSONB,
    UNIQUE (editgroup_id, ident_id)
);

-------------------- Fileset -----------------------------------------------
CREATE TABLE fileset_rev (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    extra_json          JSONB
);

CREATE TABLE fileset_rev_url (
    id                  BIGSERIAL PRIMARY KEY,
    fileset_rev         UUID REFERENCES fileset_rev(id) NOT NULL,
    rel                 TEXT NOT NULL CHECK (octet_length(rel) >= 1), -- TODO: enum? web, webarchive, repo, etc
    url                 TEXT NOT NULL CHECK (octet_length(url) >= 1)
);

CREATE INDEX fileset_rev_url_rev_idx ON fileset_rev_url(fileset_rev);

CREATE TABLE fileset_rev_file (
    id                  BIGSERIAL PRIMARY KEY,
    fileset_rev         UUID REFERENCES fileset_rev(id) NOT NULL,
    path_name           TEXT NOT NULL CHECK (octet_length(path_name) >= 1),
    size_bytes          BIGINT NOT NULL,
    md5                 TEXT CHECK(octet_length(md5) = 32),
    sha1                TEXT CHECK(octet_length(sha1) = 40),
    sha256              TEXT CHECK(octet_length(sha256) = 64),
    extra_json          JSONB
);

CREATE INDEX fileset_rev_file_rev_idx ON fileset_rev_file(fileset_rev);

CREATE TABLE fileset_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES fileset_rev(id),
    redirect_id         UUID REFERENCES fileset_ident(id)
);

CREATE INDEX fileset_ident_rev_idx ON fileset_ident(rev_id);

CREATE TABLE fileset_edit (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES fileset_ident(id) NOT NULL,
    rev_id              UUID REFERENCES fileset_rev(id),
    redirect_id         UUID REFERENCES fileset_ident(id),
    prev_rev            UUID REFERENCES fileset_rev(id),
    extra_json          JSONB,
    UNIQUE (editgroup_id, ident_id)
);

-------------------- Webcapture -----------------------------------------------
CREATE TABLE webcapture_rev (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    extra_json          JSONB,

    original_url        TEXT NOT NULL CHECK (octet_length(original_url) >= 1),
    timestamp           TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE webcapture_rev_url (
    id                  BIGSERIAL PRIMARY KEY,
    webcapture_rev      UUID REFERENCES webcapture_rev(id) NOT NULL,
    rel                 TEXT NOT NULL CHECK (octet_length(rel) >= 1), -- TODO: enum? web, webarchive, repo, etc
    url                 TEXT NOT NULL CHECK (octet_length(url) >= 1)
);

CREATE INDEX webcapture_rev_url_rev_idx ON webcapture_rev_url(webcapture_rev);

CREATE TABLE webcapture_rev_cdx (
    id                  BIGSERIAL PRIMARY KEY,
    webcapture_rev      UUID REFERENCES webcapture_rev(id) NOT NULL,
    surt                TEXT NOT NULL CHECK (octet_length(surt) >= 1),
    timestamp           TIMESTAMP WITH TIME ZONE NOT NULL,
    url                 TEXT NOT NULL CHECK (octet_length(url) >= 1),
    mimetype            TEXT CHECK (octet_length(mimetype) >= 1),
    status_code         BIGINT,
    sha1                TEXT CHECK(octet_length(sha1) = 40) NOT NULL,
    sha256              TEXT CHECK(octet_length(sha256) = 64)
    -- could extend with: language (detection), simhash, redirect
);

CREATE INDEX webcapture_rev_cdx_rev_idx ON webcapture_rev_cdx(webcapture_rev);

CREATE TABLE webcapture_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES webcapture_rev(id),
    redirect_id         UUID REFERENCES webcapture_ident(id)
);

CREATE INDEX webcapture_ident_rev_idx ON webcapture_ident(rev_id);

CREATE TABLE webcapture_edit (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES webcapture_ident(id) NOT NULL,
    rev_id              UUID REFERENCES webcapture_rev(id),
    redirect_id         UUID REFERENCES webcapture_ident(id),
    prev_rev            UUID REFERENCES webcapture_rev(id),
    extra_json          JSONB,
    UNIQUE (editgroup_id, ident_id)
);

-------------------- Release -----------------------------------------------
CREATE TABLE release_rev (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    extra_json          JSONB,

    work_ident_id       UUID NOT NULL, -- FOREIGN KEY; see ALRTER below
    container_ident_id  UUID REFERENCES container_ident(id),
    refs_blob_sha1      TEXT REFERENCES refs_blob(sha1),
    title               TEXT NOT NULL CHECK (octet_length(title) >= 1),
    original_title      TEXT CHECK (octet_length(original_title) >= 1),
    release_type        TEXT, -- TODO: enum
    release_status      TEXT, -- TODO: enum
    release_date        DATE,
    release_year        BIGINT,
    doi                 TEXT CHECK (octet_length(doi) >= 7),
    -- CHECK for length limit for data quality
    pmid                TEXT CHECK (octet_length(pmid) <= 12),
    pmcid               TEXT CHECK (octet_length(pmcid) <= 12),
    wikidata_qid        TEXT CHECK (octet_length(wikidata_qid) <= 12),
    isbn13              TEXT CHECK (octet_length(isbn13) = 17),
    core_id             TEXT CHECK (octet_length(core_id) <= 12),
    arxiv_id            TEXT CHECK (octet_length(arxiv_id) <= 12),
    jstor_id            TEXT CHECK (octet_length(jstor_id) <= 12),
    volume              TEXT CHECK (octet_length(volume) >= 1),
    issue               TEXT CHECK (octet_length(issue) >= 1),
    pages               TEXT CHECK (octet_length(pages) >= 1),
    publisher           TEXT CHECK (octet_length(publisher) >= 1), -- for books, NOT if container exists
    language            TEXT CHECK (octet_length(language) >= 1), -- primary language of the work's fulltext; RFC1766/ISO639-1
    license_slug        TEXT CHECK (octet_length(license_slug) >= 1)
    -- TODO: oclc_ocn (TEXT or BIGINT)
    -- TODO: identifier table?
);

CREATE INDEX release_rev_doi_idx ON release_rev(doi);
CREATE INDEX release_rev_pmid_idx ON release_rev(pmid);
CREATE INDEX release_rev_pmcid_idx ON release_rev(pmcid);
CREATE INDEX release_rev_wikidata_idx ON release_rev(wikidata_qid);
CREATE INDEX release_rev_isbn13_idx ON release_rev(isbn13);
CREATE INDEX release_rev_core_idx ON release_rev(core_id);
CREATE INDEX release_rev_arxiv_idx ON release_rev(arxiv_id);
CREATE INDEX release_rev_jstor_idx ON release_rev(jstor_id);
CREATE INDEX release_rev_work_idx ON release_rev(work_ident_id);

CREATE TABLE release_rev_abstract (
    id              BIGSERIAL PRIMARY KEY,
    release_rev     UUID REFERENCES release_rev(id) NOT NULL,
    abstract_sha1   TEXT REFERENCES abstracts(sha1) NOT NULL,
    mimetype        TEXT CHECK (octet_length(mimetype) >= 1),
    lang            TEXT CHECK (octet_length(lang) >= 1)
);

CREATE INDEX release_rev_abstract_rev_idx ON release_rev_abstract(release_rev);
CREATE INDEX release_rev_abstract_sha1_idx ON release_rev_abstract(abstract_sha1);

CREATE TABLE release_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES release_rev(id),
    redirect_id         UUID REFERENCES release_ident(id)
);

CREATE INDEX release_ident_rev_idx ON release_ident(rev_id);
CREATE INDEX release_ident_redirect_idx ON release_ident(redirect_id);

CREATE TABLE release_edit (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES release_ident(id) NOT NULL,
    rev_id              UUID REFERENCES release_rev(id),
    redirect_id         UUID REFERENCES release_ident(id),
    prev_rev            UUID REFERENCES release_rev(id),
    extra_json          JSONB,
    UNIQUE (editgroup_id, ident_id)
);

-------------------- Works --------------------------------------------------
CREATE TABLE work_rev (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    extra_json          JSONB
);

CREATE TABLE work_ident (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    is_live             BOOL NOT NULL DEFAULT false,
    rev_id              UUID REFERENCES work_rev(id),
    redirect_id         UUID REFERENCES work_ident(id)
);

CREATE INDEX work_ident_rev_idx ON work_ident(rev_id);
CREATE INDEX work_ident_redirect_idx ON work_ident(redirect_id);

CREATE TABLE work_edit (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    editgroup_id        UUID REFERENCES editgroup(id) NOT NULL,
    updated             TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    ident_id            UUID REFERENCES work_ident(id) NOT NULL,
    rev_id              UUID REFERENCES work_rev(id),
    redirect_id         UUID REFERENCES work_ident(id),
    prev_rev            UUID REFERENCES work_rev(id),
    extra_json          JSONB,
    UNIQUE (editgroup_id, ident_id)
);

ALTER TABLE release_rev
    ADD CONSTRAINT release_containeridentid_fkey FOREIGN KEY (work_ident_id)
            REFERENCES work_ident(id);

-------------------- Inter-Entity Relations

CREATE TABLE release_contrib (
    id                  BIGSERIAL PRIMARY KEY,
    release_rev         UUID REFERENCES release_rev(id) NOT NULL,
    creator_ident_id    UUID REFERENCES creator_ident(id),
    raw_name            TEXT CHECK (octet_length(raw_name) >= 1),
    role                TEXT, -- TODO: enum?
    raw_affiliation     TEXT CHECK (octet_length(raw_affiliation) >= 1),
    index_val           INTEGER,
    extra_json          JSONB
);

CREATE INDEX release_contrib_rev_idx ON release_contrib(release_rev);
CREATE INDEX release_contrib_creator_idx ON release_contrib(creator_ident_id);

CREATE TABLE release_ref (
    release_rev             UUID REFERENCES release_rev(id) NOT NULL,
    index_val               INTEGER NOT NULL,
    target_release_ident_id UUID REFERENCES release_ident(id) NOT NULL,
    -- all other fields are interned in refs_blob as JSONB
    -- key                     TEXT,
    -- extra_json              JSONB, -- title, year, container_title, locator (aka, page), oci_id
    -- container_name          TEXT,
    -- year                    INTEGER,
    -- title                   TEXT,
    -- locator                 TEXT
    PRIMARY KEY(release_rev, index_val)
);

CREATE INDEX release_ref_target_release_idx ON release_ref(target_release_ident_id);

CREATE TABLE file_rev_release (
    file_rev                UUID REFERENCES file_rev(id) NOT NULL,
    target_release_ident_id UUID REFERENCES release_ident(id) NOT NULL,
    PRIMARY KEY (file_rev, target_release_ident_id)
);
CREATE INDEX file_rev_release_target_release_idx ON file_rev_release(target_release_ident_id);
CREATE TABLE fileset_rev_release (
    fileset_rev             UUID REFERENCES fileset_rev(id) NOT NULL,
    target_release_ident_id UUID REFERENCES release_ident(id) NOT NULL,
    PRIMARY KEY (fileset_rev, target_release_ident_id)
);
CREATE INDEX fileset_rev_release_target_release_idx ON fileset_rev_release(target_release_ident_id);
CREATE TABLE webcapture_rev_release (
    webcapture_rev          UUID REFERENCES webcapture_rev(id) NOT NULL,
    target_release_ident_id UUID REFERENCES release_ident(id) NOT NULL,
    PRIMARY KEY (webcapture_rev, target_release_ident_id)
);
CREATE INDEX webcapture_rev_release_target_release_idx ON webcapture_rev_release(target_release_ident_id);

---------------------------------------------------------------------------
---------------------------------------------------------------------------
---------------------------------------------------------------------------

-- Fake data at the raw SQL level, for early development and testing
-- Convention:
--  * first entity is smallest possible (mostly null)
--  * second entity is rich (all fields/relations designed) but artificial
--  * third entity (and above) are realistic (real DOI, etc)
--
-- With the exception of editor IDs/usernames, these should *not* be considered
-- part of a stable external interface, and edits might be made here instead of
-- in a migration.

INSERT INTO editor (id, username, is_superuser, is_admin, is_bot, auth_epoch) VALUES
    ('00000000-0000-0000-AAAA-000000000001', 'root', true, true, false, '1970-01-01T01:01:01Z'),          -- aaaaaaaaaaaabkvkaaaaaaaaae
    ('00000000-0000-0000-AAAA-000000000002', 'admin', true, true, false, '1970-01-01T01:01:01Z'),         -- aaaaaaaaaaaabkvkaaaaaaaaai
    ('00000000-0000-0000-AAAA-000000000003', 'demo-user', false, true, false, '1970-01-01T01:01:01Z'),    -- aaaaaaaaaaaabkvkaaaaaaaaam
    ('00000000-0000-0000-AAAA-000000000004', 'claire', false, false, false, default),                     -- aaaaaaaaaaaabkvkaaaaaaaaaq
    ('00000000-0000-0000-AAAA-000000000005', 'webface-bot', true, true, true, '1970-01-01T01:01:01Z'),    -- aaaaaaaaaaaabkvkaaaaaaaaau
    ('00000000-0000-0000-AAAA-000000000006', 'bnewbold', false, true, false, '1970-01-01T01:01:01Z');     -- aaaaaaaaaaaabkvkaaaaaaaaay

INSERT INTO editgroup (id, is_accepted, editor_id, description, submitted) VALUES
    ('00000000-0000-0000-BBBB-000000000001',  true, '00000000-0000-0000-AAAA-000000000001', 'first edit ever!', NULL),      -- aaaaaaaaaaaabo53aaaaaaaaae
    ('00000000-0000-0000-BBBB-000000000002',  true, '00000000-0000-0000-AAAA-000000000001', 'another one!', NULL),          -- aaaaaaaaaaaabo53aaaaaaaaai
    ('00000000-0000-0000-BBBB-000000000003',  true, '00000000-0000-0000-AAAA-000000000003', 'user edit', NULL),             -- aaaaaaaaaaaabo53aaaaaaaaam
    ('00000000-0000-0000-BBBB-000000000004',  true, '00000000-0000-0000-AAAA-000000000002', 'uncommited edit', NULL),       -- aaaaaaaaaaaabo53aaaaaaaaaq
    ('00000000-0000-0000-BBBB-000000000005',  true, '00000000-0000-0000-AAAA-000000000001', 'journal edit', NULL),          -- aaaaaaaaaaaabo53aaaaaaaaau
    ('00000000-0000-0000-BBBB-000000000006', false, '00000000-0000-0000-AAAA-000000000004', 'another journal edit', NULL),  -- aaaaaaaaaaaabo53aaaaaaaaay
    ('00000000-0000-0000-BBBB-000000000007', false, '00000000-0000-0000-AAAA-000000000003', 'edit for submission', now());  -- aaaaaaaaaaaabo53aaaaaaaaa4

INSERT INTO editgroup_annotation (editgroup_id, editor_id, comment_markdown) VALUES
    ('00000000-0000-0000-BBBB-000000000007', '00000000-0000-0000-AAAA-000000000003', 'I love this edit!'),
    ('00000000-0000-0000-BBBB-000000000007', '00000000-0000-0000-AAAA-000000000004', 'I have concerns about this edit...'),
    ('00000000-0000-0000-BBBB-000000000007', '00000000-0000-0000-AAAA-000000000003', 'updated with changes, please re-review.');

INSERT INTO changelog (editgroup_id) VALUES
    ('00000000-0000-0000-BBBB-000000000001'),
    ('00000000-0000-0000-BBBB-000000000002'),
    ('00000000-0000-0000-BBBB-000000000003'),
    ('00000000-0000-0000-BBBB-000000000004'),
    ('00000000-0000-0000-BBBB-000000000005');

INSERT INTO abstracts (sha1, content) VALUES
    ('1ba86bf8c2979a62d29b18b537e50b2b093be27e', 'some long abstract in plain text'),
    ('0da908ab584b5e445a06beb172e3fab8cb5169e3', '<jats>A longer, more correct abstract should in theory go here</jats>');

INSERT INTO container_rev (id, name, publisher, issnl, wikidata_qid, extra_json) VALUES
    ('00000000-0000-0000-1111-FFF000000001', 'MySpace Blog', null, null, null, null),
    ('00000000-0000-0000-1111-FFF000000002', 'Journal of Trivial Results', 'bogus publishing group', '1234-5678', 'Q1234', '{"country": "nz", "languages": ["en", "es"], "original_name": "Journal of Significant Results", "doaj": {"as_of": "2019-01-01", "works": false}, "road": {"as_of": "2019-01-01"}, "first_year": 1990, "urls": ["http://www.ajnonline.com/pt/re/ajn/home.htm;jsessionid=D8IrvYWQbd7q4A7Au1Lu3cFSJuXcVZqm2YnPJ5hBG52cgZw2htKx!958525354!-949856144!9001!-1"]}'),
    ('00000000-0000-0000-1111-FFF000000003', 'PLOS Medicine', 'Public Library of Science', '1549-1277', 'Q1686921', '{"doaj": {"as_of": "2019-01-30"}, "sherpa_romeo": {"color": "green"}}');

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

INSERT INTO file_rev (id, size_bytes, sha1, sha256, md5, mimetype) VALUES
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

INSERT INTO fileset_rev (id) VALUES
    ('00000000-0000-0000-6666-FFF000000001'),
    ('00000000-0000-0000-6666-FFF000000002'),
    ('00000000-0000-0000-6666-FFF000000003');

INSERT INTO fileset_rev_file (fileset_rev, path_name, size_bytes, md5, sha1, sha256, extra_json) VALUES
    ('00000000-0000-0000-6666-FFF000000002', 'README.md', 1024, null, null, null, null),
    ('00000000-0000-0000-6666-FFF000000003', 'README.md', 2048, null, null, null, null),
    ('00000000-0000-0000-6666-FFF000000003', 'stuff/data.tar.gz', 2340000, 'f4de91152c7ab9fdc2a128f962faebff', '3f242a192acc258bdfdb151943419437f440c313', 'ffc1005680cb620eec4c913437dfabbf311b535cfe16cbaeb2faec1f92afc362', '{"mimetype": "application/gzip"}');

INSERT INTO fileset_rev_url (fileset_rev, rel, url) VALUES
    ('00000000-0000-0000-6666-FFF000000002', 'web', 'http://personal-blog.name/dataset/'),
    ('00000000-0000-0000-6666-FFF000000003', 'web', 'http://other-personal-blog.name/dataset/'),
    ('00000000-0000-0000-6666-FFF000000003', 'archive', 'https://archive.org/download/random-dataset/');

INSERT INTO fileset_ident (id, is_live, rev_id, redirect_id) VALUES
    ('00000000-0000-0000-6666-000000000001', true, '00000000-0000-0000-6666-FFF000000001', null), -- aaaaaaaaaaaaaztgaaaaaaaaam
    ('00000000-0000-0000-6666-000000000002', true, '00000000-0000-0000-6666-FFF000000002', null), -- aaaaaaaaaaaaaztgaaaaaaaaai
    ('00000000-0000-0000-6666-000000000003', true, '00000000-0000-0000-6666-FFF000000003', null); -- aaaaaaaaaaaaaztgaaaaaaaaam

INSERT INTO fileset_edit (ident_id, rev_id, redirect_id, editgroup_id, prev_rev) VALUES
    ('00000000-0000-0000-6666-000000000001', '00000000-0000-0000-6666-FFF000000001', null, '00000000-0000-0000-BBBB-000000000003', null),
    ('00000000-0000-0000-6666-000000000002', '00000000-0000-0000-6666-FFF000000002', null, '00000000-0000-0000-BBBB-000000000004', null),
    ('00000000-0000-0000-6666-000000000003', '00000000-0000-0000-6666-FFF000000003', null, '00000000-0000-0000-BBBB-000000000005', '00000000-0000-0000-6666-FFF000000002');

INSERT INTO webcapture_rev (id, original_url, timestamp) VALUES
    ('00000000-0000-0000-7777-FFF000000001', 'http://example.org', '1996-01-02T12:34:56Z'),
    ('00000000-0000-0000-7777-FFF000000002', 'http://example.org', '1996-01-02T12:34:56Z'),
    ('00000000-0000-0000-7777-FFF000000003', 'https://asheesh.org', '2003-02-17T04:47:21Z');

INSERT INTO webcapture_rev_cdx (webcapture_rev, surt, timestamp, url, mimetype, status_code, sha1, sha256) VALUES
    ('00000000-0000-0000-7777-FFF000000002', 'org,example)/', '1996-01-02T12:34:56Z', 'http://example.org', null, 200, '5886903ba5aeaf7446fe9f77bd03adfc029cedf0', null),
    ('00000000-0000-0000-7777-FFF000000003', 'org,asheesh)/', '2003-02-17T04:47:21Z', 'http://asheesh.org:80/', 'text/html', 200, '5886903ba5aeaf7446fe9f77bd03adfc029cedf0', 'ffc1005680cb620eec4c913437dfabbf311b535cfe16cbaeb2faec1f92afc362'),
    ('00000000-0000-0000-7777-FFF000000003', 'org,asheesh)/robots.txt', '2003-02-17T04:47:19Z', 'http://asheesh.org:80/robots.txt', 'text/html', 404, 'a637f1d27d9bcb237310ed29f19c07e1c8cf0aa5', 'ffc1005680cb620eec4c913437dfabbf311b535cfe16cbaeb2faec1f92afc362');

INSERT INTO webcapture_rev_url (webcapture_rev, rel, url) VALUES
    ('00000000-0000-0000-7777-FFF000000002', 'wayback', 'http://web.archive.org/web/'),
    ('00000000-0000-0000-7777-FFF000000003', 'wayback', 'http://web.archive.org/web/'),
    ('00000000-0000-0000-7777-FFF000000003', 'warc', 'https://example.org/something.warc.gz');

INSERT INTO webcapture_ident (id, is_live, rev_id, redirect_id) VALUES
    ('00000000-0000-0000-7777-000000000001', true, '00000000-0000-0000-7777-FFF000000001', null), -- aaaaaaaaaaaaa53xaaaaaaaaae
    ('00000000-0000-0000-7777-000000000002', true, '00000000-0000-0000-7777-FFF000000002', null), -- aaaaaaaaaaaaa53xaaaaaaaaai
    ('00000000-0000-0000-7777-000000000003', true, '00000000-0000-0000-7777-FFF000000003', null); -- aaaaaaaaaaaaa53xaaaaaaaaam

INSERT INTO webcapture_edit (ident_id, rev_id, redirect_id, editgroup_id, prev_rev) VALUES
    ('00000000-0000-0000-7777-000000000001', '00000000-0000-0000-7777-FFF000000001', null, '00000000-0000-0000-BBBB-000000000003', null),
    ('00000000-0000-0000-7777-000000000002', '00000000-0000-0000-7777-FFF000000002', null, '00000000-0000-0000-BBBB-000000000004', null),
    ('00000000-0000-0000-7777-000000000003', '00000000-0000-0000-7777-FFF000000003', null, '00000000-0000-0000-BBBB-000000000005', '00000000-0000-0000-7777-FFF000000002');

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

INSERT INTO refs_blob (sha1, refs_json) VALUES
    ('22222222c2979a62d29b18b537e50b2b093be27e', '[{}, {}, {}, {}, {"extra": {"unstructured":"citation note"}}]'),
    ('33333333c2979a62d29b18b537e50b2b093be27e', '[
        {"extra": {"unstructured": "Ioannidis JP, Haidich AB, Lau J. Any casualties in the clash of randomised and observational evidence? BMJ. 2001;322:879–880"}},
        {"extra": {"unstructured":"Lawlor DA, Davey Smith G, Kundu D, Bruckdorfer KR, Ebrahim S. Those confounded vitamins: What can we learn from the differences between observational versus randomised trial evidence? Lancet. 2004;363:1724–1727."}},
        {"extra": {"unstructured":"Vandenbroucke JP. When are observational studies as credible as randomised trials? Lancet. 2004;363:1728–1731."}},
        {"extra": {"unstructured":"Michiels S, Koscielny S, Hill C. Prediction of cancer outcome with microarrays: A multiple random validation strategy. Lancet. 2005;365:488–492."}},
        {"extra": {"unstructured":"Ioannidis JPA, Ntzani EE, Trikalinos TA, Contopoulos-Ioannidis DG. Replication validity of genetic association studies. Nat Genet. 2001;29:306–309."}},
        {"extra": {"unstructured":"Colhoun HM, McKeigue PM, Davey Smith G. Problems of reporting genetic associations with complex outcomes. Lancet. 2003;361:865–872."}}
    ]');

INSERT INTO release_rev (id, work_ident_id, container_ident_id, title, release_type, release_status, release_date, release_year, doi, wikidata_qid, pmid, pmcid, isbn13, core_id, volume, issue, pages, publisher, language, refs_blob_sha1, extra_json) VALUES
    ('00000000-0000-0000-4444-FFF000000001', '00000000-0000-0000-5555-000000000001',
        null,
        'example title',
        null, null, null, null, null, null, null, null, null, null, null, null,  null, null, null, null, null),
    ('00000000-0000-0000-4444-FFF000000002', '00000000-0000-0000-5555-000000000002',
        '00000000-0000-0000-1111-000000000001',
        'A bigger example with a very long title that probably needs to wrap around to another line: Useful for testing the web interface and other bibliographic display projects',
        'article-journal',
        'published',
        '2018-01-01',
        2018,
        '10.123/abc',
        'Q55555',
        '54321',
        'PMC555',
        '978-3-16-148410-0',
        '42022773',
        '12', 'IV', '5-9', 'bogus publishing group', 'cn',
        '22222222c2979a62d29b18b537e50b2b093be27e',
        '{"container-name": "example journal", "subtitle": "son of example"}'),
    ('00000000-0000-0000-4444-FFF000000003', '00000000-0000-0000-5555-000000000003',
        '00000000-0000-0000-1111-000000000003',
        'Why Most Published Research Findings Are False',
        'article-journal',
        'published',
        '2005-08-30',
        2005,
        '10.1371/journal.pmed.0020124',
        null,
        null,
        null,
        null,
        null,
        '2', '8', 'e124', 'Public Library of Science', 'en',
        '33333333c2979a62d29b18b537e50b2b093be27e',
        null);

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

INSERT INTO release_contrib (release_rev, creator_ident_id, raw_name, role, index_val, extra_json) VALUES
    ('00000000-0000-0000-4444-FFF000000002', null, null, null, null, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', 1, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', 2, null),
    ('00000000-0000-0000-4444-FFF000000002', '00000000-0000-0000-2222-000000000002', 'robin hood', 'author', 3, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', 4, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', 5, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', 6, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', 7, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', 8, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', 9, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', 10, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', null, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', null, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', null, null),
    ('00000000-0000-0000-4444-FFF000000002', null, 'robin hood', 'author', null, null),
    ('00000000-0000-0000-4444-FFF000000002', '00000000-0000-0000-2222-000000000002', 'some contrib', 'editor', 12, '{"affiliation": "Coolllleeeeeddddggee!"}'),
    ('00000000-0000-0000-4444-FFF000000003', '00000000-0000-0000-2222-000000000003', 'John P. A. Ioannidis', 'author', 0, null);

INSERT INTO release_ref (release_rev, index_val, target_release_ident_id) VALUES
    ('00000000-0000-0000-4444-FFF000000002', 4, '00000000-0000-0000-4444-000000000001'),
    ('00000000-0000-0000-4444-FFF000000003', 0, '00000000-0000-0000-4444-000000000001'),
    ('00000000-0000-0000-4444-FFF000000003', 1, '00000000-0000-0000-4444-000000000001'),
    ('00000000-0000-0000-4444-FFF000000003', 2, '00000000-0000-0000-4444-000000000001'),
    ('00000000-0000-0000-4444-FFF000000003', 3, '00000000-0000-0000-4444-000000000001'),
    ('00000000-0000-0000-4444-FFF000000003', 4, '00000000-0000-0000-4444-000000000001'),
    ('00000000-0000-0000-4444-FFF000000003', 5, '00000000-0000-0000-4444-000000000001');

INSERT INTO file_rev_release (file_rev, target_release_ident_id) VALUES
    ('00000000-0000-0000-3333-FFF000000002', '00000000-0000-0000-4444-000000000002'),
    ('00000000-0000-0000-3333-FFF000000003', '00000000-0000-0000-4444-000000000003');

INSERT INTO fileset_rev_release (fileset_rev, target_release_ident_id) VALUES
    ('00000000-0000-0000-6666-FFF000000002', '00000000-0000-0000-4444-000000000002'),
    ('00000000-0000-0000-6666-FFF000000003', '00000000-0000-0000-4444-000000000003');

INSERT INTO webcapture_rev_release (webcapture_rev, target_release_ident_id) VALUES
    ('00000000-0000-0000-7777-FFF000000002', '00000000-0000-0000-4444-000000000002'),
    ('00000000-0000-0000-7777-FFF000000003', '00000000-0000-0000-4444-000000000003');
