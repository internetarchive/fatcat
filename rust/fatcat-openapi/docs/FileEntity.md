# FileEntity

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**state** | **String** |  | [optional] [default to None]
**ident** | **String** | base32-encoded unique identifier | [optional] [default to None]
**revision** | **String** | UUID (lower-case, dash-separated, hex-encoded 128-bit) | [optional] [default to None]
**redirect** | **String** | base32-encoded unique identifier | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.  | [optional] [default to None]
**edit_extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).  | [optional] [default to None]
**size** | **i64** | Size of file in bytes. Non-zero. | [optional] [default to None]
**md5** | **String** | MD5 hash of data, in hex encoding | [optional] [default to None]
**sha1** | **String** | SHA-1 hash of data, in hex encoding | [optional] [default to None]
**sha256** | **String** | SHA-256 hash of data, in hex encoding | [optional] [default to None]
**urls** | [**Vec<models::FileUrl>**](file_url.md) |  | [optional] [default to None]
**mimetype** | **String** |  | [optional] [default to None]
**release_ids** | **Vec<String>** | Set of identifier of release entities this file represents a full manifestation of. Usually a single release, but some files contain content of multiple full releases (eg, an issue of a journal).  | [optional] [default to None]
**releases** | [**Vec<models::ReleaseEntity>**](release_entity.md) | Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


