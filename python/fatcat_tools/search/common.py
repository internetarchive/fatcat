import sys
from typing import Any, Dict, List, Union

import elasticsearch
import elasticsearch_dsl.response
from elasticsearch_dsl import Search


class FatcatSearchError(Exception):
    def __init__(self, status_code: Union[int, str], name: str, description: str = None):
        if status_code == "TIMEOUT":
            status_code = 504
        elif isinstance(status_code, str):
            try:
                status_code = int(status_code)
            except ValueError:
                status_code = 503
        self.status_code = status_code
        self.name = name
        self.description = description


def _hits_total_int(val: Any) -> int:
    """
    Compatibility hack between ES 6.x and 7.x. In ES 6x, total is returned as
    an int in many places, in ES 7 as a dict (JSON object) with 'value' key
    """
    if isinstance(val, int):
        return val
    else:
        return int(val["value"])


def results_to_dict(response: elasticsearch_dsl.response.Response) -> List[dict]:
    """
    Takes a response returns all the hits as JSON objects.

    Also handles surrogate strings that elasticsearch returns sometimes,
    probably due to mangled data processing in some pipeline. "Crimes against
    Unicode"; production workaround
    """

    results = []
    for h in response:
        r = h._d_
        # print(h.meta._d_)
        results.append(r)

    for h in results:
        for key in h:
            if type(h[key]) is str:
                h[key] = h[key].encode("utf8", "ignore").decode("utf8")
    return results


def wrap_es_execution(search: Search) -> Any:
    """
    Executes a Search object, and converts various ES error types into
    something we can pretty print to the user.
    """
    try:
        resp = search.execute()
    except elasticsearch.exceptions.RequestError as e:
        # this is a "user" error
        print("elasticsearch 400: " + str(e.info), file=sys.stderr)
        description = None
        assert isinstance(e.info, dict)
        if e.info.get("error", {}).get("root_cause", {}):
            description = str(e.info["error"]["root_cause"][0].get("reason"))
        raise FatcatSearchError(e.status_code, str(e.error), description)
    except elasticsearch.exceptions.ConnectionError as e:
        raise FatcatSearchError(e.status_code, "ConnectionError: search engine not available")
    except elasticsearch.exceptions.TransportError as e:
        # all other errors
        print(f"elasticsearch non-200 status code: {e.info}", file=sys.stderr)
        description = None
        assert isinstance(e.info, dict)
        if e.info and e.info.get("error", {}).get("root_cause", {}):
            description = str(e.info["error"]["root_cause"][0].get("reason"))
        raise FatcatSearchError(e.status_code, str(e.error), description)
    return resp


def agg_to_dict(agg: Any) -> Dict[str, Any]:
    """
    Takes a simple term aggregation result (with buckets) and returns a simple
    dict with keys as terms and counts as values. Includes an extra value
    '_other', and by convention aggregations should be written to have "missing"
    values as '_unknown'.
    """
    result = dict()
    for bucket in agg.buckets:
        result[bucket.key] = bucket.doc_count
    if agg.sum_other_doc_count:
        result["_other"] = agg.sum_other_doc_count
    return result
