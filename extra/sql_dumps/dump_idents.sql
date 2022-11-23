
BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE READ ONLY DEFERRABLE;

COPY (SELECT id FROM changelog ORDER BY id DESC LIMIT 1)                         TO '/srv/fatcat/tmp/fatcat_ident_latest_changelog.tsv'    WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM creator_ident      WHERE is_live=true) TO '/srv/fatcat/tmp/fatcat_ident_creators.tsv'            WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM container_ident    WHERE is_live=true) TO '/srv/fatcat/tmp/fatcat_ident_containers.tsv'          WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM file_ident         WHERE is_live=true) TO '/srv/fatcat/tmp/fatcat_ident_files.tsv'               WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM fileset_ident      WHERE is_live=true) TO '/srv/fatcat/tmp/fatcat_ident_filesets.tsv'            WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM webcapture_ident   WHERE is_live=true) TO '/srv/fatcat/tmp/fatcat_ident_webcaptures.tsv'         WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM release_ident      WHERE is_live=true) TO '/srv/fatcat/tmp/fatcat_ident_releases.tsv'            WITH NULL '';
COPY (SELECT id, rev_id, redirect_id FROM work_ident         WHERE is_live=true) TO '/srv/fatcat/tmp/fatcat_ident_works.tsv'               WITH NULL '';
COPY (SELECT id, editgroup_id, timestamp FROM changelog)                         TO '/srv/fatcat/tmp/fatcat_ident_changelog.tsv'           WITH NULL '';
COPY (
  SELECT
    release_ident.id,
    release_ident.rev_id,
    release_ident.redirect_id,
    release_rev.work_ident_id
  FROM
    release_ident
    LEFT JOIN release_rev ON release_ident.rev_id = release_rev.id
  WHERE
    release_ident.is_live=true
    AND release_ident.redirect_id IS NULL
    AND release_ident.rev_id IS NOT NULL
  ORDER BY
    release_rev.work_ident_id ASC NULLS LAST
) TO '/srv/fatcat/tmp/fatcat_ident_releases_by_work.tsv'            WITH NULL '';

ROLLBACK;
