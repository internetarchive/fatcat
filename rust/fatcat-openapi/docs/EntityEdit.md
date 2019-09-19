# EntityEdit

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**edit_id** | **String** | Unique UUID for this specific edit object.  | 
**ident** | **String** | Fatcat identifier of the entity this edit is mutating.  | 
**revision** | **String** | Entity revision that this edit will set the entity to. May be `null` in the case of deletions.  | [optional] [default to None]
**prev_revision** | **String** | Revision of entity just before this edit. May be used in the future to prevent edit race conditions.  | [optional] [default to None]
**redirect_ident** | **String** | When an edit is to merge entities (redirect one to another), this is the entity fatcat identifier for the target entity.  | [optional] [default to None]
**editgroup_id** | **String** | Editgroup identifier that this edit is part of.  | 
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


