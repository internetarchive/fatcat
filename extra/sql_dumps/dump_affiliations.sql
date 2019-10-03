
BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE READ ONLY DEFERRABLE;

COPY (SELECT release_ident.id, release_contrib.raw_affiliation
      FROM release_contrib
      INNER JOIN release_ident ON release_ident.rev_id = release_contrib.release_rev
      WHERE release_ident.is_live = 't' AND release_ident.redirect_id IS NULL 
          AND release_contrib.raw_affiliation IS NOT NULL)
    TO '/tmp/fatcat_affiliations.tsv'
    WITH NULL '';

ROLLBACK;

-- Post processing:
--
--  cut -f2 fatcat_affiliations.tsv | sort -S 4G | uniq -c | sort -nr | gzip > fatcat_affiliations.counts.txt.gz
--  gzip fatcat_affiliations.tsv
