
# flake8: noqa

# Import all APIs into this package.
# If you have many APIs here with many many models used in each API this may
# raise a `RecursionError`.
# In order to avoid this, import only the API that you directly need like:
#
#   from fatcat_openapi_client.api.auth_api import AuthApi
#
# or import this package, but before doing it, use:
#
#   import sys
#   sys.setrecursionlimit(n)

# Import APIs into API package:
from fatcat_openapi_client.api.auth_api import AuthApi
from fatcat_openapi_client.api.changelog_api import ChangelogApi
from fatcat_openapi_client.api.containers_api import ContainersApi
from fatcat_openapi_client.api.creators_api import CreatorsApi
from fatcat_openapi_client.api.editgroups_api import EditgroupsApi
from fatcat_openapi_client.api.editors_api import EditorsApi
from fatcat_openapi_client.api.files_api import FilesApi
from fatcat_openapi_client.api.filesets_api import FilesetsApi
from fatcat_openapi_client.api.releases_api import ReleasesApi
from fatcat_openapi_client.api.webcaptures_api import WebcapturesApi
from fatcat_openapi_client.api.works_api import WorksApi
