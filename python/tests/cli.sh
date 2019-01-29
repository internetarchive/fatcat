#!/bin/bash

set -eu -o pipefail
set -x

# This is a helper script to at least partially exersize the fatcat_*.py
# scripts. It expects to be run from the top-level directory, inside a 'pipenv
# shell' or 'pipenv run' invocation.

./fatcat_export.py changelog --start 1 --end 10 - > /dev/null

# no easy way to run this without harvest a whole day (at the moment)
./fatcat_harvest.py crossref -h

./fatcat_import.py crossref tests/files/crossref-works.2018-01-21.badsample.json tests/files/ISSN-to-ISSN-L.snip.txt
./fatcat_import.py orcid tests/files/0000-0001-8254-7103.json
./fatcat_import.py journal-metadata tests/files/journal_metadata.sample.json
./fatcat_import.py matched tests/files/matched_sample.json
./fatcat_import.py matched tests/files/example_matched.json
./fatcat_import.py grobid-metadata tests/files/example_grobid_metadata_lines.tsv

./fatcat_webface.py -h
./fatcat_worker.py -h

set +x
echo "Done running CLI examples (SUCCESS)"
