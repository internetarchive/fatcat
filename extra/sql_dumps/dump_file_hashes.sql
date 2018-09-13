
BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE READ ONLY DEFERRABLE;

COPY (SELECT file_ident.id, file_rev.id, file_rev.sha1, file_rev.sha256, file_rev.md5
      FROM file_rev
      INNER JOIN file_ident ON file_ident.rev_id = file_rev.id
      WHERE file_ident.is_live = 't' AND file_ident.redirect_id IS NULL)
    TO STDOUT
    WITH NULL '';

ROLLBACK;
