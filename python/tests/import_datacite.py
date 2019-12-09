"""
Test datacite importer.

Datacite is a aggregator, hence inputs are quite varied.

Here is small sample of ID types taken from a sample:

    497344 "DOI"
     65013 "URL"
     22210 "CCDC"
     17853 "GBIF"
     17635 "Other"
     11474 "uri"
      9170 "Publisher ID"
      7775 "URN"
      6196 "DUCHAS"
      5624 "Handle"
      5056 "publisherId"

A nice tool, not yet existing tool (maybe named indigo) would do the following:

    $ shuf -n 100000 datacite.ndjson | indigo -t md > data.md

TODO(martin): Write tests.
"""
