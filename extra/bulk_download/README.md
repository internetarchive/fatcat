
## Download Fatcat Fulltext from web.archive.org in Bulk

These quick-and-dirty directions use UNIX utilities to download from the
Internet Archive (either in the wayback machine or archive.org). To make a
proper mirror (eg, for research or preservation use), you would want to verify
hashes (fixity), handle additional retries, and handle files which are not
preserved in Internet Archive, retain linkage between files and fatcat
identifiers, etc.

You can download a file entity dump from the most recent "Bulk Metadata Export"
item from the [snapshots and exports collection](https://archive.org/details/fatcat_snapshots_and_exports?sort=-publicdate).

Create a TSV file containing the SHA1 and a single URL for each file
entity:

    zcat file_export.json.gz \
        | grep '"application/pdf"'
        | jq -cr '.sha1 as $sha1 | .urls | map(select((.url | startswith("https://web.archive.org/web/")) or (.url | startswith("https://archive.org/download/")))) | select(. != []) | [$sha1, .[0].url] | @tsv' \
        > fatcat_files_sha1_iaurl.tsv

Then use the GNU `parallel` command to call `curl` in parallel to fetch files.
The `-j` argument controls parallelism. Please don't create exessive load on
Internet Archive infrastructure by downloading with too many threads. 10
parallel threads is a decent amount of load.

    cat fatcat_files_sha1_iaurl.tsv \
        | awk '{print "curl -Lfs --write-out \"%{http_code}\\t" $1 "\\t%{url_effective}\\n\" \"" $2 "\" -o ", $1 ".pdf"}' \
        | parallel --bar -j4 {} \
        > fetch_status.log

This will write out a status log containing the HTTP status code, expected file
SHA1, and attempted URL. You can check for errors (and potentially try) with:

    grep -v "^200" fetch_status.log

Or, count status codes:

    awk '{s[$1]++} END {for(k in s){print k, s[k]}}' fetch_status.log | sort -nrk2

