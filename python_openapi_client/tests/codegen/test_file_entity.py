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
from fatcat_openapi_client.model.file_url import FileUrl
from fatcat_openapi_client.model.release_entity import ReleaseEntity
globals()['FileUrl'] = FileUrl
globals()['ReleaseEntity'] = ReleaseEntity
from fatcat_openapi_client.model.file_entity import FileEntity


class TestFileEntity(unittest.TestCase):
    """FileEntity unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def testFileEntity(self):
        """Test FileEntity"""
        # FIXME: construct object with mandatory attributes with example values
        # model = FileEntity()  # noqa: E501
        pass


if __name__ == '__main__':
    unittest.main()
