
## Production Run

Start small:

    export FATCAT_AUTH_WORKER_CLEANUP=[...]

    wc -l /srv/fatcat/datasets/file_release_bugfix_20211105.json
    228826

    head -n100 /srv/fatcat/datasets/file_release_bugfix_20211105.json \
        | python -m fatcat_tools.cleanups.file_release_bugfix -
    # Counter({'total': 100, 'update': 100, 'skip': 0, 'insert': 0, 'exists': 0})

    # example editgroup_keae3rfekffuriiy77f26rf6uq

These are all now stubs (no release associated), which isn't the ratio seen in QA. Going to do a random sample:

    shuf -n100 /srv/fatcat/datasets/file_release_bugfix_20211105.json \
        | python -m fatcat_tools.cleanups.file_release_bugfix -
    # Counter({'total': 100, 'update': 100, 'skip': 0, 'insert': 0, 'exists': 0})

    # example editgroup_34mk525kxvdu3hak7g7fr7awru

Looked at a few and all looked more like what would be expected, correct matches.

Comparing before and after counts is going to be tricky, and will require a
full re-index for an accurate count. But did do a snapshot just before this run
(2021-11-10-prod-stats.json), and got 31,110,184 `in_web`.

Full edit, in parallel:

    cat /srv/fatcat/datasets/file_release_bugfix_20211105.json \
        | pv -l \
        | parallel -j8 --linebuffer --round-robin --pipe python -m fatcat_tools.cleanups.file_release_bugfix -
    # 228k 0:26:34 [ 143 /s]
    # Counter({'total': 26090, 'update': 26071, 'skip-existing-fixed': 15, 'skip': 4, 'skip-wrong-release-is-ok': 4, 'insert': 0, 'exists': 0})
    # Counter({'total': 26080, 'update': 26061, 'skip-existing-fixed': 12, 'skip': 7, 'skip-wrong-release-is-ok': 7, 'insert': 0, 'exists': 0})
    # Counter({'total': 27517, 'update': 27497, 'skip-existing-fixed': 15, 'skip': 5, 'skip-wrong-release-is-ok': 5, 'insert': 0, 'exists': 0})
    # Counter({'total': 29534, 'update': 29420, 'skip-existing-fixed': 110, 'skip': 4, 'skip-wrong-release-is-ok': 4, 'insert': 0, 'exists': 0})
    # Counter({'total': 29544, 'update': 29517, 'skip-existing-fixed': 16, 'skip': 11, 'skip-wrong-release-is-ok': 11, 'insert': 0, 'exists': 0})
    # Counter({'total': 29535, 'update': 29518, 'skip-existing-fixed': 10, 'skip': 7, 'skip-wrong-release-is-ok': 7, 'insert': 0, 'exists': 0})
    # Counter({'total': 30082, 'update': 30065, 'skip-existing-fixed': 13, 'skip': 4, 'skip-wrong-release-is-ok': 4, 'insert': 0, 'exists': 0})
    # Counter({'total': 30444, 'update': 30420, 'skip-existing-fixed': 21, 'skip': 3, 'skip-wrong-release-is-ok': 3, 'insert': 0, 'exists': 0})

## Verification

Counts:

    SELECT file_edit.extra_json->>'ingest_request_source' as source, COUNT(*) as broken_files
    FROM file_edit
        LEFT JOIN file_ident ON file_edit.ident_id = file_ident.id
        LEFT JOIN file_rev_release ON file_edit.rev_id = file_rev_release.file_rev
        LEFT JOIN release_ident ON file_rev_release.target_release_ident_id = release_ident.id
        LEFT JOIN release_rev ON release_rev.id = release_ident.rev_id
    WHERE
        file_edit.extra_json->>'link_source_id' IS NOT NULL
        AND file_edit.extra_json->>'link_source_id' LIKE '10.%'
        AND lower(release_rev.doi) != lower(file_edit.extra_json->>'link_source_id')
        AND file_ident.rev_id = file_edit.rev_id
    GROUP BY file_edit.extra_json->>'ingest_request_source';

      source   | broken_files
    -----------+--------------
     unpaywall |          233
    (1 row)

Examples:

    SELECT file_edit.ident_id as file_ident, release_ident.id as release_ident, file_edit.extra_json->>'link_source_id' as file_edit_doi, release_rev.doi as release_doi
    FROM file_edit
        LEFT JOIN file_ident ON file_edit.ident_id = file_ident.id
        LEFT JOIN file_rev_release ON file_edit.rev_id = file_rev_release.file_rev
        LEFT JOIN release_ident ON file_rev_release.target_release_ident_id = release_ident.id
        LEFT JOIN release_rev ON release_rev.id = release_ident.rev_id
    WHERE
        file_edit.extra_json->>'link_source_id' IS NOT NULL
        AND file_edit.extra_json->>'link_source_id' LIKE '10.%'
        AND lower(release_rev.doi) != lower(file_edit.extra_json->>'link_source_id')
        AND file_ident.rev_id = file_edit.rev_id
    LIMIT 20;


Looks like many of the remaining mismatches are from "double-slash" normalization, with doi prefix 10.1037:

                  file_ident              |            release_ident             |        file_edit_doi         |         release_doi          
    --------------------------------------+--------------------------------------+------------------------------+------------------------------
     ae2f7864-66a6-4a82-a0e6-153cb4d0b03a | 0f436ae6-d7b4-4a45-a434-d158bc4a3437 | 10.1037/0096-1523.25.6.1568  | 10.1037//0096-1523.25.6.1568
     d02ff5ab-a882-4a86-8a94-ce6222708323 | 2d5ebbca-e4ba-4bb7-bb19-f1e081479eab | 10.1037//0021-9010.63.4.467  | 10.1037/0021-9010.63.4.467
     2c107387-f57f-4855-bc1a-e40704f1e9b4 | 7654b956-4776-4f6f-bc35-ccf7e6bfe99c | 10.1037/0022-0663.75.4.500   | 10.1037//0022-0663.75.4.500
     15e3636a-4bcf-4595-8a2a-b6b06a299a2f | c09e3531-1ac4-4bfa-9fcf-8acb9f0d845e | 10.1037//1064-1297.8.2.225   | 10.1037/1064-1297.8.2.225
     dc8b86c8-9b8e-4333-abbb-8811010d9c71 | bd91e7be-c360-47af-a634-f048e2c85b73 | 10.1037//0021-843x.105.4.637 | 10.1037/0021-843x.105.4.637
     35a06e0a-6f72-4624-87ca-fbb74bc9d77d | 96befa26-6eb0-47c0-a0ec-e00282e33bff | 10.1037//0735-7044.99.5.964  | 10.1037/0735-7044.99.5.964
     707bfaa1-65de-4dbb-9786-51b99d03d91d | 2d58524b-4216-4092-8ddf-336ac42d5955 | 10.1037/0096-1523.28.3.515   | 10.1037//0096-1523.28.3.515
     de9ea98f-672e-44ec-9d12-e11acd8990d0 | 20f1a857-ad51-4b80-9ce5-bc3a44df96b1 | 10.1037//0002-9432.71.1.72   | 10.1037/0002-9432.71.1.72
     4275306c-11ef-4fce-bc03-3f1efe99f9a6 | c69bc740-4da1-4f96-acc9-151a0cef5c3f | 10.1037//1064-1297.6.1.107   | 10.1037/1064-1297.6.1.107
     6a63d2ae-b953-48ba-a68a-061543d82ad4 | e3c8b8c1-defc-44ac-8c73-e21e8cf93f5c | 10.1037//0022-0167.23.6.557  | 10.1037/0022-0167.23.6.557
     2fcbb54e-8fa8-4bbc-a2ae-4b6b6eaff412 | 8500b4a5-a693-4415-b4a4-4dcfb3403d82 | 10.1037/0021-9010.73.1.68    | 10.1037//0021-9010.73.1.68
     b9aa4601-4a1b-4146-aa6b-a410d0fc3dce | 954f2072-8c53-41c7-82b0-8c6fe9ef4d0c | 10.1037//0278-6133.13.4.319  | 10.1037/0278-6133.13.4.319
     b528b924-0680-43f3-81ad-d822e51b3373 | 69387969-40bc-451d-b567-8713296f60b0 | 10.1037//0002-9432.71.1.38   | 10.1037/0002-9432.71.1.38
     f64f1ee2-b787-4a06-87ab-46b94f9d5454 | c082d47f-175d-456a-a741-650b5eaa5173 | 10.1037//0021-843x.98.4.487  | 10.1037/0021-843x.98.4.487
     86e8b655-963a-4c11-ae70-a8d528400682 | 6381254c-e339-4354-b0d8-711b5b5e4fcc | 10.1037/0022-0663.89.1.183   | 10.1037//0022-0663.89.1.183
     716e1761-8120-480b-b096-e7698c65456a | 7a4d2c7d-32b7-4292-adbf-b791387a3ac5 | 10.1037//0278-7393.21.5.1209 | 10.1037/0278-7393.21.5.1209
     bb7aa131-d5e5-497e-8040-b1729850b94c | 1ce29f33-d020-4d5f-a8b3-2b8bef53ccb8 | 10.1037//1040-3590.7.4.533   | 10.1037/1040-3590.7.4.533
     510ad392-43aa-42dc-9644-9697f425efd5 | 796c92ca-767a-495b-ad0c-f458381c071c | 10.1037//0278-7393.20.4.824  | 10.1037/0278-7393.20.4.824
     32c57e68-0793-4ded-bca2-d05f3532ff3e | 1567a003-0b5a-48bb-bb7c-7dff2c44b90b | 10.1037//1040-3590.13.1.59   | 10.1037/1040-3590.13.1.59
     e0d1fd38-17d8-42ac-b9df-60312829ddd4 | 0cefaf5d-fcdf-4049-a9d7-0c569096478e | 10.1037//0022-006x.56.4.621  | 10.1037/0022-006x.56.4.621
    (20 rows)
