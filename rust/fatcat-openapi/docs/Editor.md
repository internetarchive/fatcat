# Editor

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**editor_id** | **String** | Fatcat identifier for the editor. Can not be changed.  | [optional] [default to None]
**username** | **String** | Username/handle (short slug-like string) to identify this editor. May be changed at any time by the editor; use the `editor_id` as a persistend identifer.  | 
**is_admin** | **bool** | Whether this editor has the `admin` role.  | [optional] [default to None]
**is_bot** | **bool** | Whether this editor is a bot (as opposed to a human making manual edits)  | [optional] [default to None]
**is_active** | **bool** | Whether this editor's account is enabled (if not API tokens and web logins will not work).  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


