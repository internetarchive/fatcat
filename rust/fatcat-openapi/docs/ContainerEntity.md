# ContainerEntity

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**state** | **String** |  | [optional] [default to None]
**ident** | **String** | base32-encoded unique identifier | [optional] [default to None]
**revision** | **String** | UUID (lower-case, dash-separated, hex-encoded 128-bit) | [optional] [default to None]
**redirect** | **String** | base32-encoded unique identifier | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.  | [optional] [default to None]
**edit_extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).  | [optional] [default to None]
**name** | **String** | Name of the container (eg, Journal title). Required for entity creation. | [optional] [default to None]
**container_type** | **String** | Type of container, eg 'journal' or 'proceedings'. See Guide for list of valid types. | [optional] [default to None]
**publisher** | **String** | Name of the organization or entity responsible for publication. Not the complete imprint/brand.  | [optional] [default to None]
**issnl** | **String** | Linking ISSN number (ISSN-L). Should be valid and registered with issn.org | [optional] [default to None]
**wikidata_qid** | **String** |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


