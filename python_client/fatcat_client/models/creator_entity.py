# coding: utf-8

"""
    fatcat

    A scalable, versioned, API-oriented catalog of bibliographic entities and file metadata  # noqa: E501

    OpenAPI spec version: 0.2.0
    
    Generated by: https://github.com/swagger-api/swagger-codegen.git
"""


import pprint
import re  # noqa: F401

import six


class CreatorEntity(object):
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
        'wikidata_qid': 'str',
        'orcid': 'str',
        'surname': 'str',
        'given_name': 'str',
        'display_name': 'str',
        'state': 'str',
        'ident': 'str',
        'revision': 'str',
        'redirect': 'str',
        'extra': 'object',
        'edit_extra': 'object'
    }

    attribute_map = {
        'wikidata_qid': 'wikidata_qid',
        'orcid': 'orcid',
        'surname': 'surname',
        'given_name': 'given_name',
        'display_name': 'display_name',
        'state': 'state',
        'ident': 'ident',
        'revision': 'revision',
        'redirect': 'redirect',
        'extra': 'extra',
        'edit_extra': 'edit_extra'
    }

    def __init__(self, wikidata_qid=None, orcid=None, surname=None, given_name=None, display_name=None, state=None, ident=None, revision=None, redirect=None, extra=None, edit_extra=None):  # noqa: E501
        """CreatorEntity - a model defined in Swagger"""  # noqa: E501

        self._wikidata_qid = None
        self._orcid = None
        self._surname = None
        self._given_name = None
        self._display_name = None
        self._state = None
        self._ident = None
        self._revision = None
        self._redirect = None
        self._extra = None
        self._edit_extra = None
        self.discriminator = None

        if wikidata_qid is not None:
            self.wikidata_qid = wikidata_qid
        if orcid is not None:
            self.orcid = orcid
        if surname is not None:
            self.surname = surname
        if given_name is not None:
            self.given_name = given_name
        if display_name is not None:
            self.display_name = display_name
        if state is not None:
            self.state = state
        if ident is not None:
            self.ident = ident
        if revision is not None:
            self.revision = revision
        if redirect is not None:
            self.redirect = redirect
        if extra is not None:
            self.extra = extra
        if edit_extra is not None:
            self.edit_extra = edit_extra

    @property
    def wikidata_qid(self):
        """Gets the wikidata_qid of this CreatorEntity.  # noqa: E501


        :return: The wikidata_qid of this CreatorEntity.  # noqa: E501
        :rtype: str
        """
        return self._wikidata_qid

    @wikidata_qid.setter
    def wikidata_qid(self, wikidata_qid):
        """Sets the wikidata_qid of this CreatorEntity.


        :param wikidata_qid: The wikidata_qid of this CreatorEntity.  # noqa: E501
        :type: str
        """

        self._wikidata_qid = wikidata_qid

    @property
    def orcid(self):
        """Gets the orcid of this CreatorEntity.  # noqa: E501


        :return: The orcid of this CreatorEntity.  # noqa: E501
        :rtype: str
        """
        return self._orcid

    @orcid.setter
    def orcid(self, orcid):
        """Sets the orcid of this CreatorEntity.


        :param orcid: The orcid of this CreatorEntity.  # noqa: E501
        :type: str
        """
        if orcid is not None and len(orcid) > 19:
            raise ValueError("Invalid value for `orcid`, length must be less than or equal to `19`")  # noqa: E501
        if orcid is not None and len(orcid) < 19:
            raise ValueError("Invalid value for `orcid`, length must be greater than or equal to `19`")  # noqa: E501
        if orcid is not None and not re.search('\\d{4}-\\d{4}-\\d{4}-\\d{3}[\\dX]', orcid):  # noqa: E501
            raise ValueError("Invalid value for `orcid`, must be a follow pattern or equal to `/\\d{4}-\\d{4}-\\d{4}-\\d{3}[\\dX]/`")  # noqa: E501

        self._orcid = orcid

    @property
    def surname(self):
        """Gets the surname of this CreatorEntity.  # noqa: E501


        :return: The surname of this CreatorEntity.  # noqa: E501
        :rtype: str
        """
        return self._surname

    @surname.setter
    def surname(self, surname):
        """Sets the surname of this CreatorEntity.


        :param surname: The surname of this CreatorEntity.  # noqa: E501
        :type: str
        """

        self._surname = surname

    @property
    def given_name(self):
        """Gets the given_name of this CreatorEntity.  # noqa: E501


        :return: The given_name of this CreatorEntity.  # noqa: E501
        :rtype: str
        """
        return self._given_name

    @given_name.setter
    def given_name(self, given_name):
        """Sets the given_name of this CreatorEntity.


        :param given_name: The given_name of this CreatorEntity.  # noqa: E501
        :type: str
        """

        self._given_name = given_name

    @property
    def display_name(self):
        """Gets the display_name of this CreatorEntity.  # noqa: E501

        Required for valid entities  # noqa: E501

        :return: The display_name of this CreatorEntity.  # noqa: E501
        :rtype: str
        """
        return self._display_name

    @display_name.setter
    def display_name(self, display_name):
        """Sets the display_name of this CreatorEntity.

        Required for valid entities  # noqa: E501

        :param display_name: The display_name of this CreatorEntity.  # noqa: E501
        :type: str
        """

        self._display_name = display_name

    @property
    def state(self):
        """Gets the state of this CreatorEntity.  # noqa: E501


        :return: The state of this CreatorEntity.  # noqa: E501
        :rtype: str
        """
        return self._state

    @state.setter
    def state(self, state):
        """Sets the state of this CreatorEntity.


        :param state: The state of this CreatorEntity.  # noqa: E501
        :type: str
        """
        allowed_values = ["wip", "active", "redirect", "deleted"]  # noqa: E501
        if state not in allowed_values:
            raise ValueError(
                "Invalid value for `state` ({0}), must be one of {1}"  # noqa: E501
                .format(state, allowed_values)
            )

        self._state = state

    @property
    def ident(self):
        """Gets the ident of this CreatorEntity.  # noqa: E501

        base32-encoded unique identifier  # noqa: E501

        :return: The ident of this CreatorEntity.  # noqa: E501
        :rtype: str
        """
        return self._ident

    @ident.setter
    def ident(self, ident):
        """Sets the ident of this CreatorEntity.

        base32-encoded unique identifier  # noqa: E501

        :param ident: The ident of this CreatorEntity.  # noqa: E501
        :type: str
        """
        if ident is not None and len(ident) > 26:
            raise ValueError("Invalid value for `ident`, length must be less than or equal to `26`")  # noqa: E501
        if ident is not None and len(ident) < 26:
            raise ValueError("Invalid value for `ident`, length must be greater than or equal to `26`")  # noqa: E501
        if ident is not None and not re.search('[a-zA-Z2-7]{26}', ident):  # noqa: E501
            raise ValueError("Invalid value for `ident`, must be a follow pattern or equal to `/[a-zA-Z2-7]{26}/`")  # noqa: E501

        self._ident = ident

    @property
    def revision(self):
        """Gets the revision of this CreatorEntity.  # noqa: E501

        UUID (lower-case, dash-separated, hex-encoded 128-bit)  # noqa: E501

        :return: The revision of this CreatorEntity.  # noqa: E501
        :rtype: str
        """
        return self._revision

    @revision.setter
    def revision(self, revision):
        """Sets the revision of this CreatorEntity.

        UUID (lower-case, dash-separated, hex-encoded 128-bit)  # noqa: E501

        :param revision: The revision of this CreatorEntity.  # noqa: E501
        :type: str
        """
        if revision is not None and len(revision) > 36:
            raise ValueError("Invalid value for `revision`, length must be less than or equal to `36`")  # noqa: E501
        if revision is not None and len(revision) < 36:
            raise ValueError("Invalid value for `revision`, length must be greater than or equal to `36`")  # noqa: E501
        if revision is not None and not re.search('[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}', revision):  # noqa: E501
            raise ValueError("Invalid value for `revision`, must be a follow pattern or equal to `/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/`")  # noqa: E501

        self._revision = revision

    @property
    def redirect(self):
        """Gets the redirect of this CreatorEntity.  # noqa: E501

        base32-encoded unique identifier  # noqa: E501

        :return: The redirect of this CreatorEntity.  # noqa: E501
        :rtype: str
        """
        return self._redirect

    @redirect.setter
    def redirect(self, redirect):
        """Sets the redirect of this CreatorEntity.

        base32-encoded unique identifier  # noqa: E501

        :param redirect: The redirect of this CreatorEntity.  # noqa: E501
        :type: str
        """
        if redirect is not None and len(redirect) > 26:
            raise ValueError("Invalid value for `redirect`, length must be less than or equal to `26`")  # noqa: E501
        if redirect is not None and len(redirect) < 26:
            raise ValueError("Invalid value for `redirect`, length must be greater than or equal to `26`")  # noqa: E501
        if redirect is not None and not re.search('[a-zA-Z2-7]{26}', redirect):  # noqa: E501
            raise ValueError("Invalid value for `redirect`, must be a follow pattern or equal to `/[a-zA-Z2-7]{26}/`")  # noqa: E501

        self._redirect = redirect

    @property
    def extra(self):
        """Gets the extra of this CreatorEntity.  # noqa: E501


        :return: The extra of this CreatorEntity.  # noqa: E501
        :rtype: object
        """
        return self._extra

    @extra.setter
    def extra(self, extra):
        """Sets the extra of this CreatorEntity.


        :param extra: The extra of this CreatorEntity.  # noqa: E501
        :type: object
        """

        self._extra = extra

    @property
    def edit_extra(self):
        """Gets the edit_extra of this CreatorEntity.  # noqa: E501


        :return: The edit_extra of this CreatorEntity.  # noqa: E501
        :rtype: object
        """
        return self._edit_extra

    @edit_extra.setter
    def edit_extra(self, edit_extra):
        """Sets the edit_extra of this CreatorEntity.


        :param edit_extra: The edit_extra of this CreatorEntity.  # noqa: E501
        :type: object
        """

        self._edit_extra = edit_extra

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
        if not isinstance(other, CreatorEntity):
            return False

        return self.__dict__ == other.__dict__

    def __ne__(self, other):
        """Returns true if both objects are not equal"""
        return not self == other
