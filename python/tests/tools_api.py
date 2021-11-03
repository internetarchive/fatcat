import pytest
from fatcat_openapi_client.rest import ApiException

from fatcat_tools import authenticated_api, public_api


def test_authenticated_api():
    api = authenticated_api("http://localhost:9411/v0")
    api.get_changelog()
    api.auth_check()


def test_public_api():
    api = public_api("http://localhost:9411/v0")
    api.get_changelog()
    with pytest.raises(ApiException):
        api.auth_check()
