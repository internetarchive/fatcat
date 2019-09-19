# WebcaptureCdxLine

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**surt** | **String** | \"Sortable URL\" format. See guide for details.  | 
**timestamp** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | Date and time of capture, in ISO format. UTC, 'Z'-terminated, second (or better) precision.  | 
**url** | **String** | Full URL/URI of resource captured.  | 
**mimetype** | **String** | Mimetype of the resource at this URL. May be the Content-Type header, or the actually sniffed file type.  | [optional] [default to None]
**status_code** | **i64** | HTTP status code. Should generally be 200, especially for the primary resource, but may be 3xx (redirect) or even error codes if embedded resources can not be fetched successfully.  | [optional] [default to None]
**size** | **i64** | Resource (file) size in bytes | [optional] [default to None]
**sha1** | **String** | SHA-1 hash of data, in hex encoding | 
**sha256** | **String** | SHA-256 hash of data, in hex encoding | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


