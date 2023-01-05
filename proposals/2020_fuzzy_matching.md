
Status: mostly implemented (fuzzycat)

Bibliographic Entity Fuzzy Match and Verification
====================================================

This document summarizes work so far on entity "fuzzy matching" and match
verification, notes some specific upcoming use cases for such methods, and
proposes new generic routines to be implemented.

There are three main routines desired:

**Match verification:** given two bibliographic metadata records (or two entire
entities), confirm whether they refer to the exact same version of the entity.
Optionally, also determine if they are not the exact same entity, but are
variations of the same entity (eg, releases that should be grouped under a
shared work entity). Performance is not as critical as correctness. Should be a
pure, in-memory routine (not touch the network or external services).

**Fuzzy matching:** of bibliographic metadata to entity in current/live Fatcat
catalog. Eg, given complete or partial metadata about a paper or other entity,
return a list of candidate matches. Emphasis on recall over precision; better
to return false matches than missed records. This would likely hit the fatcat
elasticsearch indexes (search.fatcat.wiki), which are continuously updated to
be in sync with the catalog itself (API). Expected to scale to, eg, 5-10
lookups per second and operate over up to a couple million entities during
cleanup or merging operations. Should operate per record, on-demand, always
up-to-date (aka, not a batch process).

**Bulk fuzzy match:** a tool capable of matching hundreds or billions of raw
metadata records against the entire fatcat catalog, resulting in candidate
fuzzy match pairs or groupings. The two main applications are de-duplicating or
grouping releases over the entire catalog (aka, matching the catalog against
itself), matching billions of citations (structured partial metadata) against
release entities, and grouping citations which fail to match to existing
entities against each other. Likely to include a batch mode, though may also
include an efficient per-record lookup API as well.

As a terminology note, the outputs of a fuzzy match are called "candidate
matches", and the output of match verificatino would be "confirmed match" or
"confident match". A "self-match" is when an exact complete entity is compared
to a set it already exists in, and matches to itself (aka, fatcat entity
identifier is exact match). "Identifier matches" are an easy case when external
identifiers (like DOI, ISSN, ORCID) are used for matching; note that the Fatcat
lookup API always returns the first such match, when there may actually be
multiple entities with the same external identifier, so this may not always be
sufficient.


## Container Matching

Containers are a simpler case, so will discuss those first. Function signatures
would look like:

    verify_container_match(left:container, right:container) -> status:str
    match_container_fuzzy(record:??) -> [candidate:container]

Verify statuses could be:

- `exact`: all metadata fields match byte-for-byte
- `strong`: good confidence these are the same
- `weak`: could be a match, perhaps missing enough metadata to be confident
- `ambiguous`: not enough confidence that is or even isn't a match
- `not-match`: confident that these are *not* the same container (TODO: better word?)

Alternatively, could return a confidence floating point number, but that puts
burdern of interpretation on external code.

Fields of interest would be at least:

- name
- original name
- aliases
- publisher
- abbrev

Some test cases that match verification should probably handle (just making
these up):

- differences in title whitespace and capitalization
- title of one record has ISSN appended ("PLOS One" matches "PLOS One
  1932-6203")
- title of one record has publisher appended ("PLOS One" matches "PLOS One
  (Public Library of Science)")
- record with original (non-English) name in name field matches fatcat record
  where the non-Enlish title is in `original_title` field
- difference of "The" doesn't matter ("Lancet" matches "The Lancet")
- detect and reject bogus names

Nice to haves:

- records with only abbreviation or acroynm match (for "official" abbreviations
  and acronyms, which may need to be recorded as alias).
- records of full name match when the "official" title is now an acroynum (eg,
  "British Medical Journal" matches "The BMJ")
- optionally, detect and return `ambiguous` for a fixed list of known-ambiguous
  names (eg, two journals with very similar titles, name or acronym along can't
  distinguish)

Can't remember if we already have aliases stored in container entity `extra`
yet, but idea is to store "also known as" names to help with fuzzy matching.

The main application of these routines would be adding container linkage to
releases where we have a container name but no identifier (eg, an ISSN). We
have millions of releases (many with DOIs) that have a container name but no
linked container. We also want to import millions of papers based on GROBID
metadata alone, where we will likely only extract a journal name, and will want
to do lookups on that.

For the particular case of containers, we'll probably want to memoize results
(aka, cache lookups) for efficiency. Probably will also want a unified helper
function like:

    match_container(name:str, **kwargs) -> (status:str, match:Option<container>)

which would handle caching (maybe this is a class/object not a function?), call
`match_container_fuzzy()` to get candidates, call `verify_container_match()` on
each candidate, and ensure there is only one `exact` or `strong` match. Eg, if
there are multiple `weak` matches, should return `ambiguous` for the group
overall.

Another speculative application of these routines could be as part of chocula,
to try and find "true" ISSNs for records where we know the ISSN is not
registered or invalid (eg, ISSN checksum digit fails). Another would be to look
for duplicate container/ISSNs, where there are multiple ISSNs for the same
journal but only one has any papers actually published; in that case we would
maybe want to mark the other container entity as a "stub".

Test datasets:

- in kibana/ES, filter for "`container_id` does not exist" and
  "`container_name` exists"
- in sandcrawler, dump `grobid` table and look in the metadata snippet, which
  includes extracted journal info


## Release Matching (verification and fuzzy, not bulk fuzzy)

For release entities, the signature probably looks something like:

    verify_release_match(left:release, right:release) -> status:str
        how confident, based on metadata, that these are same release, or
        should be grouped under same work?
    exact_release_match(left:release, right:release) -> bool
    exact_work_match(left:release, right:release) -> bool

Instead of full release entities, we could use partial metadata in the form of
a python dataclass or named tuple. If we use a struct, will need a routine to
convert full entities to the partial struct; if we use full entities, will
probably want a helper to construct entities from partial metadata.

Don't know which fields would be needed, but guessing at least the following,
in priority order:

- if full entity: `ident` (and `work_id` for case of releases)
- `title`
- `authors` (at least list of surnames, possibly also raw and given names)
- `ext_ids`
- `release_year` (and full `release_date` if available)
- volume, issue, pages
- `subtitle`
- `original_title`
- container name (and `container_id` if available)
- `release_type` and `release_stage` (particularly for work/release grouping
  vs. match)

For releases, the "status" output could be:

- `strong` (good confidence)
- `weak` (seems good, but maybe not enough metadata to be sure)
- `work-strong` (good confidence that same work, but not same release)
- `work-weak`
- `ambiguous`
- `not-match` (name?)

Cases we should handle:

- ability to filter out self-matches from fuzzy matcher. Eg, if we want to find
  a duplicate of an existing release, should be able to filter out self-match
  (as a kwarg?)
- ability to filter out entire categories of matches. eg, match only to
  published works (`release_stage`), only to papers (`release_type`), only not
  match to longtail OA papers (`is_longtail_oa` ES flag)
- have an in-memory stoplist for ambiguous titles (eg, "abstract", "index")
- differences in punctuation, capitalization
- single-character typos
- subtitle/title matching (eg: {title: "Important Research", subtitle: "A
  History"} should match {title: "Important Research: A History", subtitle:
  None})
- record with only one author, surname only should be able to match against
  full author list. maybe a flag to indicate this case? eg, for matching some
  citation styles where only one author is listed
- years are +/- 1. certainly for pre-prints ("work matches"), but also
  different databases sometimes have different semantics about whether
  "publication date" or "submission date" or "indexed date" are used as the
  `release_date` (which we consider the "publication date"). Maybe "weak" match
  in this case
- (probably a lot more i'm not thinking of)

Nice to have (but harder?):

- journal/container matching where one side has an abbreviation, acronym, or
  alias of the other. does this require a network fetch? maybe cached or
  pre-loaded? may not be important, do testing first. Note that entire
  container entity is transcluded when doing full release entity API lookups.
  This may be important for citation matching/verification

Will probably want a helper function to check that a metadata record (or
entity) has enough metadata and seems in-scope for matching. For example, a
record with title of just "Abstract" should probably be blocked from any match
attempt, because there are so many records with that metadata. On the other
hand, if there is an external identifier (eg, DOI), could still attempt a
direct match. Maybe something like:

    can_verify_match(release:release) -> boolean

Test datasets:

- PDFs crawled as part of longtail OA crawls (see context below). We have both
  fatcat release already imported, and new GROBID-extracted works. Can use
  glutton fuzzy matching for comparison, or verify those glutton matches
- many, many unmatched reference strings. will do separate

Optionally, we could implement a glutton-compatible API with equivalent or
better performance for doing GROBID "header consolidation". If performance is
great, could even use it for reference conslidation at crawl/ingest time.


## Bulk Fuzzy

Current concept for this is to implement Semantic Scholar's algorithm
(described below) in any of python, golang, or rust, to function for all these
use-cases:

- grouping releases in fatcat which are variants of the same work (or, eg,
  publisher registered multiple DOIs for same paper by accident) into the same
  work
- reference matching from structured partial metadata to fatcat releases
- grouping of unmatched references (from reference matching) as structured
  partial metadata, with the goal of finding, eg, the "100,000 most cited
  papers which do not have a fatcat entity", and creating fatcat entities for
  them

Optionally, we could also architect/design this tool to replace biblio-glutton
for ingest-time "reference consolidation", by exposing a biblio-glutton
compatible API. If this isn't possible or hard it could become a later tool
instead. Eg, shouldn't sacrifice batch performance for this. In particular, for
ingest-time reference matching we'd want the backing corpus to be updated
continuously, which might be tricky or in conflict with batch-mode design.

## Existing Fatcat Work

### Hadoop matching pipeline

In Summer 2018, the fatcat project had a wonderful volunteer, Ellen Spertus.
Ellen implemented a batch fuzzy matching pipeline in Scala (using the Scalding
map/reduce framework) that ran on our Hadoop cluster. We used it to "join"
GROBID metadata from PDF files against a Crossref metadata dump.

The Scala job worked by converting input metadata records into simple "bibjson"
metadata subsets (plus keeping the original record identifiers, eg Crossref DOI
or PDF file hash, for later import). It created a key for each record by
normalizing the title (removing all whitespace and non-alphanumeric characters,
lower-casing, etc; we called this a "slug"), and filtering out keys from a
blocklist. We used the map/reduce framework to then join the two tables on
these keys, and then filtered the output pairs with a small bit of addiitonal
title similarity comparison logic, then dumped the list of pairs as a table to
Hadoop, sorted by join key (slug). I can't remember if we had any other
heuristics in the Scala code.

This was followed by a second processing stage in python, which iterated over
the full list. It grouped pairs by key, and discarded any groups that had too
many pairs (on the assumption that the titles were too generic). It then ran
additional quality checks (much easier/faster to implement in python) on year,
author names (eg, checking that the number of authors matched). The output of
this filtering was then fed into a fatcat importer which matched the PDFs to
releases based on DOI.

We got several million matches using this technique. In the end we really only
ran this pipeline once. Hadoop (and HBase in particular) ended up being
frustrating to work with, as jobs took hours or days to run, and many bugs
would only appear when run against the full dataset. A particular problem that
came up with the join approach was N^2 explosions for generic titles, where the
number of join rows would get very large (millions of rows) for generic titles,
even after we filtered out the top couple hundred most popular join keys.

TODO: should update this section with specific algorithms and parameters used by
reading the Scala and Python source

## Longtail OA Import Filtering

Not directly related to matching, but filtering mixed-quality metadata.

As part of Longtail OA preservation work, we ran a crawl of small OA journal
websites, and then ran GROBID over the resulting PDFs to extract metadata. We
then filtered the output metadata using quality heuristics, then inserted both
new releases (with no external identifiers, just the extracted metadata) and
the associated file.

The metadata filtering pipeline is interesting because of all the bad metadata
it detected. Eg, long titles, used the normalization and blocklist from the
Hadoop matching work, poor author metadata, etc.

A big problem that was only noticed after this import was that actually many of
the papers imported were duplicates of existing fatcat papers with DOIs or
other identifiers. This was because the crawl ended up spidering some general
purpose repositories, which contained copies of large-publisher OA papers (eg,
PLOS papers). The solution to this will use fuzzy matching in two ways. First,
for future imports of this type, we will fuzzy matches against the catalog to
check that there isn't already a metadata record; possibly link the file to
matched existing entities (based on confidence), but certainly don't create new
records unless sure that there isn't an existing one (`no-match`). Also need to
ensure there are not duplicates *within* each import batch, but either running
the import slowly in a single thread (so elasticsearch and the matching system
has time to synchronize), or doing a batch fuzzy match first. Or possibly some
other pre-filtering idea. Secondly, we can go back over these longtail OA works
(which are tagged as such in the fatcat catalog) and attempt to match them
against non-longtail fatcat releases (eg, those with existing PMID or DOI), and
merge the releases together (note: redirecting the longtail release to the full
one, resulting in a single release, not just doing work grouping of releases).

A separate problem from this import is that none of the papers have container
linkage (though many or all have container names successfully extracted). After
doing release-level merging, we should use container fuzzy matching to update
container linkage for these entities. We could potentially do this as a two
stage project where we dump all the container name strings, prioritize them by
release count, and iterate on container matching until we are happy with the
results, then run the actual release updates.

### biblio-glutton

[biblio-glutton][biblio-glutton] is a companion tool for GROBID to do record
matching and metadata enrichment of both "header" metadata (aka, extracted
metadata about the fulltext paper in the PDF itself) and references (extracted
from the reference/bibliography section of the fulltext paper). GROBID calls
this "consolidation".

biblio-glutton supports a couple different metadata index sources, including
Crossref dumps. IA has patched both GROBID and biblio-glutton to work with
fatcat metadata directly, and to embed fatcat release identifiers in output
TEI-XML when there is a match. We have found performance to be fine for header
consolidation, but too slow for reference consolidation. This is because there
is only one glutton lookup per PDF for header mode, vs 20-50 lookups per PDF for
reference consolidation, which takes longer than the overall PDF extraction. In
our default configuration, we only do header consolidation.

biblio-glutton runs as a REST API server, which can be queried separately from
the GROBID integration. It uses the JVM (can't remember if Java or Scala), and
works by doing an elasticsearch query to find candidates, then selects the best
candidate (if any) and looks up full metadata records from one or more LMDB
key/value databases. It requires it's own elasticsearch index with a custom
schema, and large LMDB files on disk (should be SSD for speed). The process of
updating the LMDB files and elasticsearch index are currently manual (with some
scripts), generated using bulk metadata fatcat dumps. This means the glutton
results get out of sync from the fatcat catalog.

The current update process involves stopping glutton (which means stopping all
GROBID processing) for a couple hours. Compare this to the fatcat search index
(search.fatcat.wiki), which is continuously updated from the changelog feed,
and even during index schema changes has zero (or near zero) downtime.

[biblio-glutton]: https://github.com/kermitt2/biblio-glutton

## Existing External Work and Reading

"The Lens MetaRecord and LensID: An open identifier system for aggregated
metadata and versioning of knowledge artefacts"
<https://osf.io/preprints/lissa/t56yh/>


### Semantic Scholar

Semantic Scholar described their technique for doing bulk reference matching
and entity merging, summarized here:

Fast candidate lookups are done using only a subset of the title. Title strings
are turned in to an array of normalized tokens (words), with stopwords (like
"a", "the") removed. Two separate indices are created: one with key as the the
first three tokens, and the other with the key as the last three tokens. For
each key, there will be a bucket of many papers (or just paper global
identifiers). A per-paper lookup by title will fetch candidates from both
indices. It is also possible to iterate over both indices by bucket and doing
further processing between all the papers, then combined the matches/groups
from both iterations. The reason for using two indices is to be robust against
mangled metadata where there is added junk or missing words at either the
beginning or end of the title.

To verify candidate pairs, the Jaccard similarity is calculated between the
full original title strings. This flexibly allows for character typos (human or
OCR), punctuation differences, and for longer titles missing or added words.
Roughly, the amount of difference is proportional to the total length of the
strings, so short titles must be a near-exact match, while long titles can have
entire whole words different.

In addition to the title similarity check, only the first author surnames and
year of publication are further checked as heuristics to confirm matches. If I
recall correctly, not even the journal name is is compared. They have done both
performance and correctness evaluation of this process and are happy with the
results, particularly for reference matching.

My (Bryan) commentary on this is that it is probably a good thing to try
implementing for bulk fuzzy candidate generation. I have noticed metadata
issues on Semantic Scholar with similarly titles papers getting grouped, or
fulltext PDF copies getting matched to the wrong paper. I think for fatcat we
should be more conservative, which we can do in our match verification
function.

### Crossref

Crossref has done a fair amount of work on the reference matching problem and
detecting duplicate records. In particular, Dominika Tkaczyk has published a
number of papers and blog posts:

    https://fatcat.wiki/release/search?q=author%3A%22Dominika+Tkaczyk%22
    https://www.crossref.org/authors/dominika-tkaczyk/

Some specific posts:

"Double trouble with DOIs"
<https://www.crossref.org/blog/double-trouble-with-dois/>

"Reference matching: for real this time"
<https://www.crossref.org/blog/reference-matching-for-real-this-time/>

Java implementation and testing/evaluation framework for their reference
matcher:

- <https://gitlab.com/crossref/search_based_reference_matcher>
- <https://gitlab.com/crossref/reference_matching_evaluation_framework>

