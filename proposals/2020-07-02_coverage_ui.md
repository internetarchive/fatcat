
status: implemented

Coverage UI/UX Enhancements
===========================

Want to generally enhance the use case of fatcat as a tool for exploring
preservation coverage of groups of papers.

Specific changes:

- make coverage bar graphs and coverage-by-year charts use preservation codes
  instead of the current categories
- container coverage page should have bar coverage by release type (horizontal bars)
- container coverage page: "volume" chart (like years)
- coverage search page: enter release query, show coverage stats/graphs
    => link to "missing releases" search query (TODO)
    => same basic view as container summary page
    => parameter: by year or past 60 days
- show preservation status in release search results (per hit) (TODO)
- high-level coverage summary pages (TODO)
    => published papers since 1900
    => "recent" (last 60 days, by day not year)

Stretch changes:

- update front page with a static (SVG) coverage diagram
- incorporate summaries in container index (new "enhanced container" index?),
  for display in search results (along with total count (TODO)
    => also periodically run a script to update them (daily? weekly?)
    => calculate these at index update time
    => rough stats by type: paper, dataset, doc, etc

Not coverage-specific, but update at the same time:

- show summary of release types on container page (as bar, top 4-5 release types)
- list keepers on container and coverage page (TODO)
    => with some link? KBART, issn.org keepers

## New Views/URLs

### Container Views

`GET /container/<ident>/stats.json`

Existing endpoint updated with new stats:

- preservation aggregation
- release type aggregation

`GET /container/<ident>/preservation_by_year.json`
`GET /container/<ident>/preservation_by_year.svg`

`GET /container/<ident>/preservation_by_volume.json`
`GET /container/<ident>/preservation_by_volume.svg`

`GET /container/<ident>/preservation_by_type.json`

### Coverage

`GET /coverage`: high-level summary (TODO)
`GET /coverage/search`: like `/release/search`, but shows aggregates not hits

## Coverage Logic and Fulltext Display

Current preservation codes (no change):

- `bright` (green): in IA or an open archive like arxiv.org, Pubmed Central
- `dark` (dark green): preserved by a known Keeper, but not in a "bright" archive
- `shadows_only` (grey/red): not in "bright" or "dark" archive, but in a shadow library
- `none` (red): no known preservation

Going to update preservation code logic of releases that have no file in IA
(yet), but do have an arxiv or pubmed central identifier:

- has arxiv id: label as "bright"
    => and show fulltext link to arxiv
- pmcid and more than 12 months old: "bright"
    => and show fulltext link to pmc
- pmcid and less than 12 months old: "dark"
