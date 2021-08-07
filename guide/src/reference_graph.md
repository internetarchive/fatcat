
# Reference Graph (refcat)

In Summer 2021, the first version of a reference graph dataset, named "refcat",
was released and integrated into the fatcat.wiki web interface. The dataset
contains billions of references between papers in the fatcat catalog, as well
as partial coverage of references from papers to books, to websites, and from
Wikipedia articles to papers. This is a first step towards identifying links
and references between scholarly works of all types preserved in archive.org.

The refcat dataset can be downloaded in JSON lines format from the archive.org
"[Fatcat Database Snapshots and Bulk Metadata Exports](https://archive.org/details/fatcat_snapshots_and_exports)"
collection, and is released under a CC-0 license for broad reuse.
Acknowledgement and attribution for both the aggregated dataset and the
original metadata sources is strongly encouraged (see below for provenance
notes).

References can be browsed on fatcat.wiki on an "outbound" ("References") and
"inbound" ("Cited By") basis for individual release entities. There are also
special pages for Wikipedia articles ("outbound", such as
[Internet](https://fatcat.wiki/wikipedia/en:Internet/refs-out)) and Open
Library books ("inbound", such as [The
Gift](https://fatcat.wiki/openlibrary/OL2670078W/refs-in)). JSON versions of
these pages are available, but do not yet represent a stable API.  The backend
reference graph is available via the [Elasticsearch API](./search_api.md) under
the `fatcat_ref` index, but these schema and semantics of this index are also
not yet stable.


## How It Works

Raw reference data comes from multiple sources (see "provenance" below), but
has the common structure of a "source" entity (which could be a paper,
Wikipedia article, etc) and a list of raw references. There might be duplicate
references for a single "source" work coming from different providers (eg, both
Pubmed and Crossref reference lists). The goal is to match as many references
as possible to the "target" work being referenced, creating a link from source
to target. If a robust match is not found, the "unmatched" reference is
retained and displayed in a human readable fashion if possible.

Depending on the source, raw references may be a simple "raw" string in an
arbitrary citation style; may have been parsed or structured in fields like
"title", "year", "volume", "issue"; might include a URL or identifier like an
arxiv.org identifier; or may have already been matched to a specific target
work by another party. It is also possible the reference is vague, malformed,
mis-parsed, or not even a reference to a specific work (eg, "personal
communication"). Based on the available structure, we might be able to do a
simple identifier lookup, or may need to parse a string, or do "fuzzy" matching
against various catalogs of known works. As a final step we take all original
and potential matches, verify the matches, and attempt to de-duplicate
references coming from different providers into a list of matched and unmatched
references as output. The refcat corpus is the output of this process.

Two dominant modes of reference matching are employed: identifier based
matching and fuzzy matching. Identifier based matching currently works with
DOI, Arxiv ids, PMID and PMCID and ISBN. Fuzzy matching employs a scalable way
to cluster documents (with pluggable clustering algorithms). For each cluster
of match candidates we run a more extensive verification process, which yields
a match confidence category, ranging from weak over strong to exact. Strong and
exact matches are included in the graph.

All the code for this process is available open source:

- [cgraph](https://gitlab.com/internetarchive/cgraph): batch processing and matching pipeline, in Python and Go
- [fuzzycat](https://gitlab.com/internetarchive/fuzzycat): Python verification code and "live" fuzzy matching


## Metadata Provenance

The provenance for each reference in the index is tracked and exposed via the
`match_provenance` field. A `fatcat-` prefix to the field means that the
reference came through the `refs` metadata field stored in the fatcat catalog,
but originally came from the indicated source. In the absence of `fatcat-`, the
reference was found, updated, or extracted at indexing time and is not recorded
in the `release` entity metadata.

Specific sources:

* `crossref` (and `fatcat-crossref`): citations deposited by publishers as part
  of DOI registration. Crossref is the largest single source of citation
  metadata in refcat. These references may be linked to a specific DOI; contain
  structured metadata fields; or be in the form of a raw citation string.
  Sometimes they are "complete" for the given work, and sometimes they only
  include references which could be matched/linked to a target work with a DOI.
* `fatcat-datacite`: same as `crossref`, but for the Datacite DOI registrar.
* `fatcat-pubmed`: references, linked or not linked, from Pubmed/MEDLINE
  metadata
* `fatcat`: references in fatcat where the original provenance can't be infered
  (but could be manually found by inspecting the release edit history)
* `grobid`: references parsed out of full-text PDFs using
  [GROBID](https://github.com/kermitt2/grobid)
* `wikipedia`: citations extracted from Wikipedia (see below for details)

Note that sources of reference metadata which have formal licensing
restrictions, even CC-BY or ODC-BY licenses as used by several similar
datasets, are not included in refcat.


## Current Limitations and Known Issues

The initial Summer 2021 version of the index has a number of limitations.
Feedback on features and coverage are welcome! We expect this dataset to be
iterated over regularly as there are a few
dimensions along which the dataset can be improved and extended.

The reference matching process is designed to eventually operate in both
"batch" and "live" modes, but currently only "batch" output is in the index.
This means that references from newly published papers are not added to the
index in an ongoing fashion.

Fatcat "release" entities (eg, papers) are matched from a Spring 2021 snapshot.
References to papers published after this time will not be linked.

Wikipedia citations come from the dataset [Wikipedia Citations: A comprehensive
dataset of citations with identifiers extracted from English
Wikipedia](https://zenodo.org/record/3940692), by Singh, West, and Colavizza.
This is a one-time corpus based on a May 2020 snapshot of English Wikipedia
only, and is missing many current references and citations. Additionally, only
direct identifier lookups (eg, DOI matches) are used, not fuzzy metadata
matching.

Open Library "target" matches are based on a
[snapshot](https://openlibrary.org/developers/dumps) of Open Library works, and
are matched either ISBN (extracted from citation string) or fuzzy metadata
matching.

Crossref references are extracted from a January 2021
[snapshot](https://archive.org/details/crossref_doi_dump_2021-01) of Crossref
metadata, and do not include many updates to existing works.

Hundreds of millions of raw citation strings ("unstructured") have not been
parsed into a structured for fuzzy matching. We plan to use GROBID to parse
these citation strings, in addition to the current use of GROBID parsing for
references from fulltext documents.

The current GROBID parsing used version v0.6.0. Newer versions of GROBID have
improved citation parsing accuracy, and we intend to re-parse all PDFs over
time. Additional manually-tagged training datasets could improve GROBID
performance even further.

In a future update, we intend to add Wayback (web archive) capture status and
access links for references to websites (distinct from references to online
journal articles or books). For example, references to an online news article
or blog post would indicate the closest (in time, to the "source" publication
date) Wayback captures to that web page, if available.

References are only displayed on fatcat.wiki, not yet on scholar.archive.org.

There is no current or planned mechanism for searching, sorting, or filtering
article search results by (inbound) citation count. This would require
resource-intensive transformations and continuous re-indexing of search
indexes.

It is unclear how the batch-generated refcat dataset and API-editable release
refs metadata will interact in the future. The original refs may eventually be
dropped from the fatcat API, or at some point the refcat corpus may stabilize
and be imported in to fatcat refs instead of being maintained as a separate
dataset and index. It would be good to retain a mechanism for human corrections
and overrides to the machine-generated reference graph.

