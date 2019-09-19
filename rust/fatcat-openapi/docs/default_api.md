# default_api

All URIs are relative to *https://api.fatcat.wiki/v0*

Method | HTTP request | Description
------------- | ------------- | -------------
**accept_editgroup**](default_api.md#accept_editgroup) | **POST** /editgroup/{editgroup_id}/accept | 
**auth_check**](default_api.md#auth_check) | **GET** /auth/check | 
**auth_oidc**](default_api.md#auth_oidc) | **POST** /auth/oidc | 
**create_auth_token**](default_api.md#create_auth_token) | **POST** /auth/token/{editor_id} | 
**create_container**](default_api.md#create_container) | **POST** /editgroup/{editgroup_id}/container | 
**create_container_auto_batch**](default_api.md#create_container_auto_batch) | **POST** /editgroup/auto/container/batch | 
**create_creator**](default_api.md#create_creator) | **POST** /editgroup/{editgroup_id}/creator | 
**create_creator_auto_batch**](default_api.md#create_creator_auto_batch) | **POST** /editgroup/auto/creator/batch | 
**create_editgroup**](default_api.md#create_editgroup) | **POST** /editgroup | 
**create_editgroup_annotation**](default_api.md#create_editgroup_annotation) | **POST** /editgroup/{editgroup_id}/annotation | 
**create_file**](default_api.md#create_file) | **POST** /editgroup/{editgroup_id}/file | 
**create_file_auto_batch**](default_api.md#create_file_auto_batch) | **POST** /editgroup/auto/file/batch | 
**create_fileset**](default_api.md#create_fileset) | **POST** /editgroup/{editgroup_id}/fileset | 
**create_fileset_auto_batch**](default_api.md#create_fileset_auto_batch) | **POST** /editgroup/auto/fileset/batch | 
**create_release**](default_api.md#create_release) | **POST** /editgroup/{editgroup_id}/release | 
**create_release_auto_batch**](default_api.md#create_release_auto_batch) | **POST** /editgroup/auto/release/batch | 
**create_webcapture**](default_api.md#create_webcapture) | **POST** /editgroup/{editgroup_id}/webcapture | 
**create_webcapture_auto_batch**](default_api.md#create_webcapture_auto_batch) | **POST** /editgroup/auto/webcapture/batch | 
**create_work**](default_api.md#create_work) | **POST** /editgroup/{editgroup_id}/work | 
**create_work_auto_batch**](default_api.md#create_work_auto_batch) | **POST** /editgroup/auto/work/batch | 
**delete_container**](default_api.md#delete_container) | **DELETE** /editgroup/{editgroup_id}/container/{ident} | 
**delete_container_edit**](default_api.md#delete_container_edit) | **DELETE** /editgroup/{editgroup_id}/container/edit/{edit_id} | 
**delete_creator**](default_api.md#delete_creator) | **DELETE** /editgroup/{editgroup_id}/creator/{ident} | 
**delete_creator_edit**](default_api.md#delete_creator_edit) | **DELETE** /editgroup/{editgroup_id}/creator/edit/{edit_id} | 
**delete_file**](default_api.md#delete_file) | **DELETE** /editgroup/{editgroup_id}/file/{ident} | 
**delete_file_edit**](default_api.md#delete_file_edit) | **DELETE** /editgroup/{editgroup_id}/file/edit/{edit_id} | 
**delete_fileset**](default_api.md#delete_fileset) | **DELETE** /editgroup/{editgroup_id}/fileset/{ident} | 
**delete_fileset_edit**](default_api.md#delete_fileset_edit) | **DELETE** /editgroup/{editgroup_id}/fileset/edit/{edit_id} | 
**delete_release**](default_api.md#delete_release) | **DELETE** /editgroup/{editgroup_id}/release/{ident} | 
**delete_release_edit**](default_api.md#delete_release_edit) | **DELETE** /editgroup/{editgroup_id}/release/edit/{edit_id} | 
**delete_webcapture**](default_api.md#delete_webcapture) | **DELETE** /editgroup/{editgroup_id}/webcapture/{ident} | 
**delete_webcapture_edit**](default_api.md#delete_webcapture_edit) | **DELETE** /editgroup/{editgroup_id}/webcapture/edit/{edit_id} | 
**delete_work**](default_api.md#delete_work) | **DELETE** /editgroup/{editgroup_id}/work/{ident} | 
**delete_work_edit**](default_api.md#delete_work_edit) | **DELETE** /editgroup/{editgroup_id}/work/edit/{edit_id} | 
**get_changelog**](default_api.md#get_changelog) | **GET** /changelog | 
**get_changelog_entry**](default_api.md#get_changelog_entry) | **GET** /changelog/{index} | 
**get_container**](default_api.md#get_container) | **GET** /container/{ident} | 
**get_container_edit**](default_api.md#get_container_edit) | **GET** /container/edit/{edit_id} | 
**get_container_history**](default_api.md#get_container_history) | **GET** /container/{ident}/history | 
**get_container_redirects**](default_api.md#get_container_redirects) | **GET** /container/{ident}/redirects | 
**get_container_revision**](default_api.md#get_container_revision) | **GET** /container/rev/{rev_id} | 
**get_creator**](default_api.md#get_creator) | **GET** /creator/{ident} | 
**get_creator_edit**](default_api.md#get_creator_edit) | **GET** /creator/edit/{edit_id} | 
**get_creator_history**](default_api.md#get_creator_history) | **GET** /creator/{ident}/history | 
**get_creator_redirects**](default_api.md#get_creator_redirects) | **GET** /creator/{ident}/redirects | 
**get_creator_releases**](default_api.md#get_creator_releases) | **GET** /creator/{ident}/releases | 
**get_creator_revision**](default_api.md#get_creator_revision) | **GET** /creator/rev/{rev_id} | 
**get_editgroup**](default_api.md#get_editgroup) | **GET** /editgroup/{editgroup_id} | 
**get_editgroup_annotations**](default_api.md#get_editgroup_annotations) | **GET** /editgroup/{editgroup_id}/annotations | 
**get_editgroups_reviewable**](default_api.md#get_editgroups_reviewable) | **GET** /editgroup/reviewable | 
**get_editor**](default_api.md#get_editor) | **GET** /editor/{editor_id} | 
**get_editor_annotations**](default_api.md#get_editor_annotations) | **GET** /editor/{editor_id}/annotations | 
**get_editor_editgroups**](default_api.md#get_editor_editgroups) | **GET** /editor/{editor_id}/editgroups | 
**get_file**](default_api.md#get_file) | **GET** /file/{ident} | 
**get_file_edit**](default_api.md#get_file_edit) | **GET** /file/edit/{edit_id} | 
**get_file_history**](default_api.md#get_file_history) | **GET** /file/{ident}/history | 
**get_file_redirects**](default_api.md#get_file_redirects) | **GET** /file/{ident}/redirects | 
**get_file_revision**](default_api.md#get_file_revision) | **GET** /file/rev/{rev_id} | 
**get_fileset**](default_api.md#get_fileset) | **GET** /fileset/{ident} | 
**get_fileset_edit**](default_api.md#get_fileset_edit) | **GET** /fileset/edit/{edit_id} | 
**get_fileset_history**](default_api.md#get_fileset_history) | **GET** /fileset/{ident}/history | 
**get_fileset_redirects**](default_api.md#get_fileset_redirects) | **GET** /fileset/{ident}/redirects | 
**get_fileset_revision**](default_api.md#get_fileset_revision) | **GET** /fileset/rev/{rev_id} | 
**get_release**](default_api.md#get_release) | **GET** /release/{ident} | 
**get_release_edit**](default_api.md#get_release_edit) | **GET** /release/edit/{edit_id} | 
**get_release_files**](default_api.md#get_release_files) | **GET** /release/{ident}/files | 
**get_release_filesets**](default_api.md#get_release_filesets) | **GET** /release/{ident}/filesets | 
**get_release_history**](default_api.md#get_release_history) | **GET** /release/{ident}/history | 
**get_release_redirects**](default_api.md#get_release_redirects) | **GET** /release/{ident}/redirects | 
**get_release_revision**](default_api.md#get_release_revision) | **GET** /release/rev/{rev_id} | 
**get_release_webcaptures**](default_api.md#get_release_webcaptures) | **GET** /release/{ident}/webcaptures | 
**get_webcapture**](default_api.md#get_webcapture) | **GET** /webcapture/{ident} | 
**get_webcapture_edit**](default_api.md#get_webcapture_edit) | **GET** /webcapture/edit/{edit_id} | 
**get_webcapture_history**](default_api.md#get_webcapture_history) | **GET** /webcapture/{ident}/history | 
**get_webcapture_redirects**](default_api.md#get_webcapture_redirects) | **GET** /webcapture/{ident}/redirects | 
**get_webcapture_revision**](default_api.md#get_webcapture_revision) | **GET** /webcapture/rev/{rev_id} | 
**get_work**](default_api.md#get_work) | **GET** /work/{ident} | 
**get_work_edit**](default_api.md#get_work_edit) | **GET** /work/edit/{edit_id} | 
**get_work_history**](default_api.md#get_work_history) | **GET** /work/{ident}/history | 
**get_work_redirects**](default_api.md#get_work_redirects) | **GET** /work/{ident}/redirects | 
**get_work_releases**](default_api.md#get_work_releases) | **GET** /work/{ident}/releases | 
**get_work_revision**](default_api.md#get_work_revision) | **GET** /work/rev/{rev_id} | 
**lookup_container**](default_api.md#lookup_container) | **GET** /container/lookup | 
**lookup_creator**](default_api.md#lookup_creator) | **GET** /creator/lookup | 
**lookup_file**](default_api.md#lookup_file) | **GET** /file/lookup | 
**lookup_release**](default_api.md#lookup_release) | **GET** /release/lookup | 
**update_container**](default_api.md#update_container) | **PUT** /editgroup/{editgroup_id}/container/{ident} | 
**update_creator**](default_api.md#update_creator) | **PUT** /editgroup/{editgroup_id}/creator/{ident} | 
**update_editgroup**](default_api.md#update_editgroup) | **PUT** /editgroup/{editgroup_id} | 
**update_editor**](default_api.md#update_editor) | **PUT** /editor/{editor_id} | 
**update_file**](default_api.md#update_file) | **PUT** /editgroup/{editgroup_id}/file/{ident} | 
**update_fileset**](default_api.md#update_fileset) | **PUT** /editgroup/{editgroup_id}/fileset/{ident} | 
**update_release**](default_api.md#update_release) | **PUT** /editgroup/{editgroup_id}/release/{ident} | 
**update_webcapture**](default_api.md#update_webcapture) | **PUT** /editgroup/{editgroup_id}/webcapture/{ident} | 
**update_work**](default_api.md#update_work) | **PUT** /editgroup/{editgroup_id}/work/{ident} | 


# **accept_editgroup**
> models::Success accept_editgroup(ctx, editgroup_id)


Accept (\"merge\") the given editgroup into the catalog. The editgroup must be open (not already accepted), and the editor making this request must have the `admin` role. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**| base32-encoded unique identifier | 

### Return type

[**models::Success**](success.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **auth_check**
> models::Success auth_check(ctx, optional)


Verify that authentication (API token) is working as expected. The optional `role` parameter can be used to verify that the current account (editor) has permissions for the given role. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **role** | **String**|  | 

### Return type

[**models::Success**](success.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **auth_oidc**
> models::AuthOidcResult auth_oidc(ctx, oidc_params)


Login or create editor account using OIDC metadata (internal method).  This method is used by privileged front-end tools (like the web interface service) to process editor logins using OpenID Connect (OIDC) and/or OAuth. Most accounts (including tool and bot accounts) do not have sufficient privileges to call this method, which requires the `admin` role.  The method returns an API token; the HTTP status code indicates whether an existing account was logged in, or a new account was created. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **oidc_params** | [**AuthOidc**](AuthOidc.md)|  | 

### Return type

[**models::AuthOidcResult**](auth_oidc_result.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_auth_token**
> models::AuthTokenResult create_auth_token(ctx, editor_id, optional)


Generate a new auth token for a given editor (internal method).  This method is used by the web interface to generate API tokens for users. It can not be called by editors (human or bot) to generate new tokens for themselves, at least at this time. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editor_id** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **editor_id** | **String**|  | 
 **duration_seconds** | **i32**| How long API token should be valid for (in seconds) | 

### Return type

[**models::AuthTokenResult**](auth_token_result.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_container**
> models::EntityEdit create_container(ctx, editgroup_id, entity)


Create a single Container entity as part of an existing editgroup.  Editgroup must be mutable (aka, not accepted) and editor must have permission (aka, have created the editgroup or have `admin` role). 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **entity** | [**ContainerEntity**](ContainerEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_container_auto_batch**
> models::Editgroup create_container_auto_batch(ctx, auto_batch)


Create a set of Container entities as part of a new editgroup, and accept that editgroup in the same atomic request.  This method is mostly useful for bulk import of new entities by carefully written bots. This method is more efficient than creating an editgroup, several entities, then accepting the editgroup, both in terms of API requests required and database load. Requires `admin` role. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **auto_batch** | [**ContainerAutoBatch**](ContainerAutoBatch.md)|  | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_creator**
> models::EntityEdit create_creator(ctx, editgroup_id, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **entity** | [**CreatorEntity**](CreatorEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_creator_auto_batch**
> models::Editgroup create_creator_auto_batch(ctx, auto_batch)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **auto_batch** | [**CreatorAutoBatch**](CreatorAutoBatch.md)|  | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_editgroup**
> models::Editgroup create_editgroup(ctx, editgroup)


Creates a new editgroup. By default the editor making this request will be the author of the editgroup. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup** | [**Editgroup**](Editgroup.md)|  | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_editgroup_annotation**
> models::EditgroupAnnotation create_editgroup_annotation(ctx, editgroup_id, annotation)


Submits a new annotation to the specified editgroup.  The editgroup must be open (not already accepted). The annotation will automatically have authorship of the editor making this request. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**| base32-encoded unique identifier | 
  **annotation** | [**EditgroupAnnotation**](EditgroupAnnotation.md)|  | 

### Return type

[**models::EditgroupAnnotation**](editgroup_annotation.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_file**
> models::EntityEdit create_file(ctx, editgroup_id, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **entity** | [**FileEntity**](FileEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_file_auto_batch**
> models::Editgroup create_file_auto_batch(ctx, auto_batch)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **auto_batch** | [**FileAutoBatch**](FileAutoBatch.md)|  | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_fileset**
> models::EntityEdit create_fileset(ctx, editgroup_id, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **entity** | [**FilesetEntity**](FilesetEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_fileset_auto_batch**
> models::Editgroup create_fileset_auto_batch(ctx, auto_batch)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **auto_batch** | [**FilesetAutoBatch**](FilesetAutoBatch.md)|  | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_release**
> models::EntityEdit create_release(ctx, editgroup_id, entity)


Create a single Release entity as part of an existing editgroup. If the `work_id` field is not set, a work entity will be created automatically and this field set pointing to the new work.  Editgroup must be mutable (aka, not accepted) and editor must have permission (aka, have created the editgrou p or have `admin` role). 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **entity** | [**ReleaseEntity**](ReleaseEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_release_auto_batch**
> models::Editgroup create_release_auto_batch(ctx, auto_batch)


Create a set of Release entities as part of a new editgroup, and accept that editgroup in the same atomic request.  This method is mostly useful for bulk import of new entities by carefully written bots. This method is more efficient than creating an editgroup, several entities, then accepting the editgroup, both in terms of API requests required and database load. Requires `admin` role. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **auto_batch** | [**ReleaseAutoBatch**](ReleaseAutoBatch.md)|  | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_webcapture**
> models::EntityEdit create_webcapture(ctx, editgroup_id, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **entity** | [**WebcaptureEntity**](WebcaptureEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_webcapture_auto_batch**
> models::Editgroup create_webcapture_auto_batch(ctx, auto_batch)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **auto_batch** | [**WebcaptureAutoBatch**](WebcaptureAutoBatch.md)|  | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_work**
> models::EntityEdit create_work(ctx, editgroup_id, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **entity** | [**WorkEntity**](WorkEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **create_work_auto_batch**
> models::Editgroup create_work_auto_batch(ctx, auto_batch)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **auto_batch** | [**WorkAutoBatch**](WorkAutoBatch.md)|  | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_container**
> models::EntityEdit delete_container(ctx, editgroup_id, ident)


Creates a new \"deletion\" edit for a specific entity as part of an existing editgroup.  This is not the method to use to remove an edit from an editgroup; for that use `delete_container_edit`. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_container_edit**
> models::Success delete_container_edit(ctx, editgroup_id, edit_id)


Removes a single edit from the specified editgroup. The editgroup must be mutable (aka, not accepted/merged), and the editor making this request must have permission (aka, have created the editgroup or hold the `admin` role).  Not to be confused with the `delete_container` method, which *creates* a new edit in the given editgroup. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::Success**](success.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_creator**
> models::EntityEdit delete_creator(ctx, editgroup_id, ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_creator_edit**
> models::Success delete_creator_edit(ctx, editgroup_id, edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::Success**](success.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_file**
> models::EntityEdit delete_file(ctx, editgroup_id, ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_file_edit**
> models::Success delete_file_edit(ctx, editgroup_id, edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::Success**](success.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_fileset**
> models::EntityEdit delete_fileset(ctx, editgroup_id, ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_fileset_edit**
> models::Success delete_fileset_edit(ctx, editgroup_id, edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::Success**](success.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_release**
> models::EntityEdit delete_release(ctx, editgroup_id, ident)


Creates a new \"deletion\" edit for a specific entity as part of an existing editgroup.  This is not the method to use to remove an edit from an editgroup; for that use `delete_release_edit`. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_release_edit**
> models::Success delete_release_edit(ctx, editgroup_id, edit_id)


Removes a single edit from the specified editgroup. The editgroup must be mutable (aka, not accepted/merged), and the editor making this request must have permission (aka, have created the editgroup or hold the `admin` role).  Not to be confused with the `delete_container` method, which *creates* a new edit in the given editgroup. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::Success**](success.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_webcapture**
> models::EntityEdit delete_webcapture(ctx, editgroup_id, ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_webcapture_edit**
> models::Success delete_webcapture_edit(ctx, editgroup_id, edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::Success**](success.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_work**
> models::EntityEdit delete_work(ctx, editgroup_id, ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_work_edit**
> models::Success delete_work_edit(ctx, editgroup_id, edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::Success**](success.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_changelog**
> Vec<models::ChangelogEntry> get_changelog(optional)


Returns a list of the most recent changelog entries accepted (merged) into the catalog.  List is sorted by changelog index in descending order. Note that the accepted timestamp roughly corresponds to order, but not strictly; there exist out-of-order timestamps on the order of several seconds.  As a known database issue, it is technically possible for there to be a gap in changelog index numbers (aka, a missing changelog entry). There are no currently known gaps and this is considered a bug that will be addressed.  There are millions of entries; to paginate through all of them, use this method to discover the highest existing entry number, then request the entries using `get_changelog_entry` one at a time. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **limit** | **i64**| Maximum count of changelog entries to return in response | 

### Return type

[**Vec<models::ChangelogEntry>**](changelog_entry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_changelog_entry**
> models::ChangelogEntry get_changelog_entry(index)


Returns a single changelog entry.  As a known database issue, it is technically possible for there to be a gap in changelog index numbers (aka, a missing changelog entry). There are no currently known gaps and this is considered a bug that will be addressed. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **index** | **i64**| Changelog index of entry to return | 

### Return type

[**models::ChangelogEntry**](changelog_entry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_container**
> models::ContainerEntity get_container(ident, optional)


Fetches a single container entity from the catalog. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. For containers, none accepted (yet). | 
 **hide** | **String**| List of entity fields to elide in response. For containers, none accepted (yet). | 

### Return type

[**models::ContainerEntity**](container_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_container_edit**
> models::EntityEdit get_container_edit(edit_id)


Returns the entity edit object with the given identifier. This method is expected to be used rarely. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_container_history**
> Vec<models::EntityHistoryEntry> get_container_history(ident, optional)


Fetches the history of accepted edits (changelog entries) for a specific entity fatcat identifier. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **limit** | **i64**| Maximum number of changelog entries to return | 

### Return type

[**Vec<models::EntityHistoryEntry>**](entity_history_entry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_container_redirects**
> Vec<String> get_container_redirects(ident)


Returns the set of entity identifiers which currently redirect to this identifier. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 

### Return type

[**Vec<String>**](string.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_container_revision**
> models::ContainerEntity get_container_revision(rev_id, optional)


Fetches a specific entity revision. Note that the returned revision will not be associated with any particular fatcat identifier (even if one or more identifiers do currently point to this revision). The revision may even be part of an un-merged editgroup.  Revisions are immutable and can not be deleted or updated. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_container`. | 
 **hide** | **String**| List of entity fields to elide in response. See `get_container`. | 

### Return type

[**models::ContainerEntity**](container_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_creator**
> models::CreatorEntity get_creator(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. For creators, none accepted (yet). | 
 **hide** | **String**| List of entity fields to elide in response. For creators, none accepted (yet). | 

### Return type

[**models::CreatorEntity**](creator_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_creator_edit**
> models::EntityEdit get_creator_edit(edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_creator_history**
> Vec<models::EntityHistoryEntry> get_creator_history(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **limit** | **i64**|  | 

### Return type

[**Vec<models::EntityHistoryEntry>**](entity_history_entry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_creator_redirects**
> Vec<String> get_creator_redirects(ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 

### Return type

[**Vec<String>**](string.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_creator_releases**
> Vec<models::ReleaseEntity> get_creator_releases(ident, optional)


Returns the set of Release entities having a `contrib` entry pointing to the creator entity. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **hide** | **String**| List of entity fields to elide in response. See `get_release`. | 

### Return type

[**Vec<models::ReleaseEntity>**](release_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_creator_revision**
> models::CreatorEntity get_creator_revision(rev_id, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_creator`, though note that identifier-based expansions (like `releases`) will always be empty for revisions.  | 
 **hide** | **String**| List of entity fields to elide in response. See `get_creator`. | 

### Return type

[**models::CreatorEntity**](creator_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_editgroup**
> models::Editgroup get_editgroup(editgroup_id)


Returns a single editgroup object.  Unlike some similar methods, this method will return a full/expanded editgroup object, which includes edit lists of each entity type (though will not include the complete entity objects). 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **editgroup_id** | **String**| base32-encoded unique identifier | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_editgroup_annotations**
> Vec<models::EditgroupAnnotation> get_editgroup_annotations(editgroup_id, optional)


Returns a list of annotations made to a specific editgroup.

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **editgroup_id** | **String**| base32-encoded unique identifier | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **editgroup_id** | **String**| base32-encoded unique identifier | 
 **expand** | **String**| List of sub-entities to expand in response. For editgroup annotations: 'editors' | 

### Return type

[**Vec<models::EditgroupAnnotation>**](editgroup_annotation.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_editgroups_reviewable**
> Vec<models::Editgroup> get_editgroups_reviewable(optional)


Returns a set of editgroups which have been submitted but not yet accepted.  Query parameters can be used to sort and paginate the list returned. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **expand** | **String**| List of sub-entities to expand in response. For editgroups: 'editors' | 
 **limit** | **i64**| Maximum number of reviewable editgroups to return in response | 
 **before** | **chrono::DateTime::<chrono::Utc>**| Return only reviewable editgroups submitted *before* the given timestamp (not inclusive). Editgroups will be sorted by submission time in descending order (most recent first). For use in pagination.  | 
 **since** | **chrono::DateTime::<chrono::Utc>**| Return only reviewable editgroups submitted *after* the given timestamp (not inclusive). Editgroups will be sorted by submission time in ascending order (most recent last). For use in pagination.  | 

### Return type

[**Vec<models::Editgroup>**](editgroup.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_editor**
> models::Editor get_editor(editor_id)


Returns an editor object, including metadata such as the username, type, and role of editor. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **editor_id** | **String**|  | 

### Return type

[**models::Editor**](editor.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_editor_annotations**
> Vec<models::EditgroupAnnotation> get_editor_annotations(editor_id, optional)


Fetches a list of annotations made by a particular editor. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **editor_id** | **String**| base32-encoded unique identifier | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **editor_id** | **String**| base32-encoded unique identifier | 
 **limit** | **i64**| Maximum number (count) of annotations to return in response | 
 **before** | **chrono::DateTime::<chrono::Utc>**| Return only annotations made *before* the given timestamp (not inclusive). Annotations will be sorted by creation time in descending order (most recent first). For use in pagination.  | 
 **since** | **chrono::DateTime::<chrono::Utc>**| Return only annotations made *after* the given timestamp (not inclusive). Annotations will be sorted by creation time in ascending order (most recent last). For use in pagination.  | 

### Return type

[**Vec<models::EditgroupAnnotation>**](editgroup_annotation.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_editor_editgroups**
> Vec<models::Editgroup> get_editor_editgroups(editor_id, optional)


Returns a set of editgroups created by the given editor, regardless of the status (accepted/submitted) of the editgroups. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **editor_id** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **editor_id** | **String**|  | 
 **limit** | **i64**|  | 
 **before** | **chrono::DateTime::<chrono::Utc>**| Return only editgroups created *before* the given timestamp (not inclusive). Editgroups will be sorted by creation time in descending order (most recent first). For use in pagination.  | 
 **since** | **chrono::DateTime::<chrono::Utc>**| Return only editgroups created *after* the given timestamp (not inclusive). Editgroups will be sorted by creation time in ascending order (most recent last). For use in pagination.  | 

### Return type

[**Vec<models::Editgroup>**](editgroup.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_file**
> models::FileEntity get_file(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. For files, `releases` is accepted. | 
 **hide** | **String**| List of entity fields to elide in response. For files, none accepted (yet). | 

### Return type

[**models::FileEntity**](file_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_file_edit**
> models::EntityEdit get_file_edit(edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_file_history**
> Vec<models::EntityHistoryEntry> get_file_history(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **limit** | **i64**|  | 

### Return type

[**Vec<models::EntityHistoryEntry>**](entity_history_entry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_file_redirects**
> Vec<String> get_file_redirects(ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 

### Return type

[**Vec<String>**](string.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_file_revision**
> models::FileEntity get_file_revision(rev_id, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_file`. | 
 **hide** | **String**| List of entity fields to elide in response. See `get_file`. | 

### Return type

[**models::FileEntity**](file_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_fileset**
> models::FilesetEntity get_fileset(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. For filesets, `releases` is accepted. | 
 **hide** | **String**| List of entity fields to elide in response. For filesets, 'manifest' is accepted. | 

### Return type

[**models::FilesetEntity**](fileset_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_fileset_edit**
> models::EntityEdit get_fileset_edit(edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_fileset_history**
> Vec<models::EntityHistoryEntry> get_fileset_history(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **limit** | **i64**|  | 

### Return type

[**Vec<models::EntityHistoryEntry>**](entity_history_entry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_fileset_redirects**
> Vec<String> get_fileset_redirects(ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 

### Return type

[**Vec<String>**](string.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_fileset_revision**
> models::FilesetEntity get_fileset_revision(rev_id, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_fileset`. | 
 **hide** | **String**| List of entity fields to elide in response. See `get_fileset`. | 

### Return type

[**models::FilesetEntity**](fileset_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_release**
> models::ReleaseEntity get_release(ident, optional)


Fetches a single Release entity from the catalog. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. For releases, 'files', 'filesets, 'webcaptures', 'container', and 'creators' are valid.  | 
 **hide** | **String**| List of entity fields to elide in response (for efficiency). For releases, 'abstracts', 'refs', and 'contribs' are valid.  | 

### Return type

[**models::ReleaseEntity**](release_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_release_edit**
> models::EntityEdit get_release_edit(edit_id)


Returns the entity edit object with the given identifier. This method is expected to be used rarely. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_release_files**
> Vec<models::FileEntity> get_release_files(ident, optional)


Returns the set of File entities that have a `release_id` pointing to this release entity. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **hide** | **String**| List of entity fields to elide in response. See `get_file`. | 

### Return type

[**Vec<models::FileEntity>**](file_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_release_filesets**
> Vec<models::FilesetEntity> get_release_filesets(ident, optional)


Returns the set of Fileset entities that have a `release_id` pointing to this release entity. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **hide** | **String**| List of entity fields to elide in response. See `get_fileset`. | 

### Return type

[**Vec<models::FilesetEntity>**](fileset_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_release_history**
> Vec<models::EntityHistoryEntry> get_release_history(ident, optional)


Fetches the history of accepted edits (changelog entries) for a specific entity fatcat identifier. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **limit** | **i64**|  | 

### Return type

[**Vec<models::EntityHistoryEntry>**](entity_history_entry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_release_redirects**
> Vec<String> get_release_redirects(ident)


Returns the set of entity identifiers which currently redirect to this identifier. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 

### Return type

[**Vec<String>**](string.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_release_revision**
> models::ReleaseEntity get_release_revision(rev_id, optional)


Fetches a specific entity revision. Note that the returned revision will not be associated with any particular fatcat identifier (even if one or more identifiers do currently point to this revision). The revision may even be part of an un-merged editgroup.  Revisions are immutable and can not be deleted or updated. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_release`, though note that identifier-based exapansions like `file` will always be empty for revisions. | 
 **hide** | **String**| List of entity fields to elide in response. See `get_release`. | 

### Return type

[**models::ReleaseEntity**](release_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_release_webcaptures**
> Vec<models::WebcaptureEntity> get_release_webcaptures(ident, optional)


Returns the set of Web Capture entities that have a `release_id` pointing to this release entity. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **hide** | **String**| List of entity fields to elide in response. See `get_webcapture`. | 

### Return type

[**Vec<models::WebcaptureEntity>**](webcapture_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_webcapture**
> models::WebcaptureEntity get_webcapture(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. For webcaptures, `releases` is accepted. | 
 **hide** | **String**| List of entity fields to elide in response. For webcaptures, 'cdx' is accepted. | 

### Return type

[**models::WebcaptureEntity**](webcapture_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_webcapture_edit**
> models::EntityEdit get_webcapture_edit(edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_webcapture_history**
> Vec<models::EntityHistoryEntry> get_webcapture_history(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **limit** | **i64**|  | 

### Return type

[**Vec<models::EntityHistoryEntry>**](entity_history_entry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_webcapture_redirects**
> Vec<String> get_webcapture_redirects(ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 

### Return type

[**Vec<String>**](string.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_webcapture_revision**
> models::WebcaptureEntity get_webcapture_revision(rev_id, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_webcapture`. | 
 **hide** | **String**| List of entity fields to elide in response. See `get_webcapture`. | 

### Return type

[**models::WebcaptureEntity**](webcapture_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_work**
> models::WorkEntity get_work(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. For works, none accepted (yet). | 
 **hide** | **String**| List of entity fields to elide in response. For works, none accepted (yet). | 

### Return type

[**models::WorkEntity**](work_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_work_edit**
> models::EntityEdit get_work_edit(edit_id)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **edit_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_work_history**
> Vec<models::EntityHistoryEntry> get_work_history(ident, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **limit** | **i64**|  | 

### Return type

[**Vec<models::EntityHistoryEntry>**](entity_history_entry.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_work_redirects**
> Vec<String> get_work_redirects(ident)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 

### Return type

[**Vec<String>**](string.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_work_releases**
> Vec<models::ReleaseEntity> get_work_releases(ident, optional)


Returns the set of release entities that are part of this work (aka, have `work_id` pointing to this work entity). 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **ident** | **String**|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ident** | **String**|  | 
 **hide** | **String**| List of entity fields to elide in response. See `get_release`. | 

### Return type

[**Vec<models::ReleaseEntity>**](release_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_work_revision**
> models::WorkEntity get_work_revision(rev_id, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **rev_id** | **String**| UUID (lower-case, dash-separated, hex-encoded 128-bit) | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_work`, though note that identifier-based expansions like `releases` will always be empty for revisions. | 
 **hide** | **String**| List of entity fields to elide in response. See `get_work`. | 

### Return type

[**models::WorkEntity**](work_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **lookup_container**
> models::ContainerEntity lookup_container(optional)


Looks for an active entity with the given external identifier. If any such entity is found, returns a single complete entity. If multiple entities have the same external identifier, this is considered a soft catalog error, and the behavior of which entity is returned is undefined.  One (and only one) external identifier should be specified per request. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **issnl** | **String**|  | 
 **wikidata_qid** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_container`. | 
 **hide** | **String**| List of entity fields to elide in response. See `get_container`. | 

### Return type

[**models::ContainerEntity**](container_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **lookup_creator**
> models::CreatorEntity lookup_creator(optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **orcid** | **String**| ORCiD (https://orcid.org) identifier | 
 **wikidata_qid** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_creator`. | 
 **hide** | **String**| List of entity fields to elide in response. See `get_creator`. | 

### Return type

[**models::CreatorEntity**](creator_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **lookup_file**
> models::FileEntity lookup_file(optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **md5** | **String**| MD5 hash of data, in hex encoding | 
 **sha1** | **String**| SHA-1 hash of data, in hex encoding | 
 **sha256** | **String**| SHA-256 hash of data, in hex encoding | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_file`. | 
 **hide** | **String**| List of entity fields to elide in response. See `get_file`. | 

### Return type

[**models::FileEntity**](file_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **lookup_release**
> models::ReleaseEntity lookup_release(optional)


Looks for an active entity with the given external identifier. If any such entity is found, returns a single complete entity. If multiple entities have the same external identifier, this is considered a soft catalog error, and the behavior of which entity is returned is undefined.  One (and only one) external identifier should be specified per request. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **doi** | **String**|  | 
 **wikidata_qid** | **String**|  | 
 **isbn13** | **String**|  | 
 **pmid** | **String**|  | 
 **pmcid** | **String**|  | 
 **core** | **String**|  | 
 **arxiv** | **String**|  | 
 **jstor** | **String**|  | 
 **ark** | **String**|  | 
 **mag** | **String**|  | 
 **expand** | **String**| List of sub-entities to expand in response. See `get_release`. | 
 **hide** | **String**| List of sub-entities to elide in response. See `get_release`. | 

### Return type

[**models::ReleaseEntity**](release_entity.md)

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_container**
> models::EntityEdit update_container(ctx, editgroup_id, ident, entity)


Updates an existing entity as part of a specific (existing) editgroup. The editgroup must be open for updates (aka, not accepted/merged), and the editor making the request must have permissions (aka, must have created the editgroup or have `admin` role).  This method can also be used to update an existing entity edit as part of an editgroup. For example, if an entity was created in this editgroup, an editor could make changes to the new entity's metadata before merging. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 
  **entity** | [**ContainerEntity**](ContainerEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_creator**
> models::EntityEdit update_creator(ctx, editgroup_id, ident, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 
  **entity** | [**CreatorEntity**](CreatorEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_editgroup**
> models::Editgroup update_editgroup(ctx, editgroup_id, editgroup, optional)


Updates metadata for a single editgroup object. Note that only metadata fields such as the `description` or `extra` metadata can be changed with this method; it does not allow adding or removing edits to the editgroup (for that use the individual entity create/update/delete methods). 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**| base32-encoded unique identifier | 
  **editgroup** | [**Editgroup**](Editgroup.md)|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **editgroup_id** | **String**| base32-encoded unique identifier | 
 **editgroup** | [**Editgroup**](Editgroup.md)|  | 
 **submit** | **bool**|  | 

### Return type

[**models::Editgroup**](editgroup.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_editor**
> models::Editor update_editor(ctx, editor_id, editor)


Allows metadata changes to some editor fields, such as the username.  Changes require authentication and permissions. An editor can change their own username; changes to role flags require the `admin` role by the editor making the request. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editor_id** | **String**|  | 
  **editor** | [**Editor**](Editor.md)|  | 

### Return type

[**models::Editor**](editor.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_file**
> models::EntityEdit update_file(ctx, editgroup_id, ident, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 
  **entity** | [**FileEntity**](FileEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_fileset**
> models::EntityEdit update_fileset(ctx, editgroup_id, ident, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 
  **entity** | [**FilesetEntity**](FilesetEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_release**
> models::EntityEdit update_release(ctx, editgroup_id, ident, entity)


Updates an existing entity as part of a specific (existing) editgroup. The editgroup must be open for updates (aka, not accepted/merged), and the editor making the requiest must have permissions (aka, must have created the editgroup or have `admin` role).  This method can also be used to update an existing entity edit as part of an editgroup. For example, if an entity was created in this editgroup, an editor could make changes to the new entity's metadata before merging. 

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 
  **entity** | [**ReleaseEntity**](ReleaseEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_webcapture**
> models::EntityEdit update_webcapture(ctx, editgroup_id, ident, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 
  **entity** | [**WebcaptureEntity**](WebcaptureEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, 

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_work**
> models::EntityEdit update_work(ctx, editgroup_id, ident, entity)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **editgroup_id** | **String**|  | 
  **ident** | **String**|  | 
  **entity** | [**WorkEntity**](WorkEntity.md)|  | 

### Return type

[**models::EntityEdit**](entity_edit.md)

### Authorization

[Bearer](../README.md#Bearer)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

