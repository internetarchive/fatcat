# FilesetEntity

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**state** | **String** |  | [optional] [default to None]
**ident** | **String** | base32-encoded unique identifier | [optional] [default to None]
**revision** | **String** | UUID (lower-case, dash-separated, hex-encoded 128-bit) | [optional] [default to None]
**redirect** | **String** | base32-encoded unique identifier | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.  | [optional] [default to None]
**edit_extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).  | [optional] [default to None]
**manifest** | [**Vec<models::FilesetFile>**](fileset_file.md) |  | [optional] [default to None]
**urls** | [**Vec<models::FilesetUrl>**](fileset_url.md) |  | [optional] [default to None]
**release_ids** | **Vec<String>** | Set of identifier of release entities this fileset represents a full manifestation of. Usually a single release.  | [optional] [default to None]
**releases** | [**Vec<models::ReleaseEntity>**](release_entity.md) | Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


