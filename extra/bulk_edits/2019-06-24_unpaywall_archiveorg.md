
Goal is to import:

- UNPAYWALL-PDF-CRAWL-2019-04.published dataset; about 6 million lines, expect
  about half (3 million) new release fulltext matches
- archive.org fulltext, about 1.8 million files

## QA UNPAYWALL-PDF-CRAWL-2019-04

    export FATCAT_AUTH_WORKER_CRAWL=...

    # this wasn't a random sample
    zcat /srv/fatcat/datasets/UNPAYWALL-PDF-CRAWL-2019-04.published.json.gz | head -n200 | ./fatcat_import.py arabesque --json-file - --extid-type doi --crawl-id UNPAYWALL-PDF-CRAWL-2019-04

    # this was!
    zcat /srv/fatcat/datasets/UNPAYWALL-PDF-CRAWL-2019-04.published.json.gz | shuf -n200 | ./fatcat_import.py arabesque --json-file - --extid-type doi --crawl-id UNPAYWALL-PDF-CRAWL-2019-04
    [...]
    Counter({'total': 199, 'insert': 106, 'exists': 62, 'skip': 31, 'skip-extid-not-found': 20, 'skip-update-disabled': 1, 'update': 0})

    # ok, big import
    zcat /srv/fatcat/datasets/UNPAYWALL-PDF-CRAWL-2019-04.published.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py arabesque --json-file - --extid-type doi --crawl-id UNPAYWALL-PDF-CRAWL-2019-04
    # ran a few hundred thousand and looked good

## prod UNPAYWALL-PDF-CRAWL-2019-04

    export FATCAT_AUTH_WORKER_CRAWL=...

    zcat /srv/fatcat/datasets/UNPAYWALL-PDF-CRAWL-2019-04.published.json.gz | shuf -n200 | ./fatcat_import.py arabesque --json-file - --extid-type doi --crawl-id UNPAYWALL-PDF-CRAWL-2019-04
    Counter({'total': 198, 'insert': 115, 'exists': 56, 'skip': 27, 'skip-extid-not-found': 13, 'skip-update-disabled': 2, 'update': 0})

    zcat /srv/fatcat/datasets/UNPAYWALL-PDF-CRAWL-2019-04.published.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py arabesque --json-file - --extid-type doi --crawl-id UNPAYWALL-PDF-CRAWL-2019-04
    [...]
    Requiring GROBID status == 200
    Counter({'total': 520139, 'insert': 282524, 'exists': 155371, 'skip': 82244, 'skip-extid-not-found': 40000, 'skip-update-disabled': 6319, 'update': 0})
    19008.63user 729.06system 3:01:57elapsed 180%CPU (0avgtext+0avgdata 51440maxresident)k
    8552inputs+4335032outputs (54major+370893minor)pagefaults 0swaps

    (loosely repeated 12x times, of course)

    oh no, lots of duplicate inserts... ugh. needed a uniq in there, but really
    only "one hit per file" export. or a shuf? blech.

    (python)fatcat@wbgrp-svc502:/srv/fatcat/datasets$ zcat UNPAYWALL-PDF-CRAWL-2019-04.published.json.gz | jq .final_sha1 -r | sort -S 4G -u | wc -l
    5621882
    (python)fatcat@wbgrp-svc502:/srv/fatcat/datasets$ zcat UNPAYWALL-PDF-CRAWL-2019-04.published.json.gz | wc -l
    6181191

    ugh. how did this get missed in QA? sloppy.

fixup is going to be
- filter input list for duplicated sha1hex
- for each duplicated sha1hex:
    - fetch file entity. if not single release_id, bail
    - fetch release expanded with files
    - find all files with same sha1 that *aren't* the fetched file
    - print file entity id
- iterate over file entity ids, batches of 100x
    - create editgroup
    - delete files
    - accept editgroup

fetch_dupes.py:

    
    #!/usr/bin/env python3

    import sys
    import fatcat_client
    from fatcat_tools import public_api

    def do_sha1(api, sha1hex):
        try:
            fe = api.lookup_file(sha1=sha1hex)
        except:
            return
        if len(fe.release_ids) != 1:
            return
        try:
            re = api.get_release(fe.release_ids[0], expand='files', hide='refs,contribs,abstracts')
        except:
            return
        for f in re.files:
            if f.sha1 == fe.sha1 and f.ident != fe.ident and f.release_ids == [re.ident]:
                print(f.ident)

    def run():
        api = public_api('https://api.qa.fatcat.wiki/v0')
        for l in sys.stdin:
            if l:
                do_sha1(api, l.strip())

    if __name__ == '__main__':
        run()

delete_dupes.py:


    #!/usr/bin/env python3

    import sys
    import fatcat_client
    from fatcat_tools import authenticated_api

    #API_ENDPOINT = 'https://api.qa.fatcat.wiki/v0'
    API_ENDPOINT = 'https://api.fatcat.wiki/v0'

    def do_batch(api, batch):
        eg = api.create_editgroup(
            fatcat_client.Editgroup(description="Cleaning up duplicated file insertions from UNPAYWALL-CRAWL-2019-04 insert"))
        for ident in batch:
            api.delete_file(eg.editgroup_id, ident)
        api.accept_editgroup(eg.editgroup_id)
        print("deleted {} - {}...".format(eg.editgroup_id, len(batch)))

    def run():
        api = authenticated_api(API_ENDPOINT)
        batch = []
        for l in sys.stdin:
            l = l.strip()
            if not l:
                continue
            try:
                fe = api.get_file(l)
            except:
                continue
            if fe.state == 'active' and fe.release_ids:
                batch.append(l)
                if len(batch) >= 100:
                    do_batch(api, batch)
                    batch = []
        if batch:
            do_batch(api, batch)

    if __name__ == '__main__':
        run()

commands:

    zcat UNPAYWALL-PDF-CRAWL-2019-04.published.json.gz | jq .final_sha1 -r | b32_hex.py | sort -S 4G | uniq -d > repeated_sha1.tsv

    cat repeated_sha1.tsv | pv -l | ./fetch_dupes.py > repeated_file_idents.tsv

    export FATCAT_API_AUTH_TOKEN=... (crawl bot)
    cat repeated_file_idents.tsv | ./delete_dupes.py

## QA archive.org files

Start with arxiv:

    # FATCAT_AUTH_WORKER_ARCHIVE_ORG
    export FATCAT_API_AUTH_TOKEN=...

    # had a 500 "unexpected internal error: invalid length at 196", which was
    due to syntax error in API token. should have a better error response

    # try sample of arxiv_id
    zcat /srv/fatcat/datasets/arxiv.match.json.gz | head -n100 | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -
    Counter({'skip': 100, 'total': 100, 'skip-no-releases': 72, 'skip-no-urls': 28, 'update': 0, 'insert': 0, 'exists': 0})

    # TODO: shouldn't re-insert if URL already in there under a different reltyp

    # Ok, made a bunch of code changes to "clean up" at least arxiv URLs. All
    # arxiv.org files should be 1-to-1 with releases that have full arxiv_ids

Ok, try JSTOR:

    zcat /srv/fatcat/datasets/jstor.match.json.gz | shuf -n1000 | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -
    [...]
    Counter({'total': 1000, 'skip': 763, 'skip-no-releases': 763, 'insert': 162, 'exists': 74, 'update': 1})

larger import got:

    HTTP response body: {"success":false,"error":"ConstraintViolation","message":"unexpected database error: duplicate key value violates unique constraint \"file_edit_editgroup_id_ident_id_key\""}

could try getting around this with shuf?

    zcat /srv/fatcat/datasets/jstor.match.json.gz | shuf | pv -l | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -

got the same errors so added "inflight" edit protection and rolled back to earlier command:

    zcat /srv/fatcat/datasets/jstor.match.json.gz | shuf -n1000 | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -
    [...]
    451k 0:25:32 [ 294 /s]
    Counter({'total': 451178, 'skip-no-releases': 351287, 'skip': 351287, 'insert': 59644, 'exists': 39198, 'update': 1049, 'skip-update-inflight': 26})

many/most of these files were already in fatcat due to earlier "paper-manifest"
work... keep forgetting that!

ok, next pmc:

    zcat /srv/fatcat/datasets/pmc.match.json.gz | shuf -n1000 | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -
    [...]
    Counter({'total': 1000, 'exists': 895, 'insert': 77, 'skip-no-releases': 22, 'skip': 22, 'update': 6})

    that's a surprisingly large fraction (2.2%) with `skip-no-releases`. some
    because pubmed import failed, some because multiple PMCID identifiers? hrm.

ok, an finally paper-doi:

    zcat /srv/fatcat/datasets/paper-doi.match.json.gz | shuf -n1000 | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -
    [...]
    Counter({'total': 1000, 'exists': 720, 'insert': 280, 'update': 0, 'skip': 0})

    lots exist! probably from the pre-1923 stuff? yup.

## prod archive.org files

    # try sample of arxiv_id
    zcat /srv/fatcat/datasets/arxiv.match.json.gz | shuf -n100 | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -
    Counter({'total': 100, 'insert': 80, 'update': 20, 'exists': 0, 'skip': 0})

    # all arxiv_id
    zcat /srv/fatcat/datasets/arxiv.match.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py --editgroup-description-override '"Import fulltext from archive.org journals collection"' matched --default-mimetype application/pdf --default-link-rel archive -
    [...]
    Counter({'total': 62296, 'insert': 49503, 'update': 12413, 'exists': 269, 'skip': 111, 'skip-no-releases': 111, 'skip-update-inflight': 10})
    2497.78user 98.87system 27:31.22elapsed 157%CPU (0avgtext+0avgdata 47604maxresident)k
    360inputs+266104outputs (3major+265297minor)pagefaults 0swaps

    # derp, some of those were crawl-bot but should have been archive-org-bot. ctrl-c and re-ran

    # sample jstor
    zcat /srv/fatcat/datasets/jstor.match.json.gz | shuf -n100 | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -
    Counter({'total': 100, 'insert': 69, 'exists': 29, 'update': 1, 'skip': 1, 'skip-no-releases': 1})

    # all jstor
    zcat /srv/fatcat/datasets/jstor.match.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py --editgroup-description-override '"Import fulltext from archive.org journals collection"' matched --default-mimetype application/pdf --default-link-rel archive -
    [...]
    Counter({'total': 41072, 'insert': 27783, 'exists': 11926, 'update': 1307, 'skip-update-inflight': 117, 'skip': 56, 'skip-no-releases': 56})
    1257.93user 54.42system 12:45.96elapsed 171%CPU (0avgtext+0avgdata 45248maxresident)k
    5384inputs+157016outputs (38major+259749minor)pagefaults 0swaps

    good, pretty low `skip-no-releases` for JSTOR imports

    # sample pmc
    zcat /srv/fatcat/datasets/pmc.match.json.gz | shuf -n100 | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -
    Counter({'total': 100, 'exists': 92, 'insert': 5, 'skip-no-releases': 2, 'skip': 2, 'update': 1})

    interesting, at least one longtail file which is actually known: https://fatcat.wiki/file/xnc3sarc3jfsnceeagn34zi5la
    almost all known!

    # all pmc
    zcat /srv/fatcat/datasets/pmc.match.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py --editgroup-description-override '"Import fulltext from archive.org journals collection"' matched --default-mimetype application/pdf --default-link-rel archive -
    [...]
    Counter({'total': 18720, 'exists': 16701, 'insert': 1461, 'skip': 357, 'skip-no-releases': 357, 'update': 201, 'skip-update-inflight': 1})

    # sample paper-doi
    zcat /srv/fatcat/datasets/paper-doi.match.json.gz | shuf -n100 | ./fatcat_import.py --editgroup-description-override "Import fulltext from archive.org journals collection" matched --default-mimetype application/pdf --default-link-rel archive -
    Counter({'total': 100, 'exists': 73, 'insert': 27, 'skip': 0, 'update': 0})

    # all paper-doi
    zcat /srv/fatcat/datasets/paper-doi.match.json.gz | pv -l | time parallel -j12 --round-robin --pipe ./fatcat_import.py --editgroup-description-override '"Import fulltext from archive.org journals collection"' matched --default-mimetype application/pdf --default-link-rel archive -
    Counter({'total': 3014, 'exists': 2280, 'insert': 734, 'update': 0, 'skip': 0})
    Counter({'total': 3464, 'exists': 2483, 'insert': 981, 'update': 0, 'skip': 0})
    Counter({'total': 3437, 'exists': 2303, 'insert': 1134, 'update': 0, 'skip': 0})
    Counter({'total': 3450, 'exists': 2379, 'insert': 1071, 'update': 0, 'skip': 0})
    Counter({'total': 3467, 'exists': 2486, 'insert': 981, 'skip': 0, 'update': 0})
    Counter({'total': 3481, 'exists': 2583, 'insert': 898, 'skip': 0, 'update': 0})
    Counter({'total': 3423, 'exists': 2178, 'insert': 1245, 'update': 0, 'skip': 0})
    62.41user 3.17system 0:31.18elapsed 210%CPU (0avgtext+0avgdata 49852maxresident)k
    96inputs+13184outputs (7major+159215minor)pagefaults 0swaps

All done!
