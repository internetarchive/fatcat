
BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE READ ONLY DEFERRABLE;

COPY (SELECT id FROM changelog ORDER BY id DESC LIMIT 1)                         TO '/tmp/fatcat_ident_latest_changelog.tsv'    WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM creator_ident      WHERE is_live=true) TO '/tmp/fatcat_ident_creators.tsv'            WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM container_ident    WHERE is_live=true) TO '/tmp/fatcat_ident_containers.tsv'          WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM file_ident         WHERE is_live=true) TO '/tmp/fatcat_ident_files.tsv'               WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM fileset_ident      WHERE is_live=true) TO '/tmp/fatcat_ident_filesets.tsv'            WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM webcapture_ident   WHERE is_live=true) TO '/tmp/fatcat_ident_webcaptures.tsv'         WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM release_ident      WHERE is_live=true) TO '/tmp/fatcat_ident_releases.tsv'            WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM work_ident         WHERE is_live=true) TO '/tmp/fatcat_ident_works.tsv'               WITH NULL '';
COPY (SELECT id, editgroup_id, timestamp FROM changelog)                         TO '/tmp/fatcat_ident_changelog.tsv'           WITH NULL '';

ROLLBACK;
