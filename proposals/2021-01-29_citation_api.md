
Describes schemas, APIs, use-cases, and data store for citation graph.

## Use Cases

**Outbound reference web pages:** on fatcat.wiki and scholar.archive.org, want
to have a page that lists all of the works cited by ("outgoing") a paper or
other fatcat release.

- query by fatcat `release_ident`
- nice to have: list references in the order they appear in the paper, and
  annotate with any "key" used in the source document itself (either an index
  number or a short name for the reference)
- need to have a formatted reference string for each reference, even if we have
  not "linked" to a specific fatcat release (aka, would need structured or
  unstructured citation text to display)


**Inbound reference web pages:** on fatcat.wiki and scholar.archive.org, want
to display a list of all works which cite a specific work ("inbound"
citations).

- query by fatcat `release_ident`, or possibly by `work_ident` and ability to
  say "cites a different version of the same work"
- nice to have: citation context snippet surrounding the citation
- like outbound, want to have good display options and access options for each
  entry
- nice to have: non-traditional works (eg, mentions from wikipedia)

**Inbound reference IA services:** OpenLibrary.org and/or web.archive.org might
want to show a count or list of papers that reference a web page (by URL) or
book (by openlibrary work identifier).

**Inbound reference counts:** ability to display number of inbound citation
links for a release or work, on demand. Eg, on a fatcat.wiki release landing
page. Not sure how important this use-case is.

**Bulk Metadata Releases:** we will want to share this citation graph as an
artifact. We can easily serialize this format into JSON and share that, or push
into a columnar file format like Parquet to get storage efficiency advances,
type/schema enforcement, and easier ingest and use for large-scale data
analysis.


## Schemas

First, a combined JSON/pydantic/elasticsearch object that represents a
reference from one thing to another, where the "source" must be known, but the
"target" may either be known ("matched") or ambiguous (eg, just a reference
string):

    BiblioRef ("bibliographic reference")
        _key: Optional[str] elasticsearch doc key
            ("release", source_release_ident, ref_index)
            ("wikipedia", source_wikipedia_article, ref_index)
        update_ts: Optional[datetime] elasticsearch doc timestamp

        # metadata about source of reference
        source_release_ident: Optional[str]
        source_work_ident: Optional[str]
        source_wikipedia_article: Optional[str]
            with lang prefix like "en:Superglue"
        source_release_stage: Optional[str]
        source_year: Optional[int]

        # context of the reference itself
        ref_index: int
            1-indexed, not 0-indexed
        ref_key: Optional[str]
            eg, "Lee86", "BIB23"
        ref_locator: Optional[str]
            eg, specific page number in the book being referenced, if
            applicable. Not used for, eg, first page of paper in a
            volume/issue.

        # target of reference (identifiers)
        target_release_ident: Optional[str]
        target_work_ident: Optional[str]
        target_openlibrary_work: Optional[str]
        target_url_surt: Optional[str]
        target_url: Optional[str]
            would not be stored in elasticsearch, but would be auto-generated
            by all "get" methods from the SURT, so calling code does not need
            to do SURT transform

        match_provenance: str
            crossref, pubmed, grobid, etc
            TODO: "ref_provenance"
        match_status: Optional[str]
            strong, weak, etc
            TODO: "match_strength"? "match_confidence"?
        match_reason: Optional[str]
            "doi", "isbn", "title-fuzzy, author", etc
            maybe "fuzzy-title-author"?

        target_unstructured: string (only if no release_ident link/match)
        target_csl: free-form JSON (only if no release_ident link/match)
            CSL-JSON schema (similar to ReleaseEntity schema, but not exactly)
            generated from unstructured by a GROBID parse, if needed

Then, two wrapper objects that add more complete metadata. These would be
pydantic/JSON objects, used in python code, and maybe exposed via API, but not
indexed in elasticsearch. These are the objects that would, eg, be used by
jinja templated to display lists of references in the user interface.

    AccessOption
        access_type: str
            describes type of access link
            controlled values: wayback, ia_file, repository, loginwall, etc
        access_url: str
            note: for `target_url` refs, would do a CDX lookup and this URL
            would be a valid/HTTP-200 web.archive.org capture URL
        mimetype: Optional[str]
            application/pdf, text/html, etc
            blank for landing pages
        size_bytes: Optional[int]
        thumbnail_url: Optional[str]

    EnrichedBiblioRef
        # enriched version of BiblioRef with complete ReleaseEntity object as
        # fetched from entity catalogs, if available. For example, fatcat API.
        biblio_ref: BiblioRef
        source_release/target_release: Optional[ReleaseEntity]
            complete ReleaseEntity from API, with optional expand/hide fields
        source_access/target_access: List[AccessOption]
        # TODO: target_openlibrary? source_wikipedia?


## Datastore

Would store in Elasticsearch as a live database, at least to start.

Example Elasticsearch index `fatcat_ref_v02_20210716` has 1.8 billion docs
(references), and consumes 435 GBytes of disk.

Might be reasonable to use PostgreSQL in the future, with more explicit control
over indexes and tuning for latency. But Elasticsearch is pretty easy to
operate (eg, replicas).


## Methods / Implementation

    get_outbound_refs(
        release_ident | work_ident | wikipedia_article,
        limit: int = 100,
        offset: Optional[int] = None,
    ) -> List[BiblioRef]

    get_inbound_refs(
        release_ident | work_ident | openlibrary_work | url_surt | url,
        consolidate_works: bool = True,
            # for work_ident lookups, whether to             
        filter_stage: List[str],
            # eg, only include "published" sources
        filter_type: List[str],
            # eg, only include "fatcat" sources, not "wikipedia" article refs
        limit: int = 25,
        offset: Optional[int] = None,
    ) -> List[BiblioRef]

    count_inbound_refs(...) -> int
        same parameters as get_inbound_refs(), but returns just a count

    # UNIMPLEMENTED
    #get_all_outbound_refs(...) -> List[BiblioRef]
    #get_all_inbound_refs(...) -> List[BiblioRef]
    #    same as get_outbound_refs()/get_inbound_refs(), but does a scroll (return list or iterator?)
    #    (optional; maybe not public)

    # run catalog API fetches for each and return "enriched" refs
    enrich_inbound_refs(refs: List[BiblioRef], hide, expand) -> List[EnrichedBiblioRef]
    enrich_outbound_refs(refs: List[BiblioRef], hide, expand) -> List[EnrichedBiblioRef]

## HTTP API Endpoints

Initial web endpoints, including unstable pseudo-APIs:

    fatcat.wiki/release/{release_ident}/refs-in (and .json)
    fatcat.wiki/release/{release_ident}/refs-out (and .json)
        &limit
        &offset
        &sort (for inbound)
        &filter_stage (for inbound)

    fatcat.wiki/openlibrary/{openlibrary_ident}/refs-in (and .json)
        &limit
        &offset
        &sort
        &filter_stage

    fatcat.wiki/web/refs-in (and .json)
        &url= (required)
        &limit
        &offset
        &sort (newest, oldest)
        &filter_stage

## Design Notes

This proposed schema is relatively close to what the "normalize" SQL table
would look like (many-to-many relationship).

Especially for "redistributing as bulk corpus", we might want to consider an
alternative data model which is a single source entity containing a list of
outbound references. Could even be a single source *work* for fatcat content,
with many release under the entity. One advantage of this is that source
metadata (eg, `release_ident`) is not duplicated on multiple rows.

We could have "source objects" as a data model in the database as well; this
would make "outbound" queries a trivial key lookup, instead of a query by
`source_release_ident`. However, for "inbound" reference queries, many large
rows would be returned, with unwanted metadata.

Another alternative design would be storing more metadata about source and
target in each row. This would remove the need to do separate
"hydration"/"enrich" fetches. This would probably blow up in the index size
though, and would require more aggressive re-indexing (in a live-updated
scenario). Eg, when a new fulltext file is updated (access option), would need
to update all citation records pointing to that work.

## Third-Party Comparison

Microsoft Academic provides a simple (source, destination) pair, at the
"edition" level. An additional citation context table, which is (source,
destination, context:str). A separate "PaperResources" table has typed URLs
(type can be project, data, code), flagged as "cites" or "own". Presumably this
allows mentions and citations of specific software and datasets, distinct from
software and datasets described as part of the contribution of the paper itself.

Open Citations REST API schema:

    occ_id: the OpenCitations Corpus local identifier of the citing bibliographic resource (e.g. "br/2384552");
    author: the semicolon-separated list of authors of the citing bibliographic resource;
    year: the year of publication of the citing bibliographic resource;
    title: the title of the citing bibliographic resource;
    source_title: the title of the venue where the citing bibliographic resource has been published;
    volume: the number of the volume in which the citing bibliographic resource has been published;
    issue: the number of the issue in which the citing bibliographic resource has been published;
    page: the starting and ending pages of the citing bibliographic resource in the context of the venue where it has been published;
    doi: the DOI of the citing bibliographic resource;
    occ_reference: the semicolon-separated OpenCitations Corpus local identifiers of all the bibliograhic resources cited by the citing bibliographic resource in consideration;
    doi_reference: the semicolon-separated DOIs of all the cited bibliograhic resources that have such identifier associated;
    citation_count: the number of citations received by the citing bibliographic resource.

## TODO / Questions

Should the enriched objects just extend the existing object type? Eg, have
fields that are only sometimes set (`Optional[]`), like we have with
`ReleaseEntity` (which always has `container_id` but only sometimes
a full `ContainerEntity` at `container`).
