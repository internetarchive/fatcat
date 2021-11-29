

## QA Runs

Trying on 2019-12-22, using Martin commit 18d411087007a30fbf027b87e30de42344119f0c from 2019-12-20.

Quick test:

    # this branch adds some new deps, so make sure to install them
    pipenv install --deploy --dev
    pipenv shell
    export FATCAT_AUTH_WORKER_DATACITE="..."
	xzcat /srv/fatcat/datasets/datacite.ndjson.xz | head -n100 | ./fatcat_import.py datacite - /srv/fatcat/datasets/20181203.ISSN-to-ISSN-L.txt --extid-map-file /srv/fatcat/datasets/release_ids.ia_munge_20180908.sqlite3

ISSUE: `--extid-map-file` not passed through, so drop the:

    --extid-map-file /srv/fatcat/datasets/release_ids.ia_munge_20180908.sqlite3

ISSUE: auth_var should be FATCAT_AUTH_WORKER_DATACITE

Test full parallel command:

    export FATCAT_AUTH_WORKER_DATACITE="..."
	time xzcat /srv/fatcat/datasets/datacite.ndjson.xz | head -n10000 | parallel -j20 --round-robin --pipe ./fatcat_import.py datacite - /srv/fatcat/datasets/20181203.ISSN-to-ISSN-L.txt --extid-map-file /srv/fatcat/datasets/release_ids.ia_munge_20180908.sqlite3

    real    0m30.017s
    user    3m5.576s
    sys     0m19.640s

Whole lot of:

    invalid literal for int() with base 10: '10,495'
    invalid literal for int() with base 10: '11,129'
    
    invalid literal for int() with base 10: 'n/a'
    invalid literal for int() with base 10: 'n/a'

    invalid literal for int() with base 10: 'OP98'
    invalid literal for int() with base 10: 'OP208'

    no mapped type: None
    no mapped type: None
    no mapped type: None

Re-ran above:

    real    0m27.764s
    user    3m2.448s
    sys     0m12.908s

Compare with `--lang-detect`:

    real    0m27.395s
    user    3m5.620s
    sys     0m13.344s

Not noticeable?

Whole run:

    export FATCAT_AUTH_WORKER_DATACITE="..."
	time xzcat /srv/fatcat/datasets/datacite.ndjson.xz | parallel -j20 --round-robin --pipe ./fatcat_import.py datacite - /srv/fatcat/datasets/20181203.ISSN-to-ISSN-L.txt --extid-map-file /srv/fatcat/datasets/release_ids.ia_munge_20180908.sqlite3

    real    35m21.051s
    user    98m57.448s
    sys     7m9.416s

Huh. Kind of suspiciously fast.

    select count(*) from editgroup where editor_id='07445cd2-cab2-4da5-9f84-34588b7296aa';
    => 9952 editgroups

    select count(*) from release_edit inner join editgroup on release_edit.editgroup_id = editgroup.id  where editgroup.editor_id='07445cd2-cab2-4da5-9f84-34588b7296aa';
    => 496,342 edits

While running:

    starting around 5k TPS in pg_activity
    starting size: 367.58G
    (this is after arxiv and some other changes on top of 2019-12-13 dump)
    host doing a load average of about 5.5; fatcatd at 115% CPU

    ending size: 371.43G

Actually seems like extremely few DOIs getting inserted? Hrm.

    xzcat /srv/fatcat/datasets/datacite.ndjson.xz | wc -l
    => 18,210,075

Last DOIs inserted were around: 10.7916/d81v6rqr

Suspect a bunch of errors or something and output getting mangled by all the
logging? Squelched logging and running again (using same DB/config), except
with `pv -l` inserted after `xzcat`.

Seem to run at a couple hundred records a second (very volatile).

    Counter({'total': 42919, 'insert': 21579, 'exists': 21334, 'skip': 6, 'skip-blank-title': 6, 'inserted.container': 1, 'update': 0})
    Counter({'total': 43396, 'insert': 23274, 'exists': 20120, 'skip-blank-title': 2, 'skip': 2, 'update': 0})

Ok! The actual errors:


    Traceback (most recent call last):
      File "./fatcat_import.py", line 507, in <module>
        main()
      File "./fatcat_import.py", line 504, in main
        args.func(args)
      File "./fatcat_import.py", line 182, in run_datacite
        JsonLinePusher(dci, args.json_file).run()
      File "/srv/fatcat/src/python/fatcat_tools/importers/common.py", line 559, in run
        self.importer.push_record(record)
      File "/srv/fatcat/src/python/fatcat_tools/importers/common.py", line 318, in push_record
        entity = self.parse_record(raw_record)
      File "/srv/fatcat/src/python/fatcat_tools/importers/datacite.py", line 447, in parse_record
        sha1 = hashlib.sha1(text.encode('utf-8')).hexdigest()
    AttributeError: 'list' object has no attribute 'encode'

    fatcat_openapi_client.exceptions.ApiException: (400) 
    Reason: Bad Request
    HTTP response headers: HTTPHeaderDict({'Content-Length': '186', 'Content-Type': 'application/json', 'Date': 'Mon, 23 Dec 2019 08:12:16 GMT', 'X-Clacks-Overhead': 'GNU aaronsw, jpb', 'X-Span-ID': '73b0b698-bf88-4721-b869-b322dbe90cbe'})
    HTTP response body: {"success":false,"error":"MalformedExternalId","message":"external identifier doesn't match required pattern for a DOI (expected, eg, '10.1234/aksjdfh'): 10.17167/mksz.2017.2.129–155"}


    Traceback (most recent call last):
      File "./fatcat_import.py", line 507, in <module>   
        main()
      File "./fatcat_import.py", line 504, in main
        args.func(args)
      File "./fatcat_import.py", line 182, in run_datacite
        JsonLinePusher(dci, args.json_file).run()
      File "/srv/fatcat/src/python/fatcat_tools/importers/common.py", line 559, in run
        self.importer.push_record(record)
      File "/srv/fatcat/src/python/fatcat_tools/importers/common.py", line 318, in push_record
        entity = self.parse_record(raw_record)
      File "/srv/fatcat/src/python/fatcat_tools/importers/datacite.py", line 447, in parse_record
        sha1 = hashlib.sha1(text.encode('utf-8')).hexdigest()
    AttributeError: 'list' object has no attribute 'encode'


    fatcat_openapi_client.exceptions.ApiException: (400) 
    Reason: Bad Request
    HTTP response headers: HTTPHeaderDict({'Content-Type': 'application/json', 'X-Span-ID': 'ca141ff4-83f7-4ee5-9256-91b23ec09e94', 'Content-Length': '188', 'X-Clacks-Overhead': 'GNU aaronsw, jpb', 'Date': 'Mon, 23 Dec 2019 08:11:25 GMT'})
    HTTP response body: {"success":false,"error":"ConstraintViolation","message":"unexpected database error: new row for relation \"release_contrib\" violates check constraint \"release_contrib_raw_name_check\""}

## Prod Import

Around first/second week of january. Needed to restart at least once due to
database deadlock on abstract inserts, which seems to be due to parallelism and
duplicated records in the bulk datacite dump.

TODO: specific command used by martin
