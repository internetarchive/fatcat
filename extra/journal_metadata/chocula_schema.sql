

CREATE TABLE IF NOT EXISTS journal
    (issnl TEXT NOT NULL PRIMARY KEY,
     issne TEXT,
     issnp TEXT,
     fatcat_ident TEXT,
     name TEXT,
     publisher TEXT,
     country TEXT,
     lang TEXT,
     publisher_type TEXT,
     is_active BOOLEAN,
     is_oa BOOLEAN default false,
     is_longtail BOOLEAN default false,
     vor_pdf BOOLEAN,
     vor_html BOOLEAN,
     vor_jats BOOLEAN,
     has_dois BOOLEAN,
     any_live_homepage BOOLEAN,
     any_preservation BOOLEAN,
     any_ia BOOLEAN
    );

CREATE TABLE IF NOT EXISTS directory
    (issnl TEXT NOT NULL,
     slug TEXT NOT NULL,
     identifier TEXT,
     name TEXT,
     work_count INTEGER,
     extra TEXT,
     PRIMARY KEY(issnl, slug)
    );

CREATE TABLE IF NOT EXISTS fatcat_container
    (issnl TEXT NOT NULL PRIMARY KEY,
     ident TEXT NOT NULL,
     revision TEXT NOT NULL,
     issne TEXT,
     issnp TEXT,
     wikidata_qid TEXT,
     name TEXT,
     container_type TEXT,
     publisher TEXT,
     country TEXT,
     lang TEXT,
     release_count INTEGER,
     ia_count INTEGER,
     ia_frac FLOAT,
     kbart_count INTEGER,
     kbart_frac FLOAT,
     preserved_count INTEGER,
     preserved_frac FLOAT
    );

CREATE TABLE IF NOT EXISTS homepage
    (id INTEGER PRIMARY KEY,
     issnl TEXT NOT NULL,
     surt TEXT NOT NULL,
     url TEXT NOT NULL,
     host TEXT NOT NULL,
     domain TEXT NOT NULL,
     suffix TEXT NOT NULL,
     status_code INTEGER,
     terminal_url TEXT,
     terminal_status_code INTEGER,
     platform_software TEXT,
     scope TEXT,
     has_issn BOOLEAN,
     blocked BOOLEAN,
     UNIQUE(issnl, surt)
    );
CREATE INDEX IF NOT EXISTS homepage_url_idx ON homepage(url);
    
CREATE TABLE IF NOT EXISTS fulltext_pattern
    (id INTEGER PRIMARY KEY,
     issnl TEXT NOT NULL,
     surt_glob TEXT,
     mimetype TEXT,
     state_date TEXT,
     end_date TEXT
    );

CREATE TABLE IF NOT EXISTS title 
    (id INTEGER PRIMARY KEY,
     issnl TEXT NOT NULL,
     title TEXT NOT NULL
    );
