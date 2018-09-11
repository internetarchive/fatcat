
BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE READ ONLY DEFERRABLE;

COPY (SELECT id, rev_id, redirect_id FROM creator_ident      WHERE is_live=true) TO '/tmp/fatcat_ident_creators.tsv';
COPY (SELECT id, rev_id, redirect_id FROM container_ident    WHERE is_live=true) TO '/tmp/fatcat_ident_containers.tsv';
COPY (SELECT id, rev_id, redirect_id FROM file_ident         WHERE is_live=true) TO '/tmp/fatcat_ident_files.tsv';
COPY (SELECT id, rev_id, redirect_id FROM release_ident      WHERE is_live=true) TO '/tmp/fatcat_ident_releases.tsv';
COPY (SELECT id, rev_id, redirect_id FROM work_ident         WHERE is_live=true) TO '/tmp/fatcat_ident_works.tsv';

COPY (SELECT row_to_json(abstracts) FROM abstracts) TO '/tmp/fatcat_abstracts.json';

ROLLBACK;
