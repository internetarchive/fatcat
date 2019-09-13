# coding: utf-8

"""
    fatcat

    Fatcat is a scalable, versioned, API-oriented catalog of bibliographic entities and file metadata.   # noqa: E501

    OpenAPI spec version: 0.3.0
    Contact: webservices@archive.org
    Generated by: https://github.com/swagger-api/swagger-codegen.git
"""


import pprint
import re  # noqa: F401

import six


class WebcaptureCdxLine(object):
    """NOTE: This class is auto generated by the swagger code generator program.

    Do not edit the class manually.
    """

    """
    Attributes:
      swagger_types (dict): The key is attribute name
                            and the value is attribute type.
      attribute_map (dict): The key is attribute name
                            and the value is json key in definition.
    """
    swagger_types = {
        'surt': 'str',
        'timestamp': 'datetime',
        'url': 'str',
        'mimetype': 'str',
        'status_code': 'int',
        'size': 'int',
        'sha1': 'str',
        'sha256': 'str'
    }

    attribute_map = {
        'surt': 'surt',
        'timestamp': 'timestamp',
        'url': 'url',
        'mimetype': 'mimetype',
        'status_code': 'status_code',
        'size': 'size',
        'sha1': 'sha1',
        'sha256': 'sha256'
    }

    def __init__(self, surt=None, timestamp=None, url=None, mimetype=None, status_code=None, size=None, sha1=None, sha256=None):  # noqa: E501
        """WebcaptureCdxLine - a model defined in Swagger"""  # noqa: E501

        self._surt = None
        self._timestamp = None
        self._url = None
        self._mimetype = None
        self._status_code = None
        self._size = None
        self._sha1 = None
        self._sha256 = None
        self.discriminator = None

        self.surt = surt
        self.timestamp = timestamp
        self.url = url
        if mimetype is not None:
            self.mimetype = mimetype
        if status_code is not None:
            self.status_code = status_code
        if size is not None:
            self.size = size
        self.sha1 = sha1
        if sha256 is not None:
            self.sha256 = sha256

    @property
    def surt(self):
        """Gets the surt of this WebcaptureCdxLine.  # noqa: E501

        \"Sortable URL\" format. See guide for details.   # noqa: E501

        :return: The surt of this WebcaptureCdxLine.  # noqa: E501
        :rtype: str
        """
        return self._surt

    @surt.setter
    def surt(self, surt):
        """Sets the surt of this WebcaptureCdxLine.

        \"Sortable URL\" format. See guide for details.   # noqa: E501

        :param surt: The surt of this WebcaptureCdxLine.  # noqa: E501
        :type: str
        """
        if surt is None:
            raise ValueError("Invalid value for `surt`, must not be `None`")  # noqa: E501

        self._surt = surt

    @property
    def timestamp(self):
        """Gets the timestamp of this WebcaptureCdxLine.  # noqa: E501

        Date and time of capture, in ISO format. UTC, 'Z'-terminated, second (or better) precision.   # noqa: E501

        :return: The timestamp of this WebcaptureCdxLine.  # noqa: E501
        :rtype: datetime
        """
        return self._timestamp

    @timestamp.setter
    def timestamp(self, timestamp):
        """Sets the timestamp of this WebcaptureCdxLine.

        Date and time of capture, in ISO format. UTC, 'Z'-terminated, second (or better) precision.   # noqa: E501

        :param timestamp: The timestamp of this WebcaptureCdxLine.  # noqa: E501
        :type: datetime
        """
        if timestamp is None:
            raise ValueError("Invalid value for `timestamp`, must not be `None`")  # noqa: E501

        self._timestamp = timestamp

    @property
    def url(self):
        """Gets the url of this WebcaptureCdxLine.  # noqa: E501

        Full URL/URI of resource captured.   # noqa: E501

        :return: The url of this WebcaptureCdxLine.  # noqa: E501
        :rtype: str
        """
        return self._url

    @url.setter
    def url(self, url):
        """Sets the url of this WebcaptureCdxLine.

        Full URL/URI of resource captured.   # noqa: E501

        :param url: The url of this WebcaptureCdxLine.  # noqa: E501
        :type: str
        """
        if url is None:
            raise ValueError("Invalid value for `url`, must not be `None`")  # noqa: E501

        self._url = url

    @property
    def mimetype(self):
        """Gets the mimetype of this WebcaptureCdxLine.  # noqa: E501

        Mimetype of the resource at this URL. May be the Content-Type header, or the actually sniffed file type.   # noqa: E501

        :return: The mimetype of this WebcaptureCdxLine.  # noqa: E501
        :rtype: str
        """
        return self._mimetype

    @mimetype.setter
    def mimetype(self, mimetype):
        """Sets the mimetype of this WebcaptureCdxLine.

        Mimetype of the resource at this URL. May be the Content-Type header, or the actually sniffed file type.   # noqa: E501

        :param mimetype: The mimetype of this WebcaptureCdxLine.  # noqa: E501
        :type: str
        """

        self._mimetype = mimetype

    @property
    def status_code(self):
        """Gets the status_code of this WebcaptureCdxLine.  # noqa: E501

        HTTP status code. Should generally be 200, especially for the primary resource, but may be 3xx (redirect) or even error codes if embedded resources can not be fetched successfully.   # noqa: E501

        :return: The status_code of this WebcaptureCdxLine.  # noqa: E501
        :rtype: int
        """
        return self._status_code

    @status_code.setter
    def status_code(self, status_code):
        """Sets the status_code of this WebcaptureCdxLine.

        HTTP status code. Should generally be 200, especially for the primary resource, but may be 3xx (redirect) or even error codes if embedded resources can not be fetched successfully.   # noqa: E501

        :param status_code: The status_code of this WebcaptureCdxLine.  # noqa: E501
        :type: int
        """

        self._status_code = status_code

    @property
    def size(self):
        """Gets the size of this WebcaptureCdxLine.  # noqa: E501

        Resource (file) size in bytes  # noqa: E501

        :return: The size of this WebcaptureCdxLine.  # noqa: E501
        :rtype: int
        """
        return self._size

    @size.setter
    def size(self, size):
        """Sets the size of this WebcaptureCdxLine.

        Resource (file) size in bytes  # noqa: E501

        :param size: The size of this WebcaptureCdxLine.  # noqa: E501
        :type: int
        """

        self._size = size

    @property
    def sha1(self):
        """Gets the sha1 of this WebcaptureCdxLine.  # noqa: E501

        SHA-1 hash of data, in hex encoding  # noqa: E501

        :return: The sha1 of this WebcaptureCdxLine.  # noqa: E501
        :rtype: str
        """
        return self._sha1

    @sha1.setter
    def sha1(self, sha1):
        """Sets the sha1 of this WebcaptureCdxLine.

        SHA-1 hash of data, in hex encoding  # noqa: E501

        :param sha1: The sha1 of this WebcaptureCdxLine.  # noqa: E501
        :type: str
        """
        if sha1 is None:
            raise ValueError("Invalid value for `sha1`, must not be `None`")  # noqa: E501
        if sha1 is not None and len(sha1) > 40:
            raise ValueError("Invalid value for `sha1`, length must be less than or equal to `40`")  # noqa: E501
        if sha1 is not None and len(sha1) < 40:
            raise ValueError("Invalid value for `sha1`, length must be greater than or equal to `40`")  # noqa: E501
        if sha1 is not None and not re.search('[a-f0-9]{40}', sha1):  # noqa: E501
            raise ValueError("Invalid value for `sha1`, must be a follow pattern or equal to `/[a-f0-9]{40}/`")  # noqa: E501

        self._sha1 = sha1

    @property
    def sha256(self):
        """Gets the sha256 of this WebcaptureCdxLine.  # noqa: E501

        SHA-256 hash of data, in hex encoding  # noqa: E501

        :return: The sha256 of this WebcaptureCdxLine.  # noqa: E501
        :rtype: str
        """
        return self._sha256

    @sha256.setter
    def sha256(self, sha256):
        """Sets the sha256 of this WebcaptureCdxLine.

        SHA-256 hash of data, in hex encoding  # noqa: E501

        :param sha256: The sha256 of this WebcaptureCdxLine.  # noqa: E501
        :type: str
        """
        if sha256 is not None and len(sha256) > 64:
            raise ValueError("Invalid value for `sha256`, length must be less than or equal to `64`")  # noqa: E501
        if sha256 is not None and len(sha256) < 64:
            raise ValueError("Invalid value for `sha256`, length must be greater than or equal to `64`")  # noqa: E501
        if sha256 is not None and not re.search('[a-f0-9]{64}', sha256):  # noqa: E501
            raise ValueError("Invalid value for `sha256`, must be a follow pattern or equal to `/[a-f0-9]{64}/`")  # noqa: E501

        self._sha256 = sha256

    def to_dict(self):
        """Returns the model properties as a dict"""
        result = {}

        for attr, _ in six.iteritems(self.swagger_types):
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
        if not isinstance(other, WebcaptureCdxLine):
            return False

        return self.__dict__ == other.__dict__

    def __ne__(self, other):
        """Returns true if both objects are not equal"""
        return not self == other
