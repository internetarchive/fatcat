# flake8: noqa

# import all models into this package
# if you have many models here with many references from one model to another this may
# raise a RecursionError
# to avoid this, import only the models that you directly need like:
# from from fatcat_openapi_client.model.pet import Pet
# or import this package, but before doing it, use:
# import sys
# sys.setrecursionlimit(n)

from fatcat_openapi_client.model.auth_oidc import AuthOidc
from fatcat_openapi_client.model.auth_oidc_result import AuthOidcResult
from fatcat_openapi_client.model.auth_token_result import AuthTokenResult
from fatcat_openapi_client.model.changelog_entry import ChangelogEntry
from fatcat_openapi_client.model.container_auto_batch import ContainerAutoBatch
from fatcat_openapi_client.model.container_entity import ContainerEntity
from fatcat_openapi_client.model.creator_auto_batch import CreatorAutoBatch
from fatcat_openapi_client.model.creator_entity import CreatorEntity
from fatcat_openapi_client.model.editgroup import Editgroup
from fatcat_openapi_client.model.editgroup_annotation import EditgroupAnnotation
from fatcat_openapi_client.model.editgroup_edits import EditgroupEdits
from fatcat_openapi_client.model.editor import Editor
from fatcat_openapi_client.model.entity_edit import EntityEdit
from fatcat_openapi_client.model.entity_history_entry import EntityHistoryEntry
from fatcat_openapi_client.model.error_response import ErrorResponse
from fatcat_openapi_client.model.file_auto_batch import FileAutoBatch
from fatcat_openapi_client.model.file_entity import FileEntity
from fatcat_openapi_client.model.file_url import FileUrl
from fatcat_openapi_client.model.fileset_auto_batch import FilesetAutoBatch
from fatcat_openapi_client.model.fileset_entity import FilesetEntity
from fatcat_openapi_client.model.fileset_file import FilesetFile
from fatcat_openapi_client.model.fileset_url import FilesetUrl
from fatcat_openapi_client.model.release_abstract import ReleaseAbstract
from fatcat_openapi_client.model.release_auto_batch import ReleaseAutoBatch
from fatcat_openapi_client.model.release_contrib import ReleaseContrib
from fatcat_openapi_client.model.release_entity import ReleaseEntity
from fatcat_openapi_client.model.release_ext_ids import ReleaseExtIds
from fatcat_openapi_client.model.release_ref import ReleaseRef
from fatcat_openapi_client.model.success import Success
from fatcat_openapi_client.model.webcapture_auto_batch import WebcaptureAutoBatch
from fatcat_openapi_client.model.webcapture_cdx_line import WebcaptureCdxLine
from fatcat_openapi_client.model.webcapture_entity import WebcaptureEntity
from fatcat_openapi_client.model.webcapture_url import WebcaptureUrl
from fatcat_openapi_client.model.work_auto_batch import WorkAutoBatch
from fatcat_openapi_client.model.work_entity import WorkEntity
