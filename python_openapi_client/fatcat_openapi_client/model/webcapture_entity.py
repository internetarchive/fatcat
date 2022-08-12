"""
    fatcat

    Fatcat is a scalable, versioned, API-oriented catalog of bibliographic entities and file metadata.   # noqa: E501

    The version of the OpenAPI document: 0.5.0
    Contact: webservices@archive.org
    Generated by: https://openapi-generator.tech
"""


import re  # noqa: F401
import sys  # noqa: F401

from fatcat_openapi_client.model_utils import (  # noqa: F401
    ApiTypeError,
    ModelComposed,
    ModelNormal,
    ModelSimple,
    cached_property,
    change_keys_js_to_python,
    convert_js_args_to_python_args,
    date,
    datetime,
    file_type,
    none_type,
    validate_get_composed_info,
    OpenApiModel
)
from fatcat_openapi_client.exceptions import ApiAttributeError


def lazy_import():
    from fatcat_openapi_client.model.release_entity import ReleaseEntity
    from fatcat_openapi_client.model.webcapture_cdx_line import WebcaptureCdxLine
    from fatcat_openapi_client.model.webcapture_url import WebcaptureUrl
    globals()['ReleaseEntity'] = ReleaseEntity
    globals()['WebcaptureCdxLine'] = WebcaptureCdxLine
    globals()['WebcaptureUrl'] = WebcaptureUrl


class WebcaptureEntity(ModelNormal):
    """NOTE: This class is auto generated by OpenAPI Generator.
    Ref: https://openapi-generator.tech

    Do not edit the class manually.

    Attributes:
      allowed_values (dict): The key is the tuple path to the attribute
          and the for var_name this is (var_name,). The value is a dict
          with a capitalized key describing the allowed value and an allowed
          value. These dicts store the allowed enum values.
      attribute_map (dict): The key is attribute name
          and the value is json key in definition.
      discriminator_value_class_map (dict): A dict to go from the discriminator
          variable value to the discriminator class name.
      validations (dict): The key is the tuple path to the attribute
          and the for var_name this is (var_name,). The value is a dict
          that stores validations for max_length, min_length, max_items,
          min_items, exclusive_maximum, inclusive_maximum, exclusive_minimum,
          inclusive_minimum, and regex.
      additional_properties_type (tuple): A tuple of classes accepted
          as additional properties values.
    """

    allowed_values = {
        ('state',): {
            'WIP': "wip",
            'ACTIVE': "active",
            'REDIRECT': "redirect",
            'DELETED': "deleted",
        },
    }

    validations = {
        ('ident',): {
            'max_length': 26,
            'min_length': 26,
        },
        ('revision',): {
            'max_length': 36,
            'min_length': 36,
        },
        ('redirect',): {
            'max_length': 26,
            'min_length': 26,
        },
    }

    @cached_property
    def additional_properties_type():
        """
        This must be a method because a model may have properties that are
        of type self, this must run after the class is loaded
        """
        lazy_import()
        return (bool, date, datetime, dict, float, int, list, str, none_type,)  # noqa: E501

    _nullable = False

    @cached_property
    def openapi_types():
        """
        This must be a method because a model may have properties that are
        of type self, this must run after the class is loaded

        Returns
            openapi_types (dict): The key is attribute name
                and the value is attribute type.
        """
        lazy_import()
        return {
            'state': (str,),  # noqa: E501
            'ident': (str,),  # noqa: E501
            'revision': (str,),  # noqa: E501
            'redirect': (str,),  # noqa: E501
            'extra': ({str: (bool, date, datetime, dict, float, int, list, str, none_type,)},),  # noqa: E501
            'edit_extra': ({str: (bool, date, datetime, dict, float, int, list, str, none_type,)},),  # noqa: E501
            'cdx': ([WebcaptureCdxLine],),  # noqa: E501
            'archive_urls': ([WebcaptureUrl],),  # noqa: E501
            'original_url': (str,),  # noqa: E501
            'timestamp': (datetime,),  # noqa: E501
            'content_scope': (str,),  # noqa: E501
            'release_ids': ([str],),  # noqa: E501
            'releases': ([ReleaseEntity],),  # noqa: E501
        }

    @cached_property
    def discriminator():
        return None


    attribute_map = {
        'state': 'state',  # noqa: E501
        'ident': 'ident',  # noqa: E501
        'revision': 'revision',  # noqa: E501
        'redirect': 'redirect',  # noqa: E501
        'extra': 'extra',  # noqa: E501
        'edit_extra': 'edit_extra',  # noqa: E501
        'cdx': 'cdx',  # noqa: E501
        'archive_urls': 'archive_urls',  # noqa: E501
        'original_url': 'original_url',  # noqa: E501
        'timestamp': 'timestamp',  # noqa: E501
        'content_scope': 'content_scope',  # noqa: E501
        'release_ids': 'release_ids',  # noqa: E501
        'releases': 'releases',  # noqa: E501
    }

    read_only_vars = {
    }

    _composed_schemas = {}

    @classmethod
    @convert_js_args_to_python_args
    def _from_openapi_data(cls, *args, **kwargs):  # noqa: E501
        """WebcaptureEntity - a model defined in OpenAPI

        Keyword Args:
            _check_type (bool): if True, values for parameters in openapi_types
                                will be type checked and a TypeError will be
                                raised if the wrong type is input.
                                Defaults to True
            _path_to_item (tuple/list): This is a list of keys or values to
                                drill down to the model in received_data
                                when deserializing a response
            _spec_property_naming (bool): True if the variable names in the input data
                                are serialized names, as specified in the OpenAPI document.
                                False if the variable names in the input data
                                are pythonic names, e.g. snake case (default)
            _configuration (Configuration): the instance to use when
                                deserializing a file_type parameter.
                                If passed, type conversion is attempted
                                If omitted no type conversion is done.
            _visited_composed_classes (tuple): This stores a tuple of
                                classes that we have traveled through so that
                                if we see that class again we will not use its
                                discriminator again.
                                When traveling through a discriminator, the
                                composed schema that is
                                is traveled through is added to this set.
                                For example if Animal has a discriminator
                                petType and we pass in "Dog", and the class Dog
                                allOf includes Animal, we move through Animal
                                once using the discriminator, and pick Dog.
                                Then in Dog, we will make an instance of the
                                Animal class but this time we won't travel
                                through its discriminator because we passed in
                                _visited_composed_classes = (Animal,)
            state (str): [optional]  # noqa: E501
            ident (str): base32-encoded unique identifier. [optional]  # noqa: E501
            revision (str): UUID (lower-case, dash-separated, hex-encoded 128-bit). [optional]  # noqa: E501
            redirect (str): base32-encoded unique identifier. [optional]  # noqa: E501
            extra ({str: (bool, date, datetime, dict, float, int, list, str, none_type,)}): Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions. . [optional]  # noqa: E501
            edit_extra ({str: (bool, date, datetime, dict, float, int, list, str, none_type,)}): Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete). . [optional]  # noqa: E501
            cdx ([WebcaptureCdxLine]): [optional]  # noqa: E501
            archive_urls ([WebcaptureUrl]): [optional]  # noqa: E501
            original_url (str): Base URL of the primary resource this is a capture of. [optional]  # noqa: E501
            timestamp (datetime): Same format as CDX line timestamp (UTC, etc). Corresponds to the overall capture timestamp. Should generally be the timestamp of capture of the primary resource URL. . [optional]  # noqa: E501
            content_scope (str): [optional]  # noqa: E501
            release_ids ([str]): Set of identifier of release entities this fileset represents a full manifestation of. Usually a single release. . [optional]  # noqa: E501
            releases ([ReleaseEntity]): Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests. . [optional]  # noqa: E501
        """

        _check_type = kwargs.pop('_check_type', True)
        _spec_property_naming = kwargs.pop('_spec_property_naming', True)
        _path_to_item = kwargs.pop('_path_to_item', ())
        _configuration = kwargs.pop('_configuration', None)
        _visited_composed_classes = kwargs.pop('_visited_composed_classes', ())

        self = super(OpenApiModel, cls).__new__(cls)

        if args:
            for arg in args:
                if isinstance(arg, dict):
                    kwargs.update(arg)
                else:
                    raise ApiTypeError(
                        "Invalid positional arguments=%s passed to %s. Remove those invalid positional arguments." % (
                            args,
                            self.__class__.__name__,
                        ),
                        path_to_item=_path_to_item,
                        valid_classes=(self.__class__,),
                    )

        self._data_store = {}
        self._check_type = _check_type
        self._spec_property_naming = _spec_property_naming
        self._path_to_item = _path_to_item
        self._configuration = _configuration
        self._visited_composed_classes = _visited_composed_classes + (self.__class__,)

        for var_name, var_value in kwargs.items():
            if var_name not in self.attribute_map and \
                        self._configuration is not None and \
                        self._configuration.discard_unknown_keys and \
                        self.additional_properties_type is None:
                # discard variable.
                continue
            setattr(self, var_name, var_value)
        return self

    required_properties = set([
        '_data_store',
        '_check_type',
        '_spec_property_naming',
        '_path_to_item',
        '_configuration',
        '_visited_composed_classes',
    ])

    @convert_js_args_to_python_args
    def __init__(self, *args, **kwargs):  # noqa: E501
        """WebcaptureEntity - a model defined in OpenAPI

        Keyword Args:
            _check_type (bool): if True, values for parameters in openapi_types
                                will be type checked and a TypeError will be
                                raised if the wrong type is input.
                                Defaults to True
            _path_to_item (tuple/list): This is a list of keys or values to
                                drill down to the model in received_data
                                when deserializing a response
            _spec_property_naming (bool): True if the variable names in the input data
                                are serialized names, as specified in the OpenAPI document.
                                False if the variable names in the input data
                                are pythonic names, e.g. snake case (default)
            _configuration (Configuration): the instance to use when
                                deserializing a file_type parameter.
                                If passed, type conversion is attempted
                                If omitted no type conversion is done.
            _visited_composed_classes (tuple): This stores a tuple of
                                classes that we have traveled through so that
                                if we see that class again we will not use its
                                discriminator again.
                                When traveling through a discriminator, the
                                composed schema that is
                                is traveled through is added to this set.
                                For example if Animal has a discriminator
                                petType and we pass in "Dog", and the class Dog
                                allOf includes Animal, we move through Animal
                                once using the discriminator, and pick Dog.
                                Then in Dog, we will make an instance of the
                                Animal class but this time we won't travel
                                through its discriminator because we passed in
                                _visited_composed_classes = (Animal,)
            state (str): [optional]  # noqa: E501
            ident (str): base32-encoded unique identifier. [optional]  # noqa: E501
            revision (str): UUID (lower-case, dash-separated, hex-encoded 128-bit). [optional]  # noqa: E501
            redirect (str): base32-encoded unique identifier. [optional]  # noqa: E501
            extra ({str: (bool, date, datetime, dict, float, int, list, str, none_type,)}): Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions. . [optional]  # noqa: E501
            edit_extra ({str: (bool, date, datetime, dict, float, int, list, str, none_type,)}): Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete). . [optional]  # noqa: E501
            cdx ([WebcaptureCdxLine]): [optional]  # noqa: E501
            archive_urls ([WebcaptureUrl]): [optional]  # noqa: E501
            original_url (str): Base URL of the primary resource this is a capture of. [optional]  # noqa: E501
            timestamp (datetime): Same format as CDX line timestamp (UTC, etc). Corresponds to the overall capture timestamp. Should generally be the timestamp of capture of the primary resource URL. . [optional]  # noqa: E501
            content_scope (str): [optional]  # noqa: E501
            release_ids ([str]): Set of identifier of release entities this fileset represents a full manifestation of. Usually a single release. . [optional]  # noqa: E501
            releases ([ReleaseEntity]): Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests. . [optional]  # noqa: E501
        """

        _check_type = kwargs.pop('_check_type', True)
        _spec_property_naming = kwargs.pop('_spec_property_naming', False)
        _path_to_item = kwargs.pop('_path_to_item', ())
        _configuration = kwargs.pop('_configuration', None)
        _visited_composed_classes = kwargs.pop('_visited_composed_classes', ())

        if args:
            for arg in args:
                if isinstance(arg, dict):
                    kwargs.update(arg)
                else:
                    raise ApiTypeError(
                        "Invalid positional arguments=%s passed to %s. Remove those invalid positional arguments." % (
                            args,
                            self.__class__.__name__,
                        ),
                        path_to_item=_path_to_item,
                        valid_classes=(self.__class__,),
                    )

        self._data_store = {}
        self._check_type = _check_type
        self._spec_property_naming = _spec_property_naming
        self._path_to_item = _path_to_item
        self._configuration = _configuration
        self._visited_composed_classes = _visited_composed_classes + (self.__class__,)

        for var_name, var_value in kwargs.items():
            if var_name not in self.attribute_map and \
                        self._configuration is not None and \
                        self._configuration.discard_unknown_keys and \
                        self.additional_properties_type is None:
                # discard variable.
                continue
            setattr(self, var_name, var_value)
            if var_name in self.read_only_vars:
                raise ApiAttributeError(f"`{var_name}` is a read-only attribute. Use `from_openapi_data` to instantiate "
                                     f"class with read only attributes.")
