
This script is used to "partition" (split up) a complete JSON dump by some key.
For example, split release dump JSON lines into separate files, one per
journal/container.

Example parititoning a sample by release type:

    cat release_export_expanded_sample.json | jq .release_type -r > release_export_expanded_sample.release_type
    cat release_export_expanded_sample.release_type | sort -S 4G | uniq -c | sort -S 500M -nr > release_export_expanded_sample.release_type.counts
    cat release_export_expanded_sample.json | paste release_export_expanded_sample.release_type - | sort -S 4G > out

More production-y example using ISSN-L:

    # will append otherwise
    rm -rf ./partitioned

    # it's a pretty huge sort, will need 300+ GB scratch space? this might not scale.
    zcat release_export_expanded.json.gz | jq .container.issnl -r > release_export_expanded.issnl
    zcat release_export_expanded.json.gz | paste release_export_expanded.issnl - | sort -S 8G | ./partition_script.py

    # for verification/stats
    cat release_export_expanded.issnl | sort -S 1G | uniq -c | sort -S 1G -nr > release_export_expanded.issnl.counts
    
    # cleanup
    rm release_export_expanded.issnl
