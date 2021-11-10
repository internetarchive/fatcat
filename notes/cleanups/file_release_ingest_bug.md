
We want to find cases where the `file_edit` metadata for a file entity does not match the DOI of the release it is linked to.

## Background


Seems to mostly happen when the `link_source_id` DOI is not found during a
fatcat 'lookup', eg due to upstream DOI metadata not including a 'title' field
(required by fatcat schema). Many of these seem to be 'journal-issue' or
something like that. Presumably upstream (unpaywall) crawled and found some PDF
for the DOI and indexes that.

TODO: what was the specific code bug that caused this? time window of edits?
https://github.com/internetarchive/fatcat/commit/d8cf02dbdc6848be4de2710e54f4463d702b6e7c

TODO: how many empty-title Crossref DOIs are there? perhaps we should import these after all, in some situations? eg, pull `html_biblio`?
TODO: examples of other mismatches? edits from the same timespan with mismatching identifiers

## Queries

    file_edit
        ident_id
        rev_id
        extra_json->>'link_source_id'::text
    file_rev_release
        file_rev
        target_release_ident_id
    release_ident
        id
        rev_id
    release_rev
        id
        doi

Query some samples:

    SELECT file_edit.ident_id as file_ident, release_ident.id as release_ident, file_edit.extra_json->>'link_source_id' as file_edit_doi, release_rev.doi as release_doi
    FROM file_edit
        LEFT JOIN file_rev_release ON file_edit.rev_id = file_rev_release.file_rev
        LEFT JOIN release_ident ON file_rev_release.target_release_ident_id = release_ident.id
        LEFT JOIN release_rev ON release_rev.id = release_ident.rev_id
    WHERE
        file_edit.extra_json->>'ingest_request_source' = 'unpaywall'
        AND release_rev.doi != file_edit.extra_json->>'link_source_id'
    LIMIT 20;

                  file_ident              |            release_ident             |             file_edit_doi             |             release_doi             
    --------------------------------------+--------------------------------------+---------------------------------------+-------------------------------------
     8fe6eb2a-da3a-4e1c-b281-85fad3cae601 | 0a0cbabd-3f04-4296-94d5-b61c27fddee4 | 10.7167/2013/520930/dataset/3         | 10.7167/2013/520930
     cbb2d4e7-a9a7-41df-be1d-faf9dd552ee1 | d7b356c7-de6b-4885-b37d-ea603849a30a | 10.1525/elementa.124.t1               | 10.1525/elementa.124
     26242648-2a7b-42ea-b2ca-e4fba8f0a2c7 | d2647e07-fbb3-47f5-b06a-de2872bf84ec | 10.1023/a:1000301702170               | 10.1007/bfb0085374
     f774854a-3939-4aa9-bf73-7e0573908b88 | e19ce792-422d-4feb-92ba-dd8536a60618 | 10.32835/2223-5752.2018.17.14-21      | 10.31652/2412-1142-2018-50-151-156
     615ddf8c-2d91-4c1c-b04a-76b69f07b300 | e7af8377-9542-4711-9444-872f45521dc1 | 10.5644/sjm.10.1.00                   | 10.5644/sjm.10.1.01
     43fa1b53-eddb-4f31-96af-bc9bd3bb31b6 | e1ec5d8a-ff88-49f4-8540-73ee4906e119 | 10.1136/bjo.81.11.934                 | 10.1038/sj.bjc.6690790
     64b04cbc-ab3d-4cef-aff5-9d6b2532c47a | 43c387d1-e4c9-49f3-9995-552451a9a246 | 10.1023/a:1002015330987               | 10.1006/jnth.1998.2271
     31e6e1a8-8457-4c93-82d3-12a51c0dc1bb | a2ec4305-63d2-4164-8470-02bf5c4ba74c | 10.1186/1742-4690-2-s1-s84            | 10.1038/nm0703-847
     3461024a-1800-44a0-a7be-73f14bac99dd | 6849d39f-6bae-460b-8a9d-d6b86fbac2d3 | 10.17265/2328-2150/2014.11            | 10.1590/s0074-02762002000900009
     c2707e43-6a4b-4f5d-8fb4-babe16b42b4d | 82127faf-d125-4cd4-88b9-9a8618392019 | 10.1055/s-005-28993                   | 10.1055/s-0034-1380711
     0fe72304-1884-4dde-98c6-7cf3aec5bf31 | e3fe46cd-0205-42e4-8255-490e0eba38ea | 10.1787/5k4c1s0z2gs1-en               | 10.1787/9789282105931-2-en
     686a1508-b6c5-4060-9035-1dd8a4b85be1 | dcecd03d-e0f6-40ce-a376-7e64bd90bdca | 10.15406/jlprr.2.3                    | 10.15406/jlprr.2015.02.00039
     9403bc0e-2a50-4c8a-ab79-943bce20e583 | 3073dee2-c8f7-42ed-a727-12306d86fa35 | 10.1787/5jrtgzfl6g9w-en               | 10.1111/1468-2443.00004
     9a080a6f-aaaf-4f4b-acab-3eda2bfacc22 | af28a167-2ddc-43db-98ca-1cc29b863f9f | 10.13040/ijpsr.0975-8232.4(6).2094-05 | 10.1002/chin.201418290
     810f1a00-623f-4a5c-88e2-7240232ae8f9 | 639d4ddd-ef24-49dc-8d4e-ea626b0b85f8 | 10.13040/ijpsr.0975-8232.5(6).2216-24 | 10.1006/phrs.1997.0184
                                                                                   10.13040/IJPSR.0975-8232.5(6).2216-24
     72752c59-9532-467d-8b4f-fe4d994bcff5 | cc3db36c-27e9-45bf-96b0-56d1119b93d6 | 10.22201/facmed.2007865xe.2018.26     | 10.22201/facmed.2007865x.2018.26.01
     a0740fc2-a1db-4bc8-ac98-177ccebde24f | 0a7136ac-86ad-4636-9c2a-b56a6d5a3a27 | 10.24966/cmph-1978/vol6iss1           | 10.1093/milmed/146.4.283
     5b05df6a-7a0e-411b-a4e1-0081d7522673 | dc0cb585-c709-4990-a82e-c7c9b254fc74 | 10.15406/jig.3.1                      | 10.15406/jig.2016.03.00039
     011964dd-24a6-4d25-b9dd-921077f1947e | deabab6d-7381-4567-bf56-5294ab081e20 | 10.17265/1537-1506/2013.01            | 10.1109/icsssm.2016.7538527
     4d7efa59-30c7-4015-bea9-d7df171a97ed | a05449a0-34d7-4f45-9b92-bcf71db4918d | 10.14413/herj.2017.01.09              | 10.14413/herj.2017.01.09.
    (20 rows)

Total counts, broken DOIs, by source:

    SELECT file_edit.extra_json->>'ingest_request_source' as source, COUNT(*) as broken_files
    FROM file_edit
        LEFT JOIN file_rev_release ON file_edit.rev_id = file_rev_release.file_rev
        LEFT JOIN release_ident ON file_rev_release.target_release_ident_id = release_ident.id
        LEFT JOIN release_rev ON release_rev.id = release_ident.rev_id
    WHERE
        file_edit.extra_json->>'link_source_id' IS NOT NULL
        AND file_edit.extra_json->>'link_source_id' LIKE '10.%'
        AND release_rev.doi != file_edit.extra_json->>'link_source_id'
    GROUP BY file_edit.extra_json->>'ingest_request_source';


          source      | broken_files
    ------------------+--------------
     fatcat-changelog |          872
     unpaywall        |       227954
    (2 rows)

## Cleanup Pseudocode

Dump file idents from above SQL queries. Transform from UUID to fatcat ident.

For each file ident, fetch file entity, with releases expanded. Also fetch edit
history. Filter by:

- single release for file
- single edit in history, matching broken agent
- edit metadata indicates a DOI, which doesn't match release DOI (or release doesn't have DOI?)

If all that is the case, do a lookup by the correct DOI. If there is a match,
update the file entity to point to that release (and only that release).
Operate in batches.

Write cleanup bot, run in QA to test. Run SQL query again and iterate if some
files were not updated. Merge to master, run full dump and update in prod.

## Export Matches Needing Fix

    COPY (
        SELECT row_to_json(row.*) FROM (
            SELECT file_edit.ident_id as file_ident, release_ident.id as wrong_release_ident, file_edit.extra_json as edit_extra
            FROM file_edit
                LEFT JOIN file_rev_release ON file_edit.rev_id = file_rev_release.file_rev
                LEFT JOIN release_ident ON file_rev_release.target_release_ident_id = release_ident.id
                LEFT JOIN release_rev ON release_rev.id = release_ident.rev_id
            WHERE
                file_edit.extra_json->>'link_source_id' IS NOT NULL
                AND file_edit.extra_json->>'link_source_id' LIKE '10.%'
                AND release_rev.doi != file_edit.extra_json->>'link_source_id'
            -- LIMIT 20;
        ) as row
    ) TO '/srv/fatcat/snapshots/file_release_bugfix_20211105.unfiltered.json';
    # COPY 228,827

Then need to do a `sed` pass, and a `rg` pass, to filter out bad escapes:

    cat /srv/fatcat/snapshots/file_release_bugfix_20211105.unfiltered.json \
        | sed 's/\\"/\"/g' \
        | rg -v "\\\\" \
        | rg '^\{' \
        | pv -l \
        > /srv/fatcat/snapshots/file_release_bugfix_20211105.json
    # 228k 0:00:00 [ 667k/s]

    wc -l /srv/fatcat/snapshots/file_release_bugfix_20211105.json
    # 228,826

And create a sample file:

    shuf -n10000 /srv/fatcat/snapshots/file_release_bugfix_20211105.json > /srv/fatcat/snapshots/file_release_bugfix_20211105.10k_sample.json

