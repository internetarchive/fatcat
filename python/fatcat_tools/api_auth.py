import os
import sys
from typing import Optional

from fatcat_openapi_client import ApiClient, Configuration, DefaultApi


def public_api(host_uri: str) -> DefaultApi:
    """
    Note: unlike the authenticated variant, this helper might get called even
    if the API isn't going to be used, so it's important that it doesn't try to
    actually connect to the API host or something.
    """
    conf = Configuration()
    conf.host = host_uri
    return DefaultApi(ApiClient(conf))


def authenticated_api(host_uri: str, token: Optional[str] = None) -> DefaultApi:
    """
    Note: if this helper is called, it's implied that an actual API connection
    is needed, so it does try to connect and verify credentials.
    """

    conf = Configuration()
    conf.host = host_uri
    if not token:
        token = os.environ["FATCAT_API_AUTH_TOKEN"]
    if not token:
        sys.stderr.write(
            "This client requires a fatcat API token (eg, in env var FATCAT_API_AUTH_TOKEN)\n"
        )
        sys.exit(-1)

    conf.api_key["Authorization"] = token
    conf.api_key_prefix["Authorization"] = "Bearer"
    api = DefaultApi(ApiClient(conf))

    # verify up front that auth is working
    api.auth_check()

    return api
