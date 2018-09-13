
BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE READ ONLY DEFERRABLE;

COPY (SELECT release_ident.id, release_rev.id, release_rev.doi, release_rev.pmcid, release_rev.pmid,
             release_rev.core_id, release_rev.wikidata_qid
      FROM release_rev
      INNER JOIN release_ident ON release_ident.rev_id = release_rev.id
      WHERE release_ident.is_live = 't' AND release_ident.redirect_id IS NULL)
    TO STDOUT
    WITH NULL '';

ROLLBACK;
