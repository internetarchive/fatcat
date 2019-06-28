# Sources

The core metadata bootstrap sources, by entity type, are:

- `releases`: Crossref metadata, with DOIs as the primary identifier, and
  PubMed (central), Wikidata, and [CORE][] identifiers cross-referenced
- `containers`: munged metadata from the DOAJ, ROAD, and Norwegian journal
  list, with ISSN-Ls as the primary identifier. ISSN provides an "ISSN to
  ISSN-L" mapping to normalize electronic and print ISSN numbers.
- `creators`: ORCID metadata and identifier.

Initial `file` metadata and matches (file-to-release) come from earlier
Internet Archive matching efforts, and in particular efforts to extra
bibliographic metadata from PDFs (using GROBID) and fuzzy match (with
conservative settings) to Crossref metadata.

[CORE]: https://core.ac.uk

The intent is to continuously ingest and merge metadata from a small number of
large (~2-3 million more more records) general-purpose aggregators and catalogs
in a centralized fashion, using bots, and then support volunteers and
organizations in writing bots to merge high-quality metadata from field or
institution-specific catalogs.

Progeny information (where the metadata comes from, or who "makes specific
claims") is stored in edit metadata in the data model. Value-level attribution
can be achieved by looking at the full edit history for an entity as a series of
patches.

