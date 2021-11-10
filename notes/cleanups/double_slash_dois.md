
Relevant github issue: https://github.com/internetarchive/fatcat/issues/48


## Investigate

At least some of these DOIs actually seem valid, like
`10.1026//1616-1041.3.2.86`. So shouldn't be re-writing them!

    zcat release_extid.tsv.gz \
        | cut -f1,3 \
        | rg '\t10\.\d+//' \
        | wc -l
    # 59,904

    zcat release_extid.tsv.gz \
        | cut -f1,3 \
        | rg '\t10\.\d+//' \
        | pv -l \
        > doubleslash_dois.tsv

Which prefixes have the most double slashes?

    cat doubleslash_dois.tsv | cut -f2 | cut -d/ -f1 | sort | uniq -c | sort -nr | head
      51220 10.1037
       2187 10.1026
       1316 10.1024
        826 10.1027
        823 10.14505
        443 10.17010
        186 10.46925
        163 10.37473
        122 10.18376
        118 10.29392
        [...]

All of the 10.1037 DOIs seem to be registered with Crossref, and at least some
have redirects to the not-with-double-slash versions. Not all doi.org lookups
include a redirect.

I think the "correct thing to do" here is to add special-case handling for the
pubmed and crossref importers, and in any other case allow double slashes.

Not clear that there are any specific cleanups to be done for now. A broader
"verify that DOIs are actually valid" push and cleanup would make sense; if
that happens checking for mangled double-slash DOIs would make sense.
