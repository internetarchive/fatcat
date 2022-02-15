"""
Helpers for doing elasticsearch queries (used in the web interface; not part of
the formal API)
"""

import datetime
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple

import elasticsearch
from elasticsearch_dsl import Q, Search

from fatcat_tools.search.common import (
    _hits_total_int,
    agg_to_dict,
    results_to_dict,
    wrap_es_execution,
)
from fatcat_tools.search.stats import query_es_container_stats
from fatcat_web import app


@dataclass
class ReleaseQuery:
    q: Optional[str] = None
    limit: Optional[int] = None
    offset: Optional[int] = None
    fulltext_only: bool = False
    container_id: Optional[str] = None
    recent: bool = False
    exclude_stubs: bool = False
    sort: Optional[List[str]] = None

    @staticmethod
    def from_args(args: Dict[str, Any]) -> "ReleaseQuery":

        query_str = args.get("q") or "*"

        offset = args.get("offset", "0")
        offset = max(0, int(offset)) if offset.isnumeric() else 0

        return ReleaseQuery(
            q=query_str,
            offset=offset,
            fulltext_only=bool(args.get("fulltext_only")),
            container_id=args.get("container_id"),
            recent=bool(args.get("recent")),
            exclude_stubs=bool(args.get("exclude_stubs")),
            sort=None,
        )


@dataclass
class GenericQuery:
    q: Optional[str] = None
    limit: Optional[int] = None
    offset: Optional[int] = None

    @staticmethod
    def from_args(args: Dict[str, Any]) -> "GenericQuery":
        query_str = args.get("q")
        if not query_str:
            query_str = "*"
        offset = args.get("offset", "0")
        offset = max(0, int(offset)) if offset.isnumeric() else 0

        return GenericQuery(
            q=query_str,
            offset=offset,
        )


@dataclass
class SearchHits:
    count_returned: int
    count_found: int
    offset: int
    limit: int
    deep_page_limit: int
    query_time_ms: int
    results: List[Any]


def do_container_search(query: GenericQuery, deep_page_limit: int = 2000) -> SearchHits:

    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_CONTAINER_INDEX"])

    basic_query = Q(
        "query_string",
        query=query.q,
        default_operator="AND",
        analyze_wildcard=True,
        allow_leading_wildcard=False,
        lenient=True,
        fields=["biblio"],
    )

    search = search.query(
        "boosting",
        positive=Q(
            "bool",
            must=basic_query,
            should=[
                Q("range", releases_total={"gte": 500}),
                Q("range", releases_total={"gte": 5000}),
            ],
        ),
        negative=Q("term", releases_total=0),
        negative_boost=0.5,
    )

    # Sanity checks
    limit = min((int(query.limit or 25), 100))
    offset = max((int(query.offset or 0), 0))
    if offset > deep_page_limit:
        # Avoid deep paging problem.
        offset = deep_page_limit

    search = search[offset : (offset + limit)]
    search = search.params(track_total_hits=True)

    resp = wrap_es_execution(search)
    results = results_to_dict(resp)

    return SearchHits(
        count_returned=len(results),
        count_found=_hits_total_int(resp.hits.total),
        offset=offset,
        limit=limit,
        deep_page_limit=deep_page_limit,
        query_time_ms=int(resp.took),
        results=results,
    )


def do_release_search(query: ReleaseQuery, deep_page_limit: int = 2000) -> SearchHits:

    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])

    # availability filters
    if query.fulltext_only:
        search = search.filter("term", in_ia=True)

    # Below, we combine several queries to improve scoring.

    # this query use the fancy built-in query string parser
    basic_biblio = Q(
        "query_string",
        query=query.q,
        default_operator="AND",
        analyze_wildcard=True,
        allow_leading_wildcard=False,
        lenient=True,
        fields=[
            "title^2",
            "biblio",
        ],
    )
    has_fulltext = Q("term", in_ia=True)
    poor_metadata = Q(
        "bool",
        should=[
            # if these fields aren't set, metadata is poor. The more that do
            # not exist, the stronger the signal.
            Q("bool", must_not=Q("exists", field="title")),
            Q("bool", must_not=Q("exists", field="release_year")),
            Q("bool", must_not=Q("exists", field="release_type")),
            Q("bool", must_not=Q("exists", field="release_stage")),
            Q("bool", must_not=Q("exists", field="container_id")),
        ],
    )

    if query.container_id:
        search = search.filter("term", container_id=query.container_id)

    search = search.query(
        "boosting",
        positive=Q(
            "bool",
            must=basic_biblio,
            should=[has_fulltext],
        ),
        negative=poor_metadata,
        negative_boost=0.5,
    )

    if query.sort:
        search = search.sort(*query.sort)

    # Sanity checks
    limit = min((int(query.limit or 25), 100))
    offset = max((int(query.offset or 0), 0))
    if offset > deep_page_limit:
        # Avoid deep paging problem.
        offset = deep_page_limit

    search = search[offset : (offset + limit)]
    search = search.params(track_total_hits=True)

    resp = wrap_es_execution(search)
    results = results_to_dict(resp)

    for h in results:
        # Ensure 'contrib_names' is a list, not a single string
        if type(h["contrib_names"]) is not list:
            h["contrib_names"] = [
                h["contrib_names"],
            ]
        h["contrib_names"] = [
            name.encode("utf8", "ignore").decode("utf8") for name in h["contrib_names"]
        ]

    return SearchHits(
        count_returned=len(results),
        count_found=_hits_total_int(resp.hits.total),
        offset=offset,
        limit=limit,
        deep_page_limit=deep_page_limit,
        query_time_ms=int(resp.took),
        results=results,
    )


def get_elastic_container_random_releases(ident: str, limit: int = 5) -> List[Dict[str, Any]]:
    """
    Returns a list of releases from the container.
    """

    assert limit > 0 and limit <= 100

    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    search = search.query(
        "bool",
        must=[
            Q("term", container_id=ident),
            Q("range", release_year={"lte": datetime.datetime.today().year}),
        ],
    )
    search = search.sort("-in_web", "-release_date")
    search = search[: int(limit)]

    search = search.params(request_cache=True)
    # not needed: search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)
    results = results_to_dict(resp)

    return results


def _sort_vol_key(val: Optional[str]) -> Tuple[bool, bool, int, str]:
    """
    Helper for sorting volume and issue strings. Defined order is:

    - None values first
    - any non-integers next, in non-integer order
    - any integers next, in integer sorted order (ascending)

    Note that the actual sort used/displayed is reversed
    """
    if val is None:
        return (False, False, 0, "")
    if val.isdigit():
        return (True, True, int(val), "")
    else:
        return (True, False, 0, val)


def get_elastic_container_browse_year_volume_issue(ident: str) -> List[Dict[str, Any]]:
    """
    Returns a set of histogram buckets, as nested dicts/lists:

        [
          { year: int,
            volumes: [
              { volume: str|None
                issues: [
                  { issue: str|None
                    count: int
                  }
                ] }
            ] }
        ]
    """

    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    search = search.query(
        "bool",
        filter=[Q("bool", must_not=[Q("match", release_type="stub")])],
    )
    search = search.filter("term", container_id=ident)
    search.aggs.bucket(
        "year_volume",
        "composite",
        size=1500,
        sources=[
            {
                "year": {
                    "histogram": {
                        "field": "release_year",
                        "interval": 1,
                        "missing_bucket": True,
                    },
                }
            },
            {
                "volume": {
                    "terms": {
                        "field": "volume",
                        "missing_bucket": True,
                    },
                }
            },
            {
                "issue": {
                    "terms": {
                        "field": "issue",
                        "missing_bucket": True,
                    },
                }
            },
        ],
    )
    search = search[:0]
    search = search.params(request_cache=True)
    resp = wrap_es_execution(search)
    buckets = resp.aggregations.year_volume.buckets
    # print(buckets)
    buckets = [h for h in buckets if h["key"]["year"]]
    year_nums = set([int(h["key"]["year"]) for h in buckets])
    year_dicts: Dict[int, Dict[str, Any]] = dict()
    if year_nums:
        for year in year_nums:
            year_dicts[year] = {}
        for row in buckets:
            year = int(row["key"]["year"])
            volume = row["key"]["volume"] or ""
            issue = row["key"]["issue"] or ""
            if volume not in year_dicts[year]:
                year_dicts[year][volume] = {}
            year_dicts[year][volume][issue] = int(row["doc_count"])

    # transform to lists-of-dicts
    year_list = []
    for year in year_dicts.keys():
        volume_list = []
        for volume in year_dicts[year].keys():
            issue_list = []
            for issue in year_dicts[year][volume].keys():
                issue_list.append(
                    dict(issue=issue or None, count=year_dicts[year][volume][issue])
                )
            issue_list = sorted(
                issue_list, key=lambda x: _sort_vol_key(x["issue"]), reverse=True
            )
            volume_list.append(dict(volume=volume or None, issues=issue_list))
        volume_list = sorted(
            volume_list, key=lambda x: _sort_vol_key(x["volume"]), reverse=True
        )
        year_list.append(dict(year=year, volumes=volume_list))
    return sorted(year_list, key=lambda x: x["year"], reverse=True)


def get_elastic_entity_stats() -> dict:
    """
    TODO: files, filesets, webcaptures (no schema yet)

    Returns dict:
        changelog: {latest: {index, datetime}}
        release: {total, refs_total}
        papers: {total, in_web, in_oa, in_kbart, in_web_not_kbart}
    """

    stats = {}

    # release totals
    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    search.aggs.bucket(
        "release_ref_count",
        "sum",
        field="ref_count",
    )
    search = search[:0]  # pylint: disable=unsubscriptable-object

    search = search.params(request_cache=True)
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)

    stats["release"] = {
        "total": _hits_total_int(resp.hits.total),
        "refs_total": int(resp.aggregations.release_ref_count.value),
    }

    # paper counts
    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    search = search.query(
        "terms",
        release_type=[
            "article-journal",
            "paper-conference",
            # "chapter",
            # "thesis",
        ],
    )
    search.aggs.bucket(
        "paper_like",
        "filters",
        filters={
            "in_web": {"term": {"in_web": "true"}},
            "is_oa": {"term": {"is_oa": "true"}},
            "in_kbart": {"term": {"in_kbart": "true"}},
            "in_web_not_kbart": {
                "bool": {
                    "filter": [
                        {"term": {"in_web": "true"}},
                        {"term": {"in_kbart": "false"}},
                    ]
                }
            },
        },
    )
    search = search[:0]

    search = search.params(request_cache=True)
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)
    buckets = resp.aggregations.paper_like.buckets
    stats["papers"] = {
        "total": _hits_total_int(resp.hits.total),
        "in_web": buckets.in_web.doc_count,
        "is_oa": buckets.is_oa.doc_count,
        "in_kbart": buckets.in_kbart.doc_count,
        "in_web_not_kbart": buckets.in_web_not_kbart.doc_count,
    }

    # container counts
    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_CONTAINER_INDEX"])
    search.aggs.bucket(
        "release_ref_count",
        "sum",
        field="ref_count",
    )
    search = search[:0]  # pylint: disable=unsubscriptable-object

    search = search.params(request_cache=True)
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)
    stats["container"] = {
        "total": _hits_total_int(resp.hits.total),
    }

    return stats


def get_elastic_search_coverage(query: ReleaseQuery) -> dict:

    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    search = search.query(
        "query_string",
        query=query.q,
        default_operator="AND",
        analyze_wildcard=True,
        allow_leading_wildcard=False,
        lenient=True,
        fields=["biblio"],
    )
    search.aggs.bucket(
        "preservation",
        "terms",
        field="preservation",
        missing="_unknown",
    )
    if query.recent:
        date_today = datetime.date.today()
        start_date = str(date_today - datetime.timedelta(days=60))
        end_date = str(date_today + datetime.timedelta(days=1))
        search = search.filter("range", release_date=dict(gte=start_date, lte=end_date))

    search = search[:0]

    search = search.params(request_cache=True)
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)

    preservation_bucket = agg_to_dict(resp.aggregations.preservation)
    preservation_bucket["total"] = _hits_total_int(resp.hits.total)
    for k in ("bright", "dark", "shadows_only", "none"):
        if k not in preservation_bucket:
            preservation_bucket[k] = 0
    if app.config["FATCAT_MERGE_SHADOW_PRESERVATION"]:
        preservation_bucket["none"] += preservation_bucket["shadows_only"]
        preservation_bucket["shadows_only"] = 0
    stats = {
        "total": _hits_total_int(resp.hits.total),
        "preservation": preservation_bucket,
    }

    return stats


def get_elastic_container_stats(
    ident: str,
    issnl: Optional[str] = None,
    es_client: Optional[elasticsearch.Elasticsearch] = None,
    es_index: Optional[str] = None,
    merge_shadows: Optional[bool] = None,
) -> Dict[str, Any]:
    """
    This is a DEPRECATED backwards-compatability wrapper around the new
    query_es_container_stats() method from fatcat_tools.

    Returns dict:
        ident
        issnl (optional)
        total: count
        in_web: count
        in_kbart: count
        is_preserved: count
        preservation{}
            "histogram" by preservation status
        release_type{}
            "histogram" by release type
    """

    if not es_client:
        es_client = app.es_client
    if not es_index:
        es_index = app.config["ELASTICSEARCH_RELEASE_INDEX"]
    if merge_shadows is None:
        merge_shadows = app.config["FATCAT_MERGE_SHADOW_PRESERVATION"]

    stats = query_es_container_stats(
        ident=ident,
        es_client=es_client,
        es_index=es_index,
        merge_shadows=merge_shadows,
    )
    stats["issnl"] = issnl
    return stats


def get_elastic_container_histogram_legacy(ident: str) -> List[Tuple[int, bool, int]]:
    """
    Fetches a stacked histogram of {year, in_ia}. This is for the older style
    of coverage graph (SVG or JSON export). This function should be DEPRECATED
    to be removed in the near future.

    Filters to the past 500 years (at most), or about 1000 values.

    Returns a list of tuples:
        (year, in_ia, count)
    """

    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    search = search.query(
        "bool",
        must=[
            Q(
                "range",
                release_year={
                    "gte": datetime.datetime.today().year - 499,
                    "lte": datetime.datetime.today().year,
                },
            ),
        ],
        filter=[
            Q(
                "bool",
                minimum_should_match=1,
                should=[
                    Q("match", container_id=ident),
                ],
            ),
        ],
    )
    search.aggs.bucket(
        "year_in_ia",
        "composite",
        size=1000,
        sources=[
            {
                "year": {
                    "histogram": {
                        "field": "release_year",
                        "interval": 1,
                    },
                }
            },
            {
                "in_ia": {
                    "terms": {
                        "field": "in_ia",
                    },
                }
            },
        ],
    )
    search = search[:0]

    search = search.params(request_cache="true")
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)

    buckets = resp.aggregations.year_in_ia.buckets
    vals = [(int(h["key"]["year"]), h["key"]["in_ia"], h["doc_count"]) for h in buckets]
    vals = sorted(vals)
    return vals


def get_elastic_preservation_by_year(query: ReleaseQuery) -> List[Dict[str, Any]]:
    """
    Fetches a stacked histogram of {year, preservation}.

    Preservation has 4 potential values; this function filters to the past 250
    years (at most), or about 1000 values.

    Returns a list of dicts, sorted by year, with keys/values like:

        {year (int), bright (int), dark (int), shadows_only (int), none (int)}

    Stubs can be excluded by setting the appropriate query flag
    """

    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    if query.q not in [None, "*"]:
        search = search.query(
            "query_string",
            query=query.q,
            default_operator="AND",
            analyze_wildcard=True,
            allow_leading_wildcard=False,
            lenient=True,
            fields=[
                "biblio",
            ],
        )
    search = search.filter("term", container_id=query.container_id)
    if query.exclude_stubs:
        search = search.query(
            "bool",
            filter=[
                Q(
                    "bool",
                    must_not=[
                        Q("match", release_type="stub"),
                    ],
                ),
            ],
        )
    search = search.filter(
        "range",
        release_year={
            "gte": datetime.datetime.today().year - 249,
            "lte": datetime.datetime.today().year,
        },
    )

    search.aggs.bucket(
        "year_preservation",
        "composite",
        size=1500,
        sources=[
            {
                "year": {
                    "histogram": {
                        "field": "release_year",
                        "interval": 1,
                    },
                }
            },
            {
                "preservation": {
                    "terms": {
                        "field": "preservation",
                    },
                }
            },
        ],
    )
    search = search[:0]
    search = search.params(request_cache="true")
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)

    buckets = resp.aggregations.year_preservation.buckets
    year_nums = set([int(h["key"]["year"]) for h in buckets])
    year_dicts = dict()
    if year_nums:
        for num in range(min(year_nums), max(year_nums) + 1):
            year_dicts[num] = dict(year=num, bright=0, dark=0, shadows_only=0, none=0)
        for row in buckets:
            year_dicts[int(row["key"]["year"])][row["key"]["preservation"]] = int(
                row["doc_count"]
            )
    if app.config["FATCAT_MERGE_SHADOW_PRESERVATION"]:
        for k in year_dicts.keys():
            year_dicts[k]["none"] += year_dicts[k]["shadows_only"]
            year_dicts[k]["shadows_only"] = 0
    return sorted(year_dicts.values(), key=lambda x: x["year"])


def get_elastic_preservation_by_date(query: ReleaseQuery) -> List[dict]:
    """
    Fetches a stacked histogram of {date, preservation}.

    Preservation has 4 potential values; this function filters to the past 250
    years (at most), or about 1000 values.

    Returns a list of dicts, sorted by date, with keys/values like:

        {date (str), bright (int), dark (int), shadows_only (int), none (int)}
    """

    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    if query.q not in [None, "*"]:
        search = search.query(
            "query_string",
            query=query.q,
            default_operator="AND",
            analyze_wildcard=True,
            allow_leading_wildcard=False,
            lenient=True,
            fields=[
                "biblio",
            ],
        )
    if query.container_id:
        search = search.filter(
            "term",
            container_id=query.container_id,
        )
    date_today = datetime.date.today()
    start_date = date_today - datetime.timedelta(days=60)
    end_date = date_today + datetime.timedelta(days=1)
    search = search.filter(
        "range",
        release_date=dict(
            gte=str(start_date),
            lte=str(end_date),
        ),
    )

    search.aggs.bucket(
        "date_preservation",
        "composite",
        size=1500,
        sources=[
            {
                "date": {
                    "histogram": {
                        "field": "release_date",
                        "interval": 1,
                    },
                }
            },
            {
                "preservation": {
                    "terms": {
                        "field": "preservation",
                    },
                }
            },
        ],
    )
    search = search[:0]
    search = search.params(request_cache="true")
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)

    buckets = resp.aggregations.date_preservation.buckets
    date_dicts: Dict[str, Dict[str, Any]] = dict()
    this_date = start_date
    while this_date <= end_date:
        date_dicts[str(this_date)] = dict(
            date=str(this_date), bright=0, dark=0, shadows_only=0, none=0
        )
        this_date = this_date + datetime.timedelta(days=1)
    for row in buckets:
        date_dicts[row["key"]["date"][0:10]][row["key"]["preservation"]] = int(row["doc_count"])
    if app.config["FATCAT_MERGE_SHADOW_PRESERVATION"]:
        for k in date_dicts.keys():
            date_dicts[k]["none"] += date_dicts[k]["shadows_only"]
            date_dicts[k]["shadows_only"] = 0
    return sorted(date_dicts.values(), key=lambda x: x["date"])


def get_elastic_container_preservation_by_volume(query: ReleaseQuery) -> List[dict]:
    """
    Fetches a stacked histogram of {volume, preservation}.

    Currently only includes volume numbers which are simple integers (all chars
    are digits).

    Returns a list of dicts, sorted by volume, with keys/values like:

        {year (int), bright (int), dark (int), shadows_only (int), none (int)}

    Stubs can be excluded by setting the appropriate query flag
    """

    assert query.container_id is not None
    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    search = search.query(
        "bool",
        filter=[
            Q(
                "bool",
                must=[
                    Q("match", container_id=query.container_id),
                    Q("exists", field="volume"),
                ],
            ),
        ],
    )
    if query.exclude_stubs:
        search = search.query(
            "bool",
            filter=[
                Q(
                    "bool",
                    must_not=[
                        Q("match", release_type="stub"),
                    ],
                ),
            ],
        )

    search.aggs.bucket(
        "volume_preservation",
        "composite",
        size=1500,
        sources=[
            {
                "volume": {
                    "terms": {
                        "field": "volume",
                    },
                }
            },
            {
                "preservation": {
                    "terms": {
                        "field": "preservation",
                    },
                }
            },
        ],
    )
    search = search[:0]
    search = search.params(request_cache="true")
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)

    buckets = resp.aggregations.volume_preservation.buckets
    volume_nums = set(
        [int(h["key"]["volume"]) for h in buckets if h["key"]["volume"].isdigit()]
    )
    volume_dicts = dict()
    if volume_nums:
        if max(volume_nums) - min(volume_nums) > 500:
            raise Exception("too many volume histogram buckets")
        for num in range(min(volume_nums), max(volume_nums) + 1):
            volume_dicts[num] = dict(volume=num, bright=0, dark=0, shadows_only=0, none=0)
        for row in buckets:
            if row["key"]["volume"].isdigit():
                volume_dicts[int(row["key"]["volume"])][row["key"]["preservation"]] = int(
                    row["doc_count"]
                )
    if app.config["FATCAT_MERGE_SHADOW_PRESERVATION"]:
        for k in volume_dicts.keys():
            volume_dicts[k]["none"] += volume_dicts[k]["shadows_only"]
            volume_dicts[k]["shadows_only"] = 0
    return sorted(volume_dicts.values(), key=lambda x: x["volume"])


def get_elastic_preservation_by_type(query: ReleaseQuery) -> List[dict]:
    """
    Fetches preservation coverage by release type

    Returns a list of dicts, sorted by total count, with keys/values like:

        {year (int), bright (int), dark (int), shadows_only (int), none (int)}
    """

    search = Search(using=app.es_client, index=app.config["ELASTICSEARCH_RELEASE_INDEX"])
    if query.q not in [None, "*"]:
        search = search.query(
            "query_string",
            query=query.q,
            default_operator="AND",
            analyze_wildcard=True,
            allow_leading_wildcard=False,
            lenient=True,
            fields=[
                "biblio",
            ],
        )
    if query.container_id:
        search = search.filter("term", container_id=query.container_id)
    if query.recent:
        date_today = datetime.date.today()
        start_date = str(date_today - datetime.timedelta(days=60))
        end_date = str(date_today + datetime.timedelta(days=1))
        search = search.filter("range", release_date=dict(gte=start_date, lte=end_date))
    search.aggs.bucket(
        "type_preservation",
        "composite",
        size=1500,
        sources=[
            {
                "release_type": {
                    "terms": {
                        "field": "release_type",
                    },
                }
            },
            {
                "preservation": {
                    "terms": {
                        "field": "preservation",
                    },
                }
            },
        ],
    )
    search = search[:0]
    search = search.params(request_cache="true")
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)

    buckets = resp.aggregations.type_preservation.buckets
    type_set = set([h["key"]["release_type"] for h in buckets])
    type_dicts = dict()
    for k in type_set:
        type_dicts[k] = dict(release_type=k, bright=0, dark=0, shadows_only=0, none=0, total=0)
    for row in buckets:
        type_dicts[row["key"]["release_type"]][row["key"]["preservation"]] = int(
            row["doc_count"]
        )
    for k in type_set:
        for p in ("bright", "dark", "shadows_only", "none"):
            type_dicts[k]["total"] += type_dicts[k][p]
    if app.config["FATCAT_MERGE_SHADOW_PRESERVATION"]:
        for k in type_set:
            type_dicts[k]["none"] += type_dicts[k]["shadows_only"]
            type_dicts[k]["shadows_only"] = 0
    return sorted(type_dicts.values(), key=lambda x: x["total"], reverse=True)
