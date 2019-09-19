# CreatorEntity

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**state** | **String** |  | [optional] [default to None]
**ident** | **String** | base32-encoded unique identifier | [optional] [default to None]
**revision** | **String** | UUID (lower-case, dash-separated, hex-encoded 128-bit) | [optional] [default to None]
**redirect** | **String** | base32-encoded unique identifier | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.  | [optional] [default to None]
**edit_extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).  | [optional] [default to None]
**display_name** | **String** | Name as should be displayed in web interface or in author lists (not index/sorted). Required for valid entities.  | [optional] [default to None]
**given_name** | **String** | In English commonly the first name, but ordering is context and culture specific.  | [optional] [default to None]
**surname** | **String** | In English commonly the last, or family name, but ordering is context and culture specific.  | [optional] [default to None]
**orcid** | **String** | ORCiD (https://orcid.org) identifier | [optional] [default to None]
**wikidata_qid** | **String** | Wikidata entity QID | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


