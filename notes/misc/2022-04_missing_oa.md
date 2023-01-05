
Short data exploration of what OA content is missing, and how it might be crawled.

Starting with "front page" query:

    is_oa:true year:>1995 year:<=2021 (type:article-journal OR type:article OR type:paper-conference) !doi_prefix:10.5281 !doi_prefix:10.6084

    doi_prefix:10.6084 is figshare
    doi_prefix:10.5281 is zenodo

    14,658,673	66.56%	preserved and publicly accessible (bright)
    3,453,052	15.68%	preserved but not publicly accessible (dark)
    3,911,614	17.77%	no known independent preservation
    22,023,339	100%	total

Virtually all of the "dark" is also `in_shadows:true`. So the
`preservation:none` is the high-impact target for crawling.

Limiting to `publisher_type:big5`, almost zero `preservation:none`, and 1.34
million (41%) dark.

## Publisher Type

Created a kibana graph of the above filters, graphing `publisher_type` ("Publisher Type breakdown of missing OA)":

    <missing>   1769k   54%
    longtail     852k   26%
    society      195k    6%
    unipress     130k    4%
    scielo       114k    3.5%
    then: repository, oa, commercial, big5

## Containers

    !container_id:* preservation:none is_oa:true year:>1995 year:<=2021 (type:article-journal OR type:article OR type:paper-conference) !doi_prefix:10.5281 !doi_prefix:10.6084

    1,993,639 missing preservation

These are virtually all Datacite DOIs (not including figshare/zenodo), and
start in 2008, ramping up. They are almost all missing `publisher_type` (which
makes sense because they have no container).

With the filters from above, here are some top containers missing content:

    Missing	                    1,993,639
    e27twid5qnbqbboxlkrja2xz2a	12,537
        "Proceedings of Indian National Science Academy"
        almost zero preservation. DOAJ website is 404 for article (!), no longer in DOAJ (!)
        some kind of bad metadata situation? almost all from 2015
    fmoqnzpewvfrnm2ni4mbvvlney	9,350
        "Chinese Medical Journal"
        PMIDs only
        missing/unpreserved is pre-2015 (significant!)
    7l5xye7sc5emxfprwmqw2a7yxq	8,999
        "Tidsskrift for Den norske legeforening" (norwegian medical)
        bunch of PMIDs only; sporadic preservation coverage
    ujftxdg3knebxhrqg4qjznz2he	5,903
        "International Research Journal" (russian)
        these are by-issue, with DOIs redirecting to pages inside issue (!)
    kfzef6kfwbhpnfw3cifit7zw7q	5,678
        "lectures"
        hosted on openeditions
        HTML ingest would work (!)
    gr4g5qzzcnembf4om6yjb6qf34	5,020
        "计算机科学"
        mostly via dblp. some DOIs, presumably chinese?
    bl77onlbbbhu5d6ohpjw2ypojy	4,994
        "EOS" (from American Geophysical Union / AGU)
        large publication, mostly preserved (dark)
        mix of wiley.com OA (but hard to crawl?) and web/HTML stuff
    3afvqhtpnjd5nmiphwxlxzirde	4,877
        "Medical Science Monitor"
        large publication, mixed preservation
        annoying PDF link situation (hard to crawl?)
    tulajqojzjabfc4iybyv6poi2e	4,786
        "Dermatology Online Journal"
        large publication, mixed preservation
        some just pmid
        some HTML or ePub-only
        escholarship.org

A take-away here for me is that containers are pretty heterogenous and have
diverse issues.

TODO: ingest things like: https://escholarship.org/uc/item/02v86610
    from container_tulajqojzjabfc4iybyv6poi2e

### revues.org / openedition

Many of these seem like they would ingest fine via HTML.

    doi_prefix:10.4000

    151,565	34.3%	preserved and publicly accessible (bright)
      7,211	1.64%	preserved but not publicly accessible (dark)
    283,139	64.08%	no known independent preservation
    441,915	100%	total

    article-journal	230,146	    63% preserved
    chapter	        200,724	     2% preserved
    book	        10,971	    12% preserved
    paper-conference	74

Chapters and books don't seem as amenable to ingest... and indeed are mostly
not marked `is_oa:true`.

DONE: bulk html-mode ingest, expecting about 80k requests:

    doi_prefix:10.4000 in_ia:false type:article-journal is_oa:true

    ./fatcat_ingest.py --env prod --enqueue-kafka --kafka-hosts wbgrp-svc280.us.archive.org,wbgrp-svc284.us.archive.org,wbgrp-svc350.us.archive.org --kafka-request-topic sandcrawler-prod.ingest-file-requests-bulk \
        --ingest-type html \
        query "doi_prefix:10.4000 in_ia:false type:article-journal is_oa:true"
    => Expecting 80032 release objects in search queries
    => Counter({'ingest_request': 80032, 'elasticsearch_release': 80032, 'estimate': 80032, 'kafka': 80032})

NOTE: have this be the default ingest type for this DOI prefix? not sure, some
do come through as PDF just fine

## Source of Records

Starting with the 3,844,142 or so `preservation:none`.

    doi                 3.204m
        datacite            1.995m
        crossref            1.087m
        <unknown>           109k
        jalc                12k
    doaj_id             553k
    pmid                192k
    dblp_id             29k
    arxiv_id, pmcid     0

I'm surprised how good dblp coverage is? Oh, but those are almost entirely
missing OA status, that explains it.

    # NOTE: not specifically OA
    dblp_id:* year:>1995 year:<=2021 (type:article-journal OR type:article OR type:paper-conference)

    406,235	    22.54%	preserved and publicly accessible (bright)
    59,009	    3.28%	preserved but not publicly accessible (dark)
    1,337,554	74.2%	no known independent preservation
    1,802,798	100%	total

Looks like doi and DOAJ are big sources.

    # NOTE: DOAJ implies OA, I checked and numbers are ~same
    doaj_id:* is_oa:true year:>1995 year:<=2021 (type:article-journal OR type:article OR type:paper-conference)

    588,364	    47.27%	preserved and publicly accessible (bright)
    103,206	    8.3%	preserved but not publicly accessible (dark)
    553,353	    44.45%	no known independent preservation
    1,244,923	100%	total

DOAJ ingest seems important to optimize!

    !publisher_type:big5 container_id:* doaj_id:* is_oa:true year:>1995 year:<=2021 (type:article-journal OR type:article OR type:paper-conference)
    => 548,709 missing preservation

    doaj_id:*
    => 589,915 missing preservation

Datacite the biggest category though, even with zenodo/figshare removed.

TODO: largest datacite DOI prefixes
TODO: check sandcrawler DB to see DOAJ ingest status; maybe these are entirely missing URLs? or just not crawling well?
TODO: dig in to "longtail" more... some random ones?

## Largest DOI Prefixes

    <missing>	640,104
    10.48550 	1,543,167
        the new arxiv.org prefix
    10.4000	68,267
        revues / openedition (handled above)
    10.25384	60,063
        figshare / SAGE
    10.3917	52,195
        cairn.info
    10.25673	41,565
        some random IR? opendata.uni-halle.de
        TODO: ingest this type of item, possibly using dataset->file crawler
    10.3406	33,778
        persee.fr
        blocks bots (don't attempt ingest)
    10.3205	33,540
        "german medical science"
        HTML articles, PDF links
        TODO: fix ingest
        https://www.egms.de/static/en/journals/gms/2020-18/000284.shtml
    10.17605	30,365
        osf.io
        TODO: fix ingest (?)
    10.25446	26,614
        figshare / oxford
        "File(s) not publicly available"
        but "CC BY 4.0"? ugh

TODO: HTML crawl cairn.info (10.3917)
TODO: ignore 10.25384, 10.25446 (figshare)
TODO: ignore arixv.org prefix (10.48550) in default dashboard
TODO: handle arxiv.org DOIs better (merge, count as preserved, etc)
