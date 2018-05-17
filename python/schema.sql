

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET client_min_messages = warning;
SET row_security = off;


COMMENT ON DATABASE postgres IS 'default administrative connection database';



CREATE EXTENSION IF NOT EXISTS plpgsql WITH SCHEMA pg_catalog;



COMMENT ON EXTENSION plpgsql IS 'PL/pgSQL procedural language';


SET default_tablespace = '';

SET default_with_oids = false;


CREATE TABLE public.changelog (
    id integer NOT NULL,
    editgroup_id integer,
    "timestamp" integer
);


ALTER TABLE public.changelog OWNER TO postgres;


CREATE SEQUENCE public.changelog_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.changelog_id_seq OWNER TO postgres;


ALTER SEQUENCE public.changelog_id_seq OWNED BY public.changelog.id;



CREATE TABLE public.container_edit (
    id integer NOT NULL,
    ident_id integer,
    rev_id integer,
    redirect_id integer,
    editgroup_id integer,
    extra_json_id character varying
);


ALTER TABLE public.container_edit OWNER TO postgres;


CREATE SEQUENCE public.container_edit_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.container_edit_id_seq OWNER TO postgres;


ALTER SEQUENCE public.container_edit_id_seq OWNED BY public.container_edit.id;



CREATE TABLE public.container_ident (
    id integer NOT NULL,
    is_live boolean NOT NULL,
    rev_id integer,
    redirect_id integer
);


ALTER TABLE public.container_ident OWNER TO postgres;


CREATE SEQUENCE public.container_ident_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.container_ident_id_seq OWNER TO postgres;


ALTER SEQUENCE public.container_ident_id_seq OWNED BY public.container_ident.id;



CREATE TABLE public.container_rev (
    id integer NOT NULL,
    extra_json_id character varying,
    name character varying,
    parent_id integer,
    publisher character varying,
    sortname character varying,
    issn character varying
);


ALTER TABLE public.container_rev OWNER TO postgres;


CREATE SEQUENCE public.container_rev_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.container_rev_id_seq OWNER TO postgres;


ALTER SEQUENCE public.container_rev_id_seq OWNED BY public.container_rev.id;



CREATE TABLE public.creator_edit (
    id integer NOT NULL,
    ident_id integer,
    rev_id integer,
    redirect_id integer,
    editgroup_id integer,
    extra_json_id character varying
);


ALTER TABLE public.creator_edit OWNER TO postgres;


CREATE SEQUENCE public.creator_edit_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.creator_edit_id_seq OWNER TO postgres;


ALTER SEQUENCE public.creator_edit_id_seq OWNED BY public.creator_edit.id;



CREATE TABLE public.creator_ident (
    id integer NOT NULL,
    is_live boolean NOT NULL,
    rev_id integer,
    redirect_id integer
);


ALTER TABLE public.creator_ident OWNER TO postgres;


CREATE SEQUENCE public.creator_ident_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.creator_ident_id_seq OWNER TO postgres;


ALTER SEQUENCE public.creator_ident_id_seq OWNED BY public.creator_ident.id;



CREATE TABLE public.creator_rev (
    id integer NOT NULL,
    extra_json_id character varying,
    name character varying,
    sortname character varying,
    orcid character varying
);


ALTER TABLE public.creator_rev OWNER TO postgres;


CREATE SEQUENCE public.creator_rev_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.creator_rev_id_seq OWNER TO postgres;


ALTER SEQUENCE public.creator_rev_id_seq OWNED BY public.creator_rev.id;



CREATE TABLE public.editgroup (
    id integer NOT NULL,
    editor_id integer NOT NULL,
    description character varying,
    extra_json_id character varying
);


ALTER TABLE public.editgroup OWNER TO postgres;


CREATE SEQUENCE public.editgroup_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.editgroup_id_seq OWNER TO postgres;


ALTER SEQUENCE public.editgroup_id_seq OWNED BY public.editgroup.id;



CREATE TABLE public.editor (
    id integer NOT NULL,
    username character varying NOT NULL,
    is_admin boolean NOT NULL,
    active_editgroup_id integer
);


ALTER TABLE public.editor OWNER TO postgres;


CREATE SEQUENCE public.editor_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.editor_id_seq OWNER TO postgres;


ALTER SEQUENCE public.editor_id_seq OWNED BY public.editor.id;



CREATE TABLE public.extra_json (
    sha1 character varying NOT NULL,
    json character varying NOT NULL
);


ALTER TABLE public.extra_json OWNER TO postgres;


CREATE TABLE public.file_edit (
    id integer NOT NULL,
    ident_id integer,
    rev_id integer,
    redirect_id integer,
    editgroup_id integer,
    extra_json_id character varying
);


ALTER TABLE public.file_edit OWNER TO postgres;


CREATE SEQUENCE public.file_edit_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.file_edit_id_seq OWNER TO postgres;


ALTER SEQUENCE public.file_edit_id_seq OWNED BY public.file_edit.id;



CREATE TABLE public.file_ident (
    id integer NOT NULL,
    is_live boolean NOT NULL,
    rev_id integer,
    redirect_id integer
);


ALTER TABLE public.file_ident OWNER TO postgres;


CREATE SEQUENCE public.file_ident_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.file_ident_id_seq OWNER TO postgres;


ALTER SEQUENCE public.file_ident_id_seq OWNED BY public.file_ident.id;



CREATE TABLE public.file_release (
    id integer NOT NULL,
    file_rev integer NOT NULL,
    release_ident_id integer NOT NULL
);


ALTER TABLE public.file_release OWNER TO postgres;


CREATE SEQUENCE public.file_release_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.file_release_id_seq OWNER TO postgres;


ALTER SEQUENCE public.file_release_id_seq OWNED BY public.file_release.id;



CREATE TABLE public.file_rev (
    id integer NOT NULL,
    extra_json_id character varying,
    size integer,
    sha1 character varying,
    url integer
);


ALTER TABLE public.file_rev OWNER TO postgres;


CREATE SEQUENCE public.file_rev_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.file_rev_id_seq OWNER TO postgres;


ALTER SEQUENCE public.file_rev_id_seq OWNED BY public.file_rev.id;



CREATE TABLE public.release_contrib (
    release_rev integer NOT NULL,
    creator_ident_id integer NOT NULL,
    stub character varying,
    type character varying
);


ALTER TABLE public.release_contrib OWNER TO postgres;


CREATE TABLE public.release_edit (
    id integer NOT NULL,
    ident_id integer,
    rev_id integer,
    redirect_id integer,
    editgroup_id integer,
    extra_json_id character varying
);


ALTER TABLE public.release_edit OWNER TO postgres;


CREATE SEQUENCE public.release_edit_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.release_edit_id_seq OWNER TO postgres;


ALTER SEQUENCE public.release_edit_id_seq OWNED BY public.release_edit.id;



CREATE TABLE public.release_ident (
    id integer NOT NULL,
    is_live boolean NOT NULL,
    rev_id integer,
    redirect_id integer
);


ALTER TABLE public.release_ident OWNER TO postgres;


CREATE SEQUENCE public.release_ident_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.release_ident_id_seq OWNER TO postgres;


ALTER SEQUENCE public.release_ident_id_seq OWNED BY public.release_ident.id;



CREATE TABLE public.release_ref (
    id integer NOT NULL,
    release_rev integer NOT NULL,
    target_release_ident_id integer,
    index integer,
    stub character varying,
    doi character varying
);


ALTER TABLE public.release_ref OWNER TO postgres;


CREATE SEQUENCE public.release_ref_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.release_ref_id_seq OWNER TO postgres;


ALTER SEQUENCE public.release_ref_id_seq OWNED BY public.release_ref.id;



CREATE TABLE public.release_rev (
    id integer NOT NULL,
    extra_json_id character varying,
    work_ident_id integer,
    container_ident_id integer,
    title character varying NOT NULL,
    license character varying,
    release_type character varying,
    date character varying,
    doi character varying,
    volume character varying,
    pages character varying,
    issue character varying
);


ALTER TABLE public.release_rev OWNER TO postgres;


CREATE SEQUENCE public.release_rev_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.release_rev_id_seq OWNER TO postgres;


ALTER SEQUENCE public.release_rev_id_seq OWNED BY public.release_rev.id;



CREATE TABLE public.work_edit (
    id integer NOT NULL,
    ident_id integer,
    rev_id integer,
    redirect_id integer,
    editgroup_id integer,
    extra_json_id character varying
);


ALTER TABLE public.work_edit OWNER TO postgres;


CREATE SEQUENCE public.work_edit_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.work_edit_id_seq OWNER TO postgres;


ALTER SEQUENCE public.work_edit_id_seq OWNED BY public.work_edit.id;



CREATE TABLE public.work_ident (
    id integer NOT NULL,
    is_live boolean NOT NULL,
    rev_id integer,
    redirect_id integer
);


ALTER TABLE public.work_ident OWNER TO postgres;


CREATE SEQUENCE public.work_ident_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.work_ident_id_seq OWNER TO postgres;


ALTER SEQUENCE public.work_ident_id_seq OWNED BY public.work_ident.id;



CREATE TABLE public.work_rev (
    id integer NOT NULL,
    extra_json_id character varying,
    title character varying,
    work_type character varying,
    primary_release_id integer
);


ALTER TABLE public.work_rev OWNER TO postgres;


CREATE SEQUENCE public.work_rev_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.work_rev_id_seq OWNER TO postgres;


ALTER SEQUENCE public.work_rev_id_seq OWNED BY public.work_rev.id;



ALTER TABLE ONLY public.changelog ALTER COLUMN id SET DEFAULT nextval('public.changelog_id_seq'::regclass);



ALTER TABLE ONLY public.container_edit ALTER COLUMN id SET DEFAULT nextval('public.container_edit_id_seq'::regclass);



ALTER TABLE ONLY public.container_ident ALTER COLUMN id SET DEFAULT nextval('public.container_ident_id_seq'::regclass);



ALTER TABLE ONLY public.container_rev ALTER COLUMN id SET DEFAULT nextval('public.container_rev_id_seq'::regclass);



ALTER TABLE ONLY public.creator_edit ALTER COLUMN id SET DEFAULT nextval('public.creator_edit_id_seq'::regclass);



ALTER TABLE ONLY public.creator_ident ALTER COLUMN id SET DEFAULT nextval('public.creator_ident_id_seq'::regclass);



ALTER TABLE ONLY public.creator_rev ALTER COLUMN id SET DEFAULT nextval('public.creator_rev_id_seq'::regclass);



ALTER TABLE ONLY public.editgroup ALTER COLUMN id SET DEFAULT nextval('public.editgroup_id_seq'::regclass);



ALTER TABLE ONLY public.editor ALTER COLUMN id SET DEFAULT nextval('public.editor_id_seq'::regclass);



ALTER TABLE ONLY public.file_edit ALTER COLUMN id SET DEFAULT nextval('public.file_edit_id_seq'::regclass);



ALTER TABLE ONLY public.file_ident ALTER COLUMN id SET DEFAULT nextval('public.file_ident_id_seq'::regclass);



ALTER TABLE ONLY public.file_release ALTER COLUMN id SET DEFAULT nextval('public.file_release_id_seq'::regclass);



ALTER TABLE ONLY public.file_rev ALTER COLUMN id SET DEFAULT nextval('public.file_rev_id_seq'::regclass);



ALTER TABLE ONLY public.release_edit ALTER COLUMN id SET DEFAULT nextval('public.release_edit_id_seq'::regclass);



ALTER TABLE ONLY public.release_ident ALTER COLUMN id SET DEFAULT nextval('public.release_ident_id_seq'::regclass);



ALTER TABLE ONLY public.release_ref ALTER COLUMN id SET DEFAULT nextval('public.release_ref_id_seq'::regclass);



ALTER TABLE ONLY public.release_rev ALTER COLUMN id SET DEFAULT nextval('public.release_rev_id_seq'::regclass);



ALTER TABLE ONLY public.work_edit ALTER COLUMN id SET DEFAULT nextval('public.work_edit_id_seq'::regclass);



ALTER TABLE ONLY public.work_ident ALTER COLUMN id SET DEFAULT nextval('public.work_ident_id_seq'::regclass);



ALTER TABLE ONLY public.work_rev ALTER COLUMN id SET DEFAULT nextval('public.work_rev_id_seq'::regclass);



ALTER TABLE ONLY public.changelog
    ADD CONSTRAINT changelog_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.container_edit
    ADD CONSTRAINT container_edit_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.container_ident
    ADD CONSTRAINT container_ident_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.container_rev
    ADD CONSTRAINT container_rev_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.creator_edit
    ADD CONSTRAINT creator_edit_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.creator_ident
    ADD CONSTRAINT creator_ident_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.creator_rev
    ADD CONSTRAINT creator_rev_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.editgroup
    ADD CONSTRAINT editgroup_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.editor
    ADD CONSTRAINT editor_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.editor
    ADD CONSTRAINT editor_username_key UNIQUE (username);



ALTER TABLE ONLY public.extra_json
    ADD CONSTRAINT extra_json_pkey PRIMARY KEY (sha1);



ALTER TABLE ONLY public.file_edit
    ADD CONSTRAINT file_edit_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.file_ident
    ADD CONSTRAINT file_ident_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.file_release
    ADD CONSTRAINT file_release_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.file_rev
    ADD CONSTRAINT file_rev_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.release_contrib
    ADD CONSTRAINT release_contrib_pkey PRIMARY KEY (release_rev, creator_ident_id);



ALTER TABLE ONLY public.release_edit
    ADD CONSTRAINT release_edit_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.release_ident
    ADD CONSTRAINT release_ident_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.release_ref
    ADD CONSTRAINT release_ref_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.release_rev
    ADD CONSTRAINT release_rev_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.work_edit
    ADD CONSTRAINT work_edit_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.work_ident
    ADD CONSTRAINT work_ident_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.work_rev
    ADD CONSTRAINT work_rev_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.changelog
    ADD CONSTRAINT changelog_editgroup_id_fkey FOREIGN KEY (editgroup_id) REFERENCES public.editgroup(id);



ALTER TABLE ONLY public.container_edit
    ADD CONSTRAINT container_edit_editgroup_id_fkey FOREIGN KEY (editgroup_id) REFERENCES public.editgroup(id);



ALTER TABLE ONLY public.container_edit
    ADD CONSTRAINT container_edit_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.container_edit
    ADD CONSTRAINT container_edit_ident_id_fkey FOREIGN KEY (ident_id) REFERENCES public.container_ident(id);



ALTER TABLE ONLY public.container_edit
    ADD CONSTRAINT container_edit_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.container_ident(id);



ALTER TABLE ONLY public.container_edit
    ADD CONSTRAINT container_edit_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.container_rev(id);



ALTER TABLE ONLY public.container_ident
    ADD CONSTRAINT container_ident_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.container_ident(id);



ALTER TABLE ONLY public.container_ident
    ADD CONSTRAINT container_ident_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.container_rev(id);



ALTER TABLE ONLY public.container_rev
    ADD CONSTRAINT container_rev_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.container_rev
    ADD CONSTRAINT container_rev_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.container_ident(id);



ALTER TABLE ONLY public.creator_edit
    ADD CONSTRAINT creator_edit_editgroup_id_fkey FOREIGN KEY (editgroup_id) REFERENCES public.editgroup(id);



ALTER TABLE ONLY public.creator_edit
    ADD CONSTRAINT creator_edit_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.creator_edit
    ADD CONSTRAINT creator_edit_ident_id_fkey FOREIGN KEY (ident_id) REFERENCES public.creator_ident(id);



ALTER TABLE ONLY public.creator_edit
    ADD CONSTRAINT creator_edit_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.creator_ident(id);



ALTER TABLE ONLY public.creator_edit
    ADD CONSTRAINT creator_edit_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.creator_rev(id);



ALTER TABLE ONLY public.creator_ident
    ADD CONSTRAINT creator_ident_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.creator_ident(id);



ALTER TABLE ONLY public.creator_ident
    ADD CONSTRAINT creator_ident_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.creator_rev(id);



ALTER TABLE ONLY public.creator_rev
    ADD CONSTRAINT creator_rev_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.editgroup
    ADD CONSTRAINT editgroup_editor_id_fkey FOREIGN KEY (editor_id) REFERENCES public.editor(id);



ALTER TABLE ONLY public.editgroup
    ADD CONSTRAINT editgroup_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.editor
    ADD CONSTRAINT editor_active_editgroup_id_fkey FOREIGN KEY (active_editgroup_id) REFERENCES public.editgroup(id);



ALTER TABLE ONLY public.file_edit
    ADD CONSTRAINT file_edit_editgroup_id_fkey FOREIGN KEY (editgroup_id) REFERENCES public.editgroup(id);



ALTER TABLE ONLY public.file_edit
    ADD CONSTRAINT file_edit_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.file_edit
    ADD CONSTRAINT file_edit_ident_id_fkey FOREIGN KEY (ident_id) REFERENCES public.file_ident(id);



ALTER TABLE ONLY public.file_edit
    ADD CONSTRAINT file_edit_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.file_ident(id);



ALTER TABLE ONLY public.file_edit
    ADD CONSTRAINT file_edit_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.file_rev(id);



ALTER TABLE ONLY public.file_ident
    ADD CONSTRAINT file_ident_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.file_ident(id);



ALTER TABLE ONLY public.file_ident
    ADD CONSTRAINT file_ident_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.file_rev(id);



ALTER TABLE ONLY public.file_release
    ADD CONSTRAINT file_release_file_rev_fkey FOREIGN KEY (file_rev) REFERENCES public.file_rev(id);



ALTER TABLE ONLY public.file_release
    ADD CONSTRAINT file_release_release_ident_id_fkey FOREIGN KEY (release_ident_id) REFERENCES public.release_ident(id);



ALTER TABLE ONLY public.file_rev
    ADD CONSTRAINT file_rev_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.release_contrib
    ADD CONSTRAINT release_contrib_creator_ident_id_fkey FOREIGN KEY (creator_ident_id) REFERENCES public.creator_ident(id);



ALTER TABLE ONLY public.release_contrib
    ADD CONSTRAINT release_contrib_release_rev_fkey FOREIGN KEY (release_rev) REFERENCES public.release_rev(id);



ALTER TABLE ONLY public.release_edit
    ADD CONSTRAINT release_edit_editgroup_id_fkey FOREIGN KEY (editgroup_id) REFERENCES public.editgroup(id);



ALTER TABLE ONLY public.release_edit
    ADD CONSTRAINT release_edit_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.release_edit
    ADD CONSTRAINT release_edit_ident_id_fkey FOREIGN KEY (ident_id) REFERENCES public.release_ident(id);



ALTER TABLE ONLY public.release_edit
    ADD CONSTRAINT release_edit_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.release_ident(id);



ALTER TABLE ONLY public.release_edit
    ADD CONSTRAINT release_edit_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.release_rev(id);



ALTER TABLE ONLY public.release_ident
    ADD CONSTRAINT release_ident_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.release_ident(id);



ALTER TABLE ONLY public.release_ident
    ADD CONSTRAINT release_ident_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.release_rev(id);



ALTER TABLE ONLY public.release_ref
    ADD CONSTRAINT release_ref_release_rev_fkey FOREIGN KEY (release_rev) REFERENCES public.release_rev(id);



ALTER TABLE ONLY public.release_ref
    ADD CONSTRAINT release_ref_target_release_ident_id_fkey FOREIGN KEY (target_release_ident_id) REFERENCES public.release_ident(id);



ALTER TABLE ONLY public.release_rev
    ADD CONSTRAINT release_rev_container_ident_id_fkey FOREIGN KEY (container_ident_id) REFERENCES public.container_ident(id);



ALTER TABLE ONLY public.release_rev
    ADD CONSTRAINT release_rev_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.release_rev
    ADD CONSTRAINT release_rev_work_ident_id_fkey FOREIGN KEY (work_ident_id) REFERENCES public.work_ident(id);



ALTER TABLE ONLY public.work_edit
    ADD CONSTRAINT work_edit_editgroup_id_fkey FOREIGN KEY (editgroup_id) REFERENCES public.editgroup(id);



ALTER TABLE ONLY public.work_edit
    ADD CONSTRAINT work_edit_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.work_edit
    ADD CONSTRAINT work_edit_ident_id_fkey FOREIGN KEY (ident_id) REFERENCES public.work_ident(id);



ALTER TABLE ONLY public.work_edit
    ADD CONSTRAINT work_edit_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.work_ident(id);



ALTER TABLE ONLY public.work_edit
    ADD CONSTRAINT work_edit_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.work_rev(id);



ALTER TABLE ONLY public.work_ident
    ADD CONSTRAINT work_ident_redirect_id_fkey FOREIGN KEY (redirect_id) REFERENCES public.work_ident(id);



ALTER TABLE ONLY public.work_ident
    ADD CONSTRAINT work_ident_rev_id_fkey FOREIGN KEY (rev_id) REFERENCES public.work_rev(id);



ALTER TABLE ONLY public.work_rev
    ADD CONSTRAINT work_rev_extra_json_id_fkey FOREIGN KEY (extra_json_id) REFERENCES public.extra_json(sha1);



ALTER TABLE ONLY public.work_rev
    ADD CONSTRAINT work_rev_primary_release_id_fkey FOREIGN KEY (primary_release_id) REFERENCES public.release_ident(id);



