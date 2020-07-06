
import json
import collections
from fatcat_openapi_client import ApiClient

def entity_to_dict(entity, api_client=None):
    """
    Hack to take advantage of the code-generated serialization code.

    Initializing/destroying ApiClient objects is surprisingly expensive
    (because it involves a threadpool), so we allow passing an existing
    instance. If you already have a full-on API connection `api`, you can
    access the ApiClient object as `api.api_client`. This is such a speed-up
    that this argument may become mandatory.
    """
    if not api_client:
        api_client = ApiClient()
    return api_client.sanitize_for_serialization(entity)

def entity_from_json(json_str, entity_type, api_client=None):
    """
    Hack to take advantage of the code-generated deserialization code

    See note on `entity_to_dict()` about api_client argument.
    """
    if not api_client:
        api_client = ApiClient()
    thing = collections.namedtuple('Thing', ['data'])
    thing.data = json_str
    return api_client.deserialize(thing, entity_type)

def entity_from_dict(obj, entity_type, api_client=None):
    json_str = json.dumps(obj)
    return entity_from_json(json_str, entity_type, api_client=api_client)
