"""
    fatcat

    Fatcat is a scalable, versioned, API-oriented catalog of bibliographic entities and file metadata.   # noqa: E501

    The version of the OpenAPI document: 0.5.0
    Contact: webservices@archive.org
    Generated by: https://openapi-generator.tech
"""


import sys
import unittest

import fatcat_openapi_client
from fatcat_openapi_client.model.fileset_file import FilesetFile
from fatcat_openapi_client.model.fileset_url import FilesetUrl
from fatcat_openapi_client.model.release_entity import ReleaseEntity
globals()['FilesetFile'] = FilesetFile
globals()['FilesetUrl'] = FilesetUrl
globals()['ReleaseEntity'] = ReleaseEntity
from fatcat_openapi_client.model.fileset_entity import FilesetEntity


class TestFilesetEntity(unittest.TestCase):
    """FilesetEntity unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def testFilesetEntity(self):
        """Test FilesetEntity"""
        # FIXME: construct object with mandatory attributes with example values
        # model = FilesetEntity()  # noqa: E501
        pass


if __name__ == '__main__':
    unittest.main()
