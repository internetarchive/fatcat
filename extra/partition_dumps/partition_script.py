#!/usr/bin/env python3
"""
Reads key-prefixed JSON lines from stdin, and writes out to gzipped files under
./partitioned/.

Skips empty keys and "null" (to handle a jq common-case).

Eg, for tab-separated input:

    something   {"a": 1}
    something2  {"b": 2}

Will write to ./partitioned/something.json.gz:

    {"a": 1}

(and "b" object to ./partitioned/something2.json.gz)
"""

import os, sys, gzip

def run():
    last_prefix = None
    f = None
    os.makedirs('partitioned', exist_ok=True)
    
    for line in sys.stdin:
        (prefix, obj) = line.strip().split('\t')[:2]
        if not prefix or prefix == "null":
            continue
        if prefix != last_prefix:
            if f:
                f.close()
            f = gzip.GzipFile(f'partitioned/{prefix}.json.gz', 'a')
        f.write(obj.encode('utf-8'))
        f.write(b"\n")
        last_prefix = prefix
    if f:
        f.close()

if __name__=="__main__":
    run()
