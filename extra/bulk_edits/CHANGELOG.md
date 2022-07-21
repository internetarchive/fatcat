
# Fatcat Production Import CHANGELOG

This file tracks major content (metadata) imports to the Fatcat production
database (at https://fatcat.wiki). It complements the code CHANGELOG file.

In general, changes that impact more than 50k entities will get logged here;
this file should probably get merged into the guide at some point.

This file should not turn in to a TODO list!

## 2022-07

Ran a journal-level metadata update, using chocula.

Cleaned up just under 500 releases with missing `container_id` from an older
DOAJ article import.

Imported roughly 100k releases from DOAJ, new since 2022-04.

Imported roughly 2.7 million new ORCiD `creator` entities, using the 2021 dump
(first update since 2020 dump).

Imported almost 1 million new DOI release entities from JALC, first update in
more than a year.

Imported at least 400 new dblp containers, and an unknown number of new dblp
release entities.

Cleaned up about a thousand containers with incorrect `publisher_type`, based
on current publisher name. Further updates will populate after the next chocula
import.


## 2022-04

Imported some initial fileset entities.

Updated about 25k file entities from isiarticles.com, which are samples (spam
for translation service) to remove release linkage and set
`content_scope=sample` (similar to the springer "page one" case).

## 2022-03

Ran a journal-level metadata update, using chocula.

Run a DOAJ article-level metadata import, yielding a couple hundred thousand
new release entities. Crawling and bulk ingest of HTML and PDF fulltext for
these articles also started.

## 2022-02

- removed `container_id` linkage for some Datacite DOI releases which are
  respository deposits of published papers (eg, PLOS OA papers mirrored in
  IRs). a few tens of thousands of releases.
- just over 150k "Revised Cambridge NCI database" chemical database DOIs
  updated from 'article' to 'entry'
- a few tens of thousands of Zenodo releases marked as spam (and `stub`)
- tens of thousands of no-longer-used Crossref DOIs marked as `stub`
- hundreds of test/dummy/null releases marked as `stub`

## 2021-11

Ran a series of cleanups. See background and prep notes in `notes/cleanups/`
and specific final commands in this directory. Quick summary:

- more than 9.5 million file entities had truncated timestamps wayback URLs,
  and were fixed with the full timestamps. there are still a small fraction
  (0.5%) which were identified but not corrected in this first pass
- over 140k release entities with non-lowercase DOIs were updated with
  lowercase DOI. all DOIs in current release entities now lowercase (at least,
  no ASCII uppercase characters found)
- over 220k file entities with incorrect release relation, due to an
  import-time code bug, were fixed. a couple hundred questionable cases remain,
  but are all mismatched due to DOI slash/double-slash issues and will not be
  fixed in an automated way.
- de-uplicated a few thousand file entities, on the basis of SHA-1 hash
- updated file metadata for around 160k file entities (a couple hundred
  thousand remain with partial metadata)


## 2021-06

Created new containers via chocula pipeline. Did not update any existing
chocula entities.

Ran DOAJ import manually, yielding almost 130k new release entities.

Ran dblp import manually, resulting in about 17k new release entities, as well
as 108 new containers. Note that 146k releases were not inserted due to
`skip-dblp-container-missing` and 203k due to `exists-fuzzy`.

## 2020-12

Updated ORCIDs from 2020 dump. About 2.4 million new `creator` entities.

Imported DOAJ article metadata from a 2020-11 dump. Crawled and imported
several hundred thousand file entities matched by DOAJ identifier. Updated
journal metadata using chocula took (before the release ingest). Filtered out
fuzzy-matching papers before importing.

Imported dblp from a 2020 snapshot, both containers (primarily for conferences
lacking an ISSN) and release entities (primarily conference papers). Filtered
out fuzzy-matching papers before importing.

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
