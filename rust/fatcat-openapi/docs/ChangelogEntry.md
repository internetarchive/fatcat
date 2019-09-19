# ChangelogEntry

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**index** | **i64** | Monotonically increasing sequence number of this changelog entry.  | 
**editgroup_id** | **String** | Identifier of editgroup accepted/merged in this changelog entry.  | 
**timestamp** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | Date and time when the editgroup was accpeted.  | 
**editgroup** | [***models::Editgroup**](editgroup.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


