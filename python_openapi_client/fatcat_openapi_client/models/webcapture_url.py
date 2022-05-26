# coding: utf-8

"""
    fatcat

    Fatcat is a scalable, versioned, API-oriented catalog of bibliographic entities and file metadata.   # noqa: E501

    The version of the OpenAPI document: 0.3.1
    Contact: webservices@archive.org
    Generated by: https://openapi-generator.tech
"""


import pprint
import re  # noqa: F401

import six


class WebcaptureUrl(object):
    """NOTE: This class is auto generated by OpenAPI Generator.
    Ref: https://openapi-generator.tech

    Do not edit the class manually.
    """

    """
    Attributes:
      openapi_types (dict): The key is attribute name
                            and the value is attribute type.
      attribute_map (dict): The key is attribute name
                            and the value is json key in definition.
    """
    openapi_types = {
        'url': 'str',
        'rel': 'str'
    }

    attribute_map = {
        'url': 'url',
        'rel': 'rel'
    }

    def __init__(self, url=None, rel=None):  # noqa: E501
        """WebcaptureUrl - a model defined in OpenAPI"""  # noqa: E501

        self._url = None
        self._rel = None
        self.discriminator = None

        self.url = url
        self.rel = rel

    @property
    def url(self):
        """Gets the url of this WebcaptureUrl.  # noqa: E501

        URL/URI pointing to archive of this web resource.   # noqa: E501

        :return: The url of this WebcaptureUrl.  # noqa: E501
        :rtype: str
        """
        return self._url

    @url.setter
    def url(self, url):
        """Sets the url of this WebcaptureUrl.

        URL/URI pointing to archive of this web resource.   # noqa: E501

        :param url: The url of this WebcaptureUrl.  # noqa: E501
        :type: str
        """
        if url is None:
            raise ValueError("Invalid value for `url`, must not be `None`")  # noqa: E501

        self._url = url

    @property
    def rel(self):
        """Gets the rel of this WebcaptureUrl.  # noqa: E501

        Type of archive endpoint. Usually `wayback` (WBM replay of primary resource), or `warc` (direct URL to a WARC file containing all resources of the capture). See guide for full list.   # noqa: E501

        :return: The rel of this WebcaptureUrl.  # noqa: E501
        :rtype: str
        """
        return self._rel

    @rel.setter
    def rel(self, rel):
        """Sets the rel of this WebcaptureUrl.

        Type of archive endpoint. Usually `wayback` (WBM replay of primary resource), or `warc` (direct URL to a WARC file containing all resources of the capture). See guide for full list.   # noqa: E501

        :param rel: The rel of this WebcaptureUrl.  # noqa: E501
        :type: str
        """
        if rel is None:
            raise ValueError("Invalid value for `rel`, must not be `None`")  # noqa: E501

        self._rel = rel

    def to_dict(self):
        """Returns the model properties as a dict"""
        result = {}

        for attr, _ in six.iteritems(self.openapi_types):
            value = getattr(self, attr)
            if isinstance(value, list):
                result[attr] = list(map(
                    lambda x: x.to_dict() if hasattr(x, "to_dict") else x,
                    value
                ))
            elif hasattr(value, "to_dict"):
                result[attr] = value.to_dict()
            elif isinstance(value, dict):
                result[attr] = dict(map(
                    lambda item: (item[0], item[1].to_dict())
                    if hasattr(item[1], "to_dict") else item,
                    value.items()
                ))
            else:
                result[attr] = value

        return result

    def to_str(self):
        """Returns the string representation of the model"""
        return pprint.pformat(self.to_dict())

    def __repr__(self):
        """For `print` and `pprint`"""
        return self.to_str()

    def __eq__(self, other):
        """Returns true if both objects are equal"""
        if not isinstance(other, WebcaptureUrl):
            return False

        return self.__dict__ == other.__dict__

    def __ne__(self, other):
        """Returns true if both objects are not equal"""
        return not self == other
