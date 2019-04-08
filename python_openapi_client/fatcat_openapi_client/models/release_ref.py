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


class ReleaseRef(object):
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
        'index': 'int',
        'target_release_id': 'str',
        'extra': 'dict(str, object)',
        'key': 'str',
        'year': 'int',
        'container_name': 'str',
        'title': 'str',
        'locator': 'str'
    }

    attribute_map = {
        'index': 'index',
        'target_release_id': 'target_release_id',
        'extra': 'extra',
        'key': 'key',
        'year': 'year',
        'container_name': 'container_name',
        'title': 'title',
        'locator': 'locator'
    }

    def __init__(self, index=None, target_release_id=None, extra=None, key=None, year=None, container_name=None, title=None, locator=None):  # noqa: E501
        """ReleaseRef - a model defined in OpenAPI"""  # noqa: E501

        self._index = None
        self._target_release_id = None
        self._extra = None
        self._key = None
        self._year = None
        self._container_name = None
        self._title = None
        self._locator = None
        self.discriminator = None

        if index is not None:
            self.index = index
        if target_release_id is not None:
            self.target_release_id = target_release_id
        if extra is not None:
            self.extra = extra
        if key is not None:
            self.key = key
        if year is not None:
            self.year = year
        if container_name is not None:
            self.container_name = container_name
        if title is not None:
            self.title = title
        if locator is not None:
            self.locator = locator

    @property
    def index(self):
        """Gets the index of this ReleaseRef.  # noqa: E501

        Zero-indexed sequence number of this reference in the list of references. Assigned automatically and used internally; don't confuse with `key`.   # noqa: E501

        :return: The index of this ReleaseRef.  # noqa: E501
        :rtype: int
        """
        return self._index

    @index.setter
    def index(self, index):
        """Sets the index of this ReleaseRef.

        Zero-indexed sequence number of this reference in the list of references. Assigned automatically and used internally; don't confuse with `key`.   # noqa: E501

        :param index: The index of this ReleaseRef.  # noqa: E501
        :type: int
        """

        self._index = index

    @property
    def target_release_id(self):
        """Gets the target_release_id of this ReleaseRef.  # noqa: E501

        Optional, fatcat identifier of release entity that this reference is citing.   # noqa: E501

        :return: The target_release_id of this ReleaseRef.  # noqa: E501
        :rtype: str
        """
        return self._target_release_id

    @target_release_id.setter
    def target_release_id(self, target_release_id):
        """Sets the target_release_id of this ReleaseRef.

        Optional, fatcat identifier of release entity that this reference is citing.   # noqa: E501

        :param target_release_id: The target_release_id of this ReleaseRef.  # noqa: E501
        :type: str
        """
        if target_release_id is not None and len(target_release_id) > 26:
            raise ValueError("Invalid value for `target_release_id`, length must be less than or equal to `26`")  # noqa: E501
        if target_release_id is not None and len(target_release_id) < 26:
            raise ValueError("Invalid value for `target_release_id`, length must be greater than or equal to `26`")  # noqa: E501
        if target_release_id is not None and not re.search(r'[a-zA-Z2-7]{26}', target_release_id):  # noqa: E501
            raise ValueError(r"Invalid value for `target_release_id`, must be a follow pattern or equal to `/[a-zA-Z2-7]{26}/`")  # noqa: E501

        self._target_release_id = target_release_id

    @property
    def extra(self):
        """Gets the extra of this ReleaseRef.  # noqa: E501

        Additional free-form JSON metadata about this citation. Generally follows Citation Style Language (CSL) JSON schema. See guide for details.   # noqa: E501

        :return: The extra of this ReleaseRef.  # noqa: E501
        :rtype: dict(str, object)
        """
        return self._extra

    @extra.setter
    def extra(self, extra):
        """Sets the extra of this ReleaseRef.

        Additional free-form JSON metadata about this citation. Generally follows Citation Style Language (CSL) JSON schema. See guide for details.   # noqa: E501

        :param extra: The extra of this ReleaseRef.  # noqa: E501
        :type: dict(str, object)
        """

        self._extra = extra

    @property
    def key(self):
        """Gets the key of this ReleaseRef.  # noqa: E501

        Short string used to indicate this reference from within the release text; or numbering of references as typeset in the release itself. Optional; don't confuse with `index` field.   # noqa: E501

        :return: The key of this ReleaseRef.  # noqa: E501
        :rtype: str
        """
        return self._key

    @key.setter
    def key(self, key):
        """Sets the key of this ReleaseRef.

        Short string used to indicate this reference from within the release text; or numbering of references as typeset in the release itself. Optional; don't confuse with `index` field.   # noqa: E501

        :param key: The key of this ReleaseRef.  # noqa: E501
        :type: str
        """

        self._key = key

    @property
    def year(self):
        """Gets the year of this ReleaseRef.  # noqa: E501

        Year that the cited work was published in.   # noqa: E501

        :return: The year of this ReleaseRef.  # noqa: E501
        :rtype: int
        """
        return self._year

    @year.setter
    def year(self, year):
        """Sets the year of this ReleaseRef.

        Year that the cited work was published in.   # noqa: E501

        :param year: The year of this ReleaseRef.  # noqa: E501
        :type: int
        """

        self._year = year

    @property
    def container_name(self):
        """Gets the container_name of this ReleaseRef.  # noqa: E501

        Name of the container (eg, journal) that the citation work was published as part of. May be an acronym or full name.   # noqa: E501

        :return: The container_name of this ReleaseRef.  # noqa: E501
        :rtype: str
        """
        return self._container_name

    @container_name.setter
    def container_name(self, container_name):
        """Sets the container_name of this ReleaseRef.

        Name of the container (eg, journal) that the citation work was published as part of. May be an acronym or full name.   # noqa: E501

        :param container_name: The container_name of this ReleaseRef.  # noqa: E501
        :type: str
        """

        self._container_name = container_name

    @property
    def title(self):
        """Gets the title of this ReleaseRef.  # noqa: E501

        Name of the work being cited.  # noqa: E501

        :return: The title of this ReleaseRef.  # noqa: E501
        :rtype: str
        """
        return self._title

    @title.setter
    def title(self, title):
        """Sets the title of this ReleaseRef.

        Name of the work being cited.  # noqa: E501

        :param title: The title of this ReleaseRef.  # noqa: E501
        :type: str
        """

        self._title = title

    @property
    def locator(self):
        """Gets the locator of this ReleaseRef.  # noqa: E501

        Page number or other indicator of the specific subset of a work being cited. Not to be confused with the first page (or page range) of an entire paper or chapter being cited.   # noqa: E501

        :return: The locator of this ReleaseRef.  # noqa: E501
        :rtype: str
        """
        return self._locator

    @locator.setter
    def locator(self, locator):
        """Sets the locator of this ReleaseRef.

        Page number or other indicator of the specific subset of a work being cited. Not to be confused with the first page (or page range) of an entire paper or chapter being cited.   # noqa: E501

        :param locator: The locator of this ReleaseRef.  # noqa: E501
        :type: str
        """

        self._locator = locator

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
        if not isinstance(other, ReleaseRef):
            return False

        return self.__dict__ == other.__dict__

    def __ne__(self, other):
        """Returns true if both objects are not equal"""
        return not self == other