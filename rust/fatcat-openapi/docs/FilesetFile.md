# FilesetFile

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**path** | **String** | Path name of file within this fileset (eg, directory)  | 
**size** | **i64** | File size in bytes | 
**md5** | **String** | MD5 hash of data, in hex encoding | [optional] [default to None]
**sha1** | **String** | SHA-1 hash of data, in hex encoding | [optional] [default to None]
**sha256** | **String** | SHA-256 hash of data, in hex encoding | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form additional metadata about this specific file in the set. Eg, `mimetype`. See guide for nomative (but unenforced) schema fields.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


