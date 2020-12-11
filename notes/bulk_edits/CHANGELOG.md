
# Fatcat Production Import CHANGELOG

This file tracks major content (metadata) imports to the Fatcat production
database (at https://fatcat.wiki). It complements the code CHANGELOG file.

In general, changes that impact more than 50k entities will get logged here;
this file should probably get merged into the guide at some point.

This file should not turn in to a TODO list!

## 2020-12

Updated ORCIDs from 2020 dump. About 2.4 million new `creator` entities.

## 2020-03

Started harvesting both Arxiv and Pubmed metadata daily and importing to
fatcat. Did backfill imports for both sources.

JALC DOI registry update from 2019 dump.

## 2020-01

Imported around 2,500 new containers (journals, by ISSN-L) from chocula
analysis script.

Imported DOIs from Datacite (around 16 million, plus or minus a couple
million).

Imported new release entities from 2020 Pubmed/MEDLINE baseline. This import
included only new Pubmed works cataloged in 2019 (up until December or so).
Only a few hundred thousand new release entities.

Daily "ingest" (crawling) pipeline running.

## 2019-12

Started continuous harvesting Datacite DOI metadata; first date harvested was
`2019-12-13`. No importer running yet.

Imported about 3.3m new ORCID identifiers from 2019 bulk dump (after converting
from XML to JSON): <https://archive.org/details/orcid-dump-2019>

Inserted about 154k new arxiv release entities. Still no automatic daily
harvesting.

"Save Paper Now" importer running. This bot only *submits* editgroups for
review, doesn't auto-accept them.

## 2019-11

Daily ingest of fulltext for OA releases now enabled. New file entities created
and merged automatically.

## 2019-10

Inserted 1.45m new release entities from Crossref which had been missed during
a previous gap in continuous metadata harvesting.

Updated 304,308 file entities to remove broken
"https://web.archive.org/web/None/*" URLs.

## 2019-09

Created and updated metadata for tens of thousands of containers, using
"chocula" pipeline.

## 2019-08

Merged/fixed roughly 100 container entities with invalid ISSN-L numbers (eg,
invalid ISSN checksum).

## 2019-04

Imported files (matched to releases by DOI) from Semantic Scholar
(`DIRECT-OA-CRAWL-2019` crawl).

Imported files (matched to releases by DOI) from pre-1923/pre-1909 items uploaded
by a user to archive.org.

Imported files (matched to releases by DOI) from CORE.ac.uk
(`DIRECT-OA-CRAWL-2019` crawl).

Imported files (matched to releases by DOI) from the public web (including many
repositories) from the `UNPAYWALL` 2018 crawl.

## 2019-02

Bootstrapped!
