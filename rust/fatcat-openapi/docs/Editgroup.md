# Editgroup

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**editgroup_id** | **String** | Fatcat identifier for this editgroup. Assigned on creation.  | [optional] [default to None]
**editor_id** | **String** | Fatcat identifer of editor that created this editgroup.  | [optional] [default to None]
**editor** | [***models::Editor**](editor.md) |  | [optional] [default to None]
**changelog_index** | **i64** | For accepted/merged editgroups, the changelog index that the accept occured at. WARNING: not populated in all contexts that an editgroup could be included in a response.  | [optional] [default to None]
**created** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | Timestamp when this editgroup was first created.  | [optional] [default to None]
**submitted** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | Timestamp when this editgroup was most recently submitted for review. If withdrawn, or never submitted, will be `null`.  | [optional] [default to None]
**description** | **String** | Comment describing the changes in this editgroup. Can be updated with PUT request.  | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata attached to this editgroup. Eg, metadata provenance, or script user-agent details. See guide for (unenforced) schema norms.  | [optional] [default to None]
**annotations** | [**Vec<models::EditgroupAnnotation>**](editgroup_annotation.md) | Only included in GET responses, and not in all contexts. Do not include this field in PUT or POST requests.  | [optional] [default to None]
**edits** | [***models::EditgroupEdits**](editgroup_edits.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


