# ReleaseRef

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**index** | **i64** | Zero-indexed sequence number of this reference in the list of references. Assigned automatically and used internally; don't confuse with `key`.  | [optional] [default to None]
**target_release_id** | **String** | Optional, fatcat identifier of release entity that this reference is citing.  | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Additional free-form JSON metadata about this citation. Generally follows Citation Style Language (CSL) JSON schema. See guide for details.  | [optional] [default to None]
**key** | **String** | Short string used to indicate this reference from within the release text; or numbering of references as typeset in the release itself. Optional; don't confuse with `index` field.  | [optional] [default to None]
**year** | **i64** | Year that the cited work was published in.  | [optional] [default to None]
**container_name** | **String** | Name of the container (eg, journal) that the citation work was published as part of. May be an acronym or full name.  | [optional] [default to None]
**title** | **String** | Name of the work being cited. | [optional] [default to None]
**locator** | **String** | Page number or other indicator of the specific subset of a work being cited. Not to be confused with the first page (or page range) of an entire paper or chapter being cited.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


