
status: work-in-progress

The default "release" search on fatcat.wiki currently uses the elasticsearch
built-in `query_string` parser, which is explicitly not recommended for
public/production use.

The best way forward is likely a custom query parser (eg, PEG-generated parser)
that generates a complete elasticsearch query JSON structure.

A couple search issues this would help with:

- better parsing of keywords (year, year-range, DOI, ISSN, etc) in complex
  queries and turning these in to keyword term sub-queries
- queries including terms from multiple fields which aren't explicitly tagged
  (eg, "lovelace computer" vs. "author:lovelace title:computer")
- avoiding unsustainably expensive queries (eg, prefix wildcard, regex)
- handling single-character mispellings and synonyms
- collapsing multiple releases under the same work in search results

In the near future, we may also create a fulltext search index, which will have
it's own issues.
