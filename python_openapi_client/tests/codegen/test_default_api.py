"""
    fatcat

    Fatcat is a scalable, versioned, API-oriented catalog of bibliographic entities and file metadata.   # noqa: E501

    The version of the OpenAPI document: 0.4.0
    Contact: webservices@archive.org
    Generated by: https://openapi-generator.tech
"""



import unittest

import fatcat_openapi_client
from fatcat_openapi_client.api.default_api import DefaultApi  # noqa: E501
from fatcat_openapi_client.rest import ApiException


class TestDefaultApi(unittest.TestCase):
    """DefaultApi unit test stubs"""

    def setUp(self):
        self.api = fatcat_openapi_client.api.default_api.DefaultApi()  # noqa: E501

    def tearDown(self):
        pass

    def test_accept_editgroup(self):
        """Test case for accept_editgroup

        """
        pass

    def test_auth_check(self):
        """Test case for auth_check

        """
        pass

    def test_auth_oidc(self):
        """Test case for auth_oidc

        """
        pass

    def test_create_auth_token(self):
        """Test case for create_auth_token

        """
        pass

    def test_create_container(self):
        """Test case for create_container

        """
        pass

    def test_create_container_auto_batch(self):
        """Test case for create_container_auto_batch

        """
        pass

    def test_create_creator(self):
        """Test case for create_creator

        """
        pass

    def test_create_creator_auto_batch(self):
        """Test case for create_creator_auto_batch

        """
        pass

    def test_create_editgroup(self):
        """Test case for create_editgroup

        """
        pass

    def test_create_editgroup_annotation(self):
        """Test case for create_editgroup_annotation

        """
        pass

    def test_create_file(self):
        """Test case for create_file

        """
        pass

    def test_create_file_auto_batch(self):
        """Test case for create_file_auto_batch

        """
        pass

    def test_create_fileset(self):
        """Test case for create_fileset

        """
        pass

    def test_create_fileset_auto_batch(self):
        """Test case for create_fileset_auto_batch

        """
        pass

    def test_create_release(self):
        """Test case for create_release

        """
        pass

    def test_create_release_auto_batch(self):
        """Test case for create_release_auto_batch

        """
        pass

    def test_create_webcapture(self):
        """Test case for create_webcapture

        """
        pass

    def test_create_webcapture_auto_batch(self):
        """Test case for create_webcapture_auto_batch

        """
        pass

    def test_create_work(self):
        """Test case for create_work

        """
        pass

    def test_create_work_auto_batch(self):
        """Test case for create_work_auto_batch

        """
        pass

    def test_delete_container(self):
        """Test case for delete_container

        """
        pass

    def test_delete_container_edit(self):
        """Test case for delete_container_edit

        """
        pass

    def test_delete_creator(self):
        """Test case for delete_creator

        """
        pass

    def test_delete_creator_edit(self):
        """Test case for delete_creator_edit

        """
        pass

    def test_delete_file(self):
        """Test case for delete_file

        """
        pass

    def test_delete_file_edit(self):
        """Test case for delete_file_edit

        """
        pass

    def test_delete_fileset(self):
        """Test case for delete_fileset

        """
        pass

    def test_delete_fileset_edit(self):
        """Test case for delete_fileset_edit

        """
        pass

    def test_delete_release(self):
        """Test case for delete_release

        """
        pass

    def test_delete_release_edit(self):
        """Test case for delete_release_edit

        """
        pass

    def test_delete_webcapture(self):
        """Test case for delete_webcapture

        """
        pass

    def test_delete_webcapture_edit(self):
        """Test case for delete_webcapture_edit

        """
        pass

    def test_delete_work(self):
        """Test case for delete_work

        """
        pass

    def test_delete_work_edit(self):
        """Test case for delete_work_edit

        """
        pass

    def test_get_changelog(self):
        """Test case for get_changelog

        """
        pass

    def test_get_changelog_entry(self):
        """Test case for get_changelog_entry

        """
        pass

    def test_get_container(self):
        """Test case for get_container

        """
        pass

    def test_get_container_edit(self):
        """Test case for get_container_edit

        """
        pass

    def test_get_container_history(self):
        """Test case for get_container_history

        """
        pass

    def test_get_container_redirects(self):
        """Test case for get_container_redirects

        """
        pass

    def test_get_container_revision(self):
        """Test case for get_container_revision

        """
        pass

    def test_get_creator(self):
        """Test case for get_creator

        """
        pass

    def test_get_creator_edit(self):
        """Test case for get_creator_edit

        """
        pass

    def test_get_creator_history(self):
        """Test case for get_creator_history

        """
        pass

    def test_get_creator_redirects(self):
        """Test case for get_creator_redirects

        """
        pass

    def test_get_creator_releases(self):
        """Test case for get_creator_releases

        """
        pass

    def test_get_creator_revision(self):
        """Test case for get_creator_revision

        """
        pass

    def test_get_editgroup(self):
        """Test case for get_editgroup

        """
        pass

    def test_get_editgroup_annotations(self):
        """Test case for get_editgroup_annotations

        """
        pass

    def test_get_editgroups_reviewable(self):
        """Test case for get_editgroups_reviewable

        """
        pass

    def test_get_editor(self):
        """Test case for get_editor

        """
        pass

    def test_get_editor_annotations(self):
        """Test case for get_editor_annotations

        """
        pass

    def test_get_editor_editgroups(self):
        """Test case for get_editor_editgroups

        """
        pass

    def test_get_file(self):
        """Test case for get_file

        """
        pass

    def test_get_file_edit(self):
        """Test case for get_file_edit

        """
        pass

    def test_get_file_history(self):
        """Test case for get_file_history

        """
        pass

    def test_get_file_redirects(self):
        """Test case for get_file_redirects

        """
        pass

    def test_get_file_revision(self):
        """Test case for get_file_revision

        """
        pass

    def test_get_fileset(self):
        """Test case for get_fileset

        """
        pass

    def test_get_fileset_edit(self):
        """Test case for get_fileset_edit

        """
        pass

    def test_get_fileset_history(self):
        """Test case for get_fileset_history

        """
        pass

    def test_get_fileset_redirects(self):
        """Test case for get_fileset_redirects

        """
        pass

    def test_get_fileset_revision(self):
        """Test case for get_fileset_revision

        """
        pass

    def test_get_release(self):
        """Test case for get_release

        """
        pass

    def test_get_release_edit(self):
        """Test case for get_release_edit

        """
        pass

    def test_get_release_files(self):
        """Test case for get_release_files

        """
        pass

    def test_get_release_filesets(self):
        """Test case for get_release_filesets

        """
        pass

    def test_get_release_history(self):
        """Test case for get_release_history

        """
        pass

    def test_get_release_redirects(self):
        """Test case for get_release_redirects

        """
        pass

    def test_get_release_revision(self):
        """Test case for get_release_revision

        """
        pass

    def test_get_release_webcaptures(self):
        """Test case for get_release_webcaptures

        """
        pass

    def test_get_webcapture(self):
        """Test case for get_webcapture

        """
        pass

    def test_get_webcapture_edit(self):
        """Test case for get_webcapture_edit

        """
        pass

    def test_get_webcapture_history(self):
        """Test case for get_webcapture_history

        """
        pass

    def test_get_webcapture_redirects(self):
        """Test case for get_webcapture_redirects

        """
        pass

    def test_get_webcapture_revision(self):
        """Test case for get_webcapture_revision

        """
        pass

    def test_get_work(self):
        """Test case for get_work

        """
        pass

    def test_get_work_edit(self):
        """Test case for get_work_edit

        """
        pass

    def test_get_work_history(self):
        """Test case for get_work_history

        """
        pass

    def test_get_work_redirects(self):
        """Test case for get_work_redirects

        """
        pass

    def test_get_work_releases(self):
        """Test case for get_work_releases

        """
        pass

    def test_get_work_revision(self):
        """Test case for get_work_revision

        """
        pass

    def test_lookup_container(self):
        """Test case for lookup_container

        """
        pass

    def test_lookup_creator(self):
        """Test case for lookup_creator

        """
        pass

    def test_lookup_editor(self):
        """Test case for lookup_editor

        """
        pass

    def test_lookup_file(self):
        """Test case for lookup_file

        """
        pass

    def test_lookup_release(self):
        """Test case for lookup_release

        """
        pass

    def test_update_container(self):
        """Test case for update_container

        """
        pass

    def test_update_creator(self):
        """Test case for update_creator

        """
        pass

    def test_update_editgroup(self):
        """Test case for update_editgroup

        """
        pass

    def test_update_editor(self):
        """Test case for update_editor

        """
        pass

    def test_update_file(self):
        """Test case for update_file

        """
        pass

    def test_update_fileset(self):
        """Test case for update_fileset

        """
        pass

    def test_update_release(self):
        """Test case for update_release

        """
        pass

    def test_update_webcapture(self):
        """Test case for update_webcapture

        """
        pass

    def test_update_work(self):
        """Test case for update_work

        """
        pass


if __name__ == '__main__':
    unittest.main()
