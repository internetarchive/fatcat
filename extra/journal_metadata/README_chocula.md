
## Fatcat Container Counts

    cat container_export.json | jq .issnl -r | sort -u > container_issnl.tsv
    cat container_issnl.tsv | parallel -j20 curl -s 'https://fatcat.wiki/container/issnl/{}/stats.json' > container_stats.json

Takes... more than 5 minutes but less than an hour.
