# EditgroupAnnotation

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**annotation_id** | **String** | UUID (lower-case, dash-separated, hex-encoded 128-bit) | [optional] [default to None]
**editgroup_id** | **String** | Editgroup that this annotation applies to. Set automatically in creations based on URL parameter.  | [optional] [default to None]
**editor_id** | **String** | Defaults to editor created the annotation via POST request.  | [optional] [default to None]
**editor** | [***models::Editor**](editor.md) |  | [optional] [default to None]
**created** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | Timestamp when annotation was first created.  | [optional] [default to None]
**comment_markdown** | **String** |  | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Additional free-form JSON metadata that can be included as part of the annotation (or even as the primary annotation itself). See guide for details.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


