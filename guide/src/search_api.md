
# Search Index

The Elasticsearch indices used to power metadata search, statistics, and graphs
on the fatcat web interface are exposed publicly at
<https://search.fatcat.wiki/>. Third parties can make queries using the
Elasticsearch API, which is [well documented online](https://www.elastic.co/guide/en/elasticsearch/reference/7.10/search-search.html)
and has client libraries in many programming languages.

A thin proxy ([`es-public-proxy`](https://gitlab.com/bnewbold/es-public-proxy))
filters requests to avoid expensive queries which could cause problems for
search queries on the web interface, but most of the Elasticsearch API is
supported, including powerful aggregation queries. CORS headers are supported,
meaning that queries can be made directly from web browsers.

There is a short delay between updates to the fatcat catalog (via the main API)
and updates to the search index.

Notable indices include:

- `fatcat_release`: release entity metadata ([schema](https://search.fatcat.wiki/fatcat_release/_mapping))
- `fatcat_container`: container entity metadata ([schema](https://search.fatcat.wiki/fatcat_container/_mapping))
- `fatcat_ref`: reference graph ([schema](https://search.fatcat.wiki/fatcat_ref/_mapping))
- `scholar_fulltext`: [scholar.archive.org](https://scholar.archive.org) full-text index (body text can be queried, but not downloaded or extracted from index) ([schema](https://search.fatcat.wiki/scholar_fulltext/_mapping))

Schemas for these indices can be fetched directly from the index (eg,
<https://search.fatcat.wiki/fatcat_release/_mapping>), and are versioned in the
fatcat git repository under `fatcat:extra/eleasticsearch/`. They are a
simplification and transform of the regular entity schemas, and include some
synthesized fields (such as "preservation status" for releases). Note that the
search schemas are likely to change over time with less notice and stability
guarantees than the primary catalog API schema.
