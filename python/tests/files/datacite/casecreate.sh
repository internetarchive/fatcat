#!/bin/bash
#
# casecreate.sh creates a new test case file pair by copying the last one.
#
set -eo pipefail

max=$(find . -name 'datacite_doc_*' | sort -n | tail -1 | grep -Eo '[0-9]+')
if [ -z $max ]; then
    echo "failed, expected datacite_doc_[NUMBER]..."
    exit 1
fi
new=$((max+1))
cp "datacite_doc_$max.json" "datacite_doc_$new.json"
cp "datacite_result_$max.json" "datacite_result_$new.json"
