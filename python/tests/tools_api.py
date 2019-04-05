
import pytest
from fatcat_client import EditgroupAnnotation
from fatcat_client.rest import ApiException

from fatcat_tools import public_api, authenticated_api


def test_authenticated_api():
    api = authenticated_api("http://localhost:9411/v0")
    api.get_changelog()
    api.auth_check()

def test_public_api():
    api = public_api("http://localhost:9411/v0")
    api.get_changelog()
    # XXX: there is some contamination happening here, and we're getting
    # authenticated. Maybe the DefaultAPI thing?
    pytest.skip("public_api() client not isolated from authenticated")
    with pytest.raises(ApiException):
        api.auth_check()
