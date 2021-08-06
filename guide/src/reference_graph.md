
# Reference Graph

Since 08/2021 references are available on an "inbound" and "outbound" basis in
the web interface.

The backend reference graph is available via the [Search Index](./search_api.md)
under the `fatcat_ref` index.

## Background and Mode of Operation

Release entities in fatcat have a [refs fields](./entity_release.md) which
contains citations, which in turn may be identified in different ways. Another
source of reference metadata is provided by structured data extraction from PDF
with tools such as [GROBID](https://grobid.readthedocs.io). The raw reference data combined
amounts to over 2B documents which we take as input for a batch process, that
derives the graph structure.

Two main modes of citation matching are employed: identifier based matching and
fuzzy matching. Identifier based matching currently works with DOI, Arxiv ids,
PMID and PMCID and ISBN. Fuzzy matching employs a scalable way to cluster
documents (with pluggable clustering algorithms). For each cluster of match
candidates we run a more extensive verification process, which yields a match
confidence category, ranging from weak over strong to exact. Strong and exact
matches are included in the graph.

The current reference search index contains both matches and yet unmatched
references. We expect this dataset to be iterated over regularly as there are
a few dimensions along which the dataset can be improved and extended.
