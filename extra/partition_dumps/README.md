
This script is used to "partition" (split up) a complete JSON dump by some key.
For example, split release dump JSON lines into separate files, one per
journal/container.

Example parititoning a sample by release type:

    cat release_dump_expanded_sample.json | jq .release_type -r > release_dump_expanded_sample.release_type
    cat release_dump_expanded_sample.release_type | sort | uniq -c | sort -nr > release_dump_expanded_sample.release_type.counts
    cat release_dump_expanded_sample.json | paste release_dump_expanded_sample.release_type - | sort > out

More production-y example using ISSN-L:

    # will append otherwise
    rm -rf ./partitioned

    # it's a pretty huge sort, will need 300+ GB scratch space? this might not scale.
    zcat release_dump_expanded.json.gz | jq .container.issnl -r > release_dump_expanded.issnl
    zcat release_dump_expanded.json.gz | paste release_dump_expanded.issnl - | sort  | ./partition_script.py

    # for verification/stats
    cat release_dump_expanded.issnl | sort | uniq -c | sort -nr > release_dump_expanded.issnl.counts
    
    # cleanup
    rm release_dump_expanded.issnl
