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
from fatcat_openapi_client.model.editgroup import Editgroup
from fatcat_openapi_client.model.work_entity import WorkEntity
globals()['Editgroup'] = Editgroup
globals()['WorkEntity'] = WorkEntity
from fatcat_openapi_client.model.work_auto_batch import WorkAutoBatch


class TestWorkAutoBatch(unittest.TestCase):
    """WorkAutoBatch unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def testWorkAutoBatch(self):
        """Test WorkAutoBatch"""
        # FIXME: construct object with mandatory attributes with example values
        # model = WorkAutoBatch()  # noqa: E501
        pass


if __name__ == '__main__':
    unittest.main()
