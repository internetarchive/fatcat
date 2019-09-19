# ReleaseContrib

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**index** | **i64** | Internally assigned zero-indexed sequence number of contribution. Authors should come first; this encodes the order of attriubtion.  | [optional] [default to None]
**creator_id** | **String** | If known, indicates the creator entity this contribution was made by.  | [optional] [default to None]
**creator** | [***models::CreatorEntity**](creator_entity.md) |  | [optional] [default to None]
**raw_name** | **String** | Full name of the contributor as typeset in the release.  | [optional] [default to None]
**given_name** | **String** | In English commonly the first name, but ordering is context and culture specific.  | [optional] [default to None]
**surname** | **String** | In English commonly the last, or family name, but ordering is context and culture specific.  | [optional] [default to None]
**role** | **String** | Short string (slug) indicating type of contribution (eg, \"author\", \"translator\"). See guide for list of accpeted values.  | [optional] [default to None]
**raw_affiliation** | **String** | Raw affiliation string as displayed in text | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Additional free-form JSON metadata about this contributor/contribution. See guide for normative schema.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


