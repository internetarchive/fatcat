from typing import Any, Dict

import elasticsearch
from elasticsearch_dsl import Search

from fatcat_tools.search.common import _hits_total_int, agg_to_dict, wrap_es_execution


def query_es_container_stats(
    ident: str,
    es_client: elasticsearch.Elasticsearch,
    es_index: str = "fatcat_release",
    merge_shadows: bool = False,
) -> Dict[str, Any]:
    """
    Returns dict:
        ident
        total: count
        in_web: count
        in_kbart: count
        is_preserved: count
        preservation{}
            "histogram" by preservation status
        release_type{}
            "histogram" by release type
    """

    search = Search(using=es_client, index=es_index)
    search = search.query(
        "term",
        container_id=ident,
    )
    search.aggs.bucket(
        "container_stats",
        "filters",
        filters={
            "in_web": {
                "term": {"in_web": True},
            },
            "in_kbart": {
                "term": {"in_kbart": True},
            },
            "is_preserved": {
                "term": {"is_preserved": True},
            },
        },
    )
    search.aggs.bucket(
        "preservation",
        "terms",
        field="preservation",
        missing="_unknown",
    )
    search.aggs.bucket(
        "release_type",
        "terms",
        field="release_type",
        missing="_unknown",
    )

    search = search[:0]

    search = search.params(request_cache=True)
    search = search.params(track_total_hits=True)
    resp = wrap_es_execution(search)

    container_stats = resp.aggregations.container_stats.buckets
    preservation_bucket = agg_to_dict(resp.aggregations.preservation)
    preservation_bucket["total"] = _hits_total_int(resp.hits.total)
    for k in ("bright", "dark", "shadows_only", "none"):
        if k not in preservation_bucket:
            preservation_bucket[k] = 0
    if merge_shadows:
        preservation_bucket["none"] += preservation_bucket["shadows_only"]
        preservation_bucket["shadows_only"] = 0
    release_type_bucket = agg_to_dict(resp.aggregations.release_type)
    stats = {
        "ident": ident,
        "total": _hits_total_int(resp.hits.total),
        "in_web": container_stats["in_web"]["doc_count"],
        "in_kbart": container_stats["in_kbart"]["doc_count"],
        "is_preserved": container_stats["is_preserved"]["doc_count"],
        "preservation": preservation_bucket,
        "release_type": release_type_bucket,
    }

    return stats
