# AuthOidc

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**provider** | **String** | Fatcat-specific short name (slug) for remote service being used for authentication.  | 
**sub** | **String** | `SUB` from OIDC protocol. Usually a URL. | 
**iss** | **String** | `ISS` from OIDC protocol. Usually a stable account username, number, or identifier. | 
**preferred_username** | **String** | What it sounds like; returned by OIDC, and used as a hint when creating new editor accounts. Fatcat usernames are usually this string with the `provider` slug as a suffix, though some munging may occur.  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


