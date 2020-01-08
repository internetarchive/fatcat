#!/bin/bash
#
# Open input and output in vertical vim split.
#
# $ caseview 13
#
view() {
    if [ -z "$1" ]; then
        echo usage: "$0" CASE-NUMBER
        exit 1
    else
        padded=$(printf "%02d\n" "$1")
        vim -O "datacite_doc_$padded.json" "datacite_result_$padded.json"
    fi
}

view "$@"
