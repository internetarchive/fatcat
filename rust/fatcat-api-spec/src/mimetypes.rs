/// mime types for requests and responses

pub mod responses {
    use hyper::mime::*;

    // The macro is called per-operation to beat the recursion limit
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainer
    lazy_static! {
        pub static ref DELETE_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainerEdit
    lazy_static! {
        pub static ref DELETE_CONTAINER_EDIT_DELETED_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainerEdit
    lazy_static! {
        pub static ref DELETE_CONTAINER_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainerEdit
    lazy_static! {
        pub static ref DELETE_CONTAINER_EDIT_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainerEdit
    lazy_static! {
        pub static ref DELETE_CONTAINER_EDIT_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainerEdit
    lazy_static! {
        pub static ref DELETE_CONTAINER_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteContainerEdit
    lazy_static! {
        pub static ref DELETE_CONTAINER_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainer
    lazy_static! {
        pub static ref GET_CONTAINER_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainer
    lazy_static! {
        pub static ref GET_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainer
    lazy_static! {
        pub static ref GET_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainer
    lazy_static! {
        pub static ref GET_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerEdit
    lazy_static! {
        pub static ref GET_CONTAINER_EDIT_FOUND_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerEdit
    lazy_static! {
        pub static ref GET_CONTAINER_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerEdit
    lazy_static! {
        pub static ref GET_CONTAINER_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerEdit
    lazy_static! {
        pub static ref GET_CONTAINER_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerHistory
    lazy_static! {
        pub static ref GET_CONTAINER_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerHistory
    lazy_static! {
        pub static ref GET_CONTAINER_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerHistory
    lazy_static! {
        pub static ref GET_CONTAINER_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerHistory
    lazy_static! {
        pub static ref GET_CONTAINER_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerRedirects
    lazy_static! {
        pub static ref GET_CONTAINER_REDIRECTS_FOUND_ENTITY_REDIRECTS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerRedirects
    lazy_static! {
        pub static ref GET_CONTAINER_REDIRECTS_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerRedirects
    lazy_static! {
        pub static ref GET_CONTAINER_REDIRECTS_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerRedirects
    lazy_static! {
        pub static ref GET_CONTAINER_REDIRECTS_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerRevision
    lazy_static! {
        pub static ref GET_CONTAINER_REVISION_FOUND_ENTITY_REVISION: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerRevision
    lazy_static! {
        pub static ref GET_CONTAINER_REVISION_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerRevision
    lazy_static! {
        pub static ref GET_CONTAINER_REVISION_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetContainerRevision
    lazy_static! {
        pub static ref GET_CONTAINER_REVISION_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupContainer
    lazy_static! {
        pub static ref LOOKUP_CONTAINER_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupContainer
    lazy_static! {
        pub static ref LOOKUP_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupContainer
    lazy_static! {
        pub static ref LOOKUP_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupContainer
    lazy_static! {
        pub static ref LOOKUP_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreator
    lazy_static! {
        pub static ref DELETE_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreatorEdit
    lazy_static! {
        pub static ref DELETE_CREATOR_EDIT_DELETED_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreatorEdit
    lazy_static! {
        pub static ref DELETE_CREATOR_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreatorEdit
    lazy_static! {
        pub static ref DELETE_CREATOR_EDIT_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreatorEdit
    lazy_static! {
        pub static ref DELETE_CREATOR_EDIT_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreatorEdit
    lazy_static! {
        pub static ref DELETE_CREATOR_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteCreatorEdit
    lazy_static! {
        pub static ref DELETE_CREATOR_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreator
    lazy_static! {
        pub static ref GET_CREATOR_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreator
    lazy_static! {
        pub static ref GET_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreator
    lazy_static! {
        pub static ref GET_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreator
    lazy_static! {
        pub static ref GET_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorEdit
    lazy_static! {
        pub static ref GET_CREATOR_EDIT_FOUND_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorEdit
    lazy_static! {
        pub static ref GET_CREATOR_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorEdit
    lazy_static! {
        pub static ref GET_CREATOR_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorEdit
    lazy_static! {
        pub static ref GET_CREATOR_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorHistory
    lazy_static! {
        pub static ref GET_CREATOR_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorHistory
    lazy_static! {
        pub static ref GET_CREATOR_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorHistory
    lazy_static! {
        pub static ref GET_CREATOR_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorHistory
    lazy_static! {
        pub static ref GET_CREATOR_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorRedirects
    lazy_static! {
        pub static ref GET_CREATOR_REDIRECTS_FOUND_ENTITY_REDIRECTS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorRedirects
    lazy_static! {
        pub static ref GET_CREATOR_REDIRECTS_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorRedirects
    lazy_static! {
        pub static ref GET_CREATOR_REDIRECTS_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorRedirects
    lazy_static! {
        pub static ref GET_CREATOR_REDIRECTS_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorReleases
    lazy_static! {
        pub static ref GET_CREATOR_RELEASES_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorReleases
    lazy_static! {
        pub static ref GET_CREATOR_RELEASES_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorReleases
    lazy_static! {
        pub static ref GET_CREATOR_RELEASES_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorReleases
    lazy_static! {
        pub static ref GET_CREATOR_RELEASES_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorRevision
    lazy_static! {
        pub static ref GET_CREATOR_REVISION_FOUND_ENTITY_REVISION: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorRevision
    lazy_static! {
        pub static ref GET_CREATOR_REVISION_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorRevision
    lazy_static! {
        pub static ref GET_CREATOR_REVISION_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetCreatorRevision
    lazy_static! {
        pub static ref GET_CREATOR_REVISION_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupCreator
    lazy_static! {
        pub static ref LOOKUP_CREATOR_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupCreator
    lazy_static! {
        pub static ref LOOKUP_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupCreator
    lazy_static! {
        pub static ref LOOKUP_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupCreator
    lazy_static! {
        pub static ref LOOKUP_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthCheck
    lazy_static! {
        pub static ref AUTH_CHECK_SUCCESS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthCheck
    lazy_static! {
        pub static ref AUTH_CHECK_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthCheck
    lazy_static! {
        pub static ref AUTH_CHECK_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthCheck
    lazy_static! {
        pub static ref AUTH_CHECK_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthCheck
    lazy_static! {
        pub static ref AUTH_CHECK_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthOidc
    lazy_static! {
        pub static ref AUTH_OIDC_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthOidc
    lazy_static! {
        pub static ref AUTH_OIDC_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthOidc
    lazy_static! {
        pub static ref AUTH_OIDC_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthOidc
    lazy_static! {
        pub static ref AUTH_OIDC_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthOidc
    lazy_static! {
        pub static ref AUTH_OIDC_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthOidc
    lazy_static! {
        pub static ref AUTH_OIDC_CONFLICT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AuthOidc
    lazy_static! {
        pub static ref AUTH_OIDC_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditor
    lazy_static! {
        pub static ref GET_EDITOR_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditor
    lazy_static! {
        pub static ref GET_EDITOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditor
    lazy_static! {
        pub static ref GET_EDITOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditor
    lazy_static! {
        pub static ref GET_EDITOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditorChangelog
    lazy_static! {
        pub static ref GET_EDITOR_CHANGELOG_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditorChangelog
    lazy_static! {
        pub static ref GET_EDITOR_CHANGELOG_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditorChangelog
    lazy_static! {
        pub static ref GET_EDITOR_CHANGELOG_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditorChangelog
    lazy_static! {
        pub static ref GET_EDITOR_CHANGELOG_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateEditor
    lazy_static! {
        pub static ref UPDATE_EDITOR_UPDATED_EDITOR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateEditor
    lazy_static! {
        pub static ref UPDATE_EDITOR_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateEditor
    lazy_static! {
        pub static ref UPDATE_EDITOR_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateEditor
    lazy_static! {
        pub static ref UPDATE_EDITOR_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateEditor
    lazy_static! {
        pub static ref UPDATE_EDITOR_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateEditor
    lazy_static! {
        pub static ref UPDATE_EDITOR_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_MERGED_SUCCESSFULLY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_EDIT_CONFLICT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for AcceptEditgroup
    lazy_static! {
        pub static ref ACCEPT_EDITGROUP_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP_SUCCESSFULLY_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelog
    lazy_static! {
        pub static ref GET_CHANGELOG_SUCCESS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelog
    lazy_static! {
        pub static ref GET_CHANGELOG_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelogEntry
    lazy_static! {
        pub static ref GET_CHANGELOG_ENTRY_FOUND_CHANGELOG_ENTRY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelogEntry
    lazy_static! {
        pub static ref GET_CHANGELOG_ENTRY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetChangelogEntry
    lazy_static! {
        pub static ref GET_CHANGELOG_ENTRY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditgroup
    lazy_static! {
        pub static ref GET_EDITGROUP_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditgroup
    lazy_static! {
        pub static ref GET_EDITGROUP_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditgroup
    lazy_static! {
        pub static ref GET_EDITGROUP_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetEditgroup
    lazy_static! {
        pub static ref GET_EDITGROUP_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFile
    lazy_static! {
        pub static ref DELETE_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileEdit
    lazy_static! {
        pub static ref DELETE_FILE_EDIT_DELETED_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileEdit
    lazy_static! {
        pub static ref DELETE_FILE_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileEdit
    lazy_static! {
        pub static ref DELETE_FILE_EDIT_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileEdit
    lazy_static! {
        pub static ref DELETE_FILE_EDIT_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileEdit
    lazy_static! {
        pub static ref DELETE_FILE_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileEdit
    lazy_static! {
        pub static ref DELETE_FILE_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFile
    lazy_static! {
        pub static ref GET_FILE_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFile
    lazy_static! {
        pub static ref GET_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFile
    lazy_static! {
        pub static ref GET_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFile
    lazy_static! {
        pub static ref GET_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileEdit
    lazy_static! {
        pub static ref GET_FILE_EDIT_FOUND_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileEdit
    lazy_static! {
        pub static ref GET_FILE_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileEdit
    lazy_static! {
        pub static ref GET_FILE_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileEdit
    lazy_static! {
        pub static ref GET_FILE_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileHistory
    lazy_static! {
        pub static ref GET_FILE_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileHistory
    lazy_static! {
        pub static ref GET_FILE_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileHistory
    lazy_static! {
        pub static ref GET_FILE_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileHistory
    lazy_static! {
        pub static ref GET_FILE_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileRedirects
    lazy_static! {
        pub static ref GET_FILE_REDIRECTS_FOUND_ENTITY_REDIRECTS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileRedirects
    lazy_static! {
        pub static ref GET_FILE_REDIRECTS_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileRedirects
    lazy_static! {
        pub static ref GET_FILE_REDIRECTS_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileRedirects
    lazy_static! {
        pub static ref GET_FILE_REDIRECTS_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileRevision
    lazy_static! {
        pub static ref GET_FILE_REVISION_FOUND_ENTITY_REVISION: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileRevision
    lazy_static! {
        pub static ref GET_FILE_REVISION_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileRevision
    lazy_static! {
        pub static ref GET_FILE_REVISION_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileRevision
    lazy_static! {
        pub static ref GET_FILE_REVISION_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupFile
    lazy_static! {
        pub static ref LOOKUP_FILE_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupFile
    lazy_static! {
        pub static ref LOOKUP_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupFile
    lazy_static! {
        pub static ref LOOKUP_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupFile
    lazy_static! {
        pub static ref LOOKUP_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileset
    lazy_static! {
        pub static ref CREATE_FILESET_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileset
    lazy_static! {
        pub static ref CREATE_FILESET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileset
    lazy_static! {
        pub static ref CREATE_FILESET_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileset
    lazy_static! {
        pub static ref CREATE_FILESET_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileset
    lazy_static! {
        pub static ref CREATE_FILESET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFileset
    lazy_static! {
        pub static ref CREATE_FILESET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFilesetBatch
    lazy_static! {
        pub static ref CREATE_FILESET_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFilesetBatch
    lazy_static! {
        pub static ref CREATE_FILESET_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFilesetBatch
    lazy_static! {
        pub static ref CREATE_FILESET_BATCH_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFilesetBatch
    lazy_static! {
        pub static ref CREATE_FILESET_BATCH_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFilesetBatch
    lazy_static! {
        pub static ref CREATE_FILESET_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateFilesetBatch
    lazy_static! {
        pub static ref CREATE_FILESET_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileset
    lazy_static! {
        pub static ref DELETE_FILESET_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileset
    lazy_static! {
        pub static ref DELETE_FILESET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileset
    lazy_static! {
        pub static ref DELETE_FILESET_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileset
    lazy_static! {
        pub static ref DELETE_FILESET_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileset
    lazy_static! {
        pub static ref DELETE_FILESET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFileset
    lazy_static! {
        pub static ref DELETE_FILESET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFilesetEdit
    lazy_static! {
        pub static ref DELETE_FILESET_EDIT_DELETED_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFilesetEdit
    lazy_static! {
        pub static ref DELETE_FILESET_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFilesetEdit
    lazy_static! {
        pub static ref DELETE_FILESET_EDIT_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFilesetEdit
    lazy_static! {
        pub static ref DELETE_FILESET_EDIT_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFilesetEdit
    lazy_static! {
        pub static ref DELETE_FILESET_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteFilesetEdit
    lazy_static! {
        pub static ref DELETE_FILESET_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileset
    lazy_static! {
        pub static ref GET_FILESET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileset
    lazy_static! {
        pub static ref GET_FILESET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileset
    lazy_static! {
        pub static ref GET_FILESET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFileset
    lazy_static! {
        pub static ref GET_FILESET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetEdit
    lazy_static! {
        pub static ref GET_FILESET_EDIT_FOUND_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetEdit
    lazy_static! {
        pub static ref GET_FILESET_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetEdit
    lazy_static! {
        pub static ref GET_FILESET_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetEdit
    lazy_static! {
        pub static ref GET_FILESET_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetHistory
    lazy_static! {
        pub static ref GET_FILESET_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetHistory
    lazy_static! {
        pub static ref GET_FILESET_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetHistory
    lazy_static! {
        pub static ref GET_FILESET_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetHistory
    lazy_static! {
        pub static ref GET_FILESET_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetRedirects
    lazy_static! {
        pub static ref GET_FILESET_REDIRECTS_FOUND_ENTITY_REDIRECTS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetRedirects
    lazy_static! {
        pub static ref GET_FILESET_REDIRECTS_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetRedirects
    lazy_static! {
        pub static ref GET_FILESET_REDIRECTS_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetRedirects
    lazy_static! {
        pub static ref GET_FILESET_REDIRECTS_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetRevision
    lazy_static! {
        pub static ref GET_FILESET_REVISION_FOUND_ENTITY_REVISION: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetRevision
    lazy_static! {
        pub static ref GET_FILESET_REVISION_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetRevision
    lazy_static! {
        pub static ref GET_FILESET_REVISION_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetFilesetRevision
    lazy_static! {
        pub static ref GET_FILESET_REVISION_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFileset
    lazy_static! {
        pub static ref UPDATE_FILESET_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFileset
    lazy_static! {
        pub static ref UPDATE_FILESET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFileset
    lazy_static! {
        pub static ref UPDATE_FILESET_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFileset
    lazy_static! {
        pub static ref UPDATE_FILESET_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFileset
    lazy_static! {
        pub static ref UPDATE_FILESET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateFileset
    lazy_static! {
        pub static ref UPDATE_FILESET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteRelease
    lazy_static! {
        pub static ref DELETE_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteReleaseEdit
    lazy_static! {
        pub static ref DELETE_RELEASE_EDIT_DELETED_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteReleaseEdit
    lazy_static! {
        pub static ref DELETE_RELEASE_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteReleaseEdit
    lazy_static! {
        pub static ref DELETE_RELEASE_EDIT_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteReleaseEdit
    lazy_static! {
        pub static ref DELETE_RELEASE_EDIT_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteReleaseEdit
    lazy_static! {
        pub static ref DELETE_RELEASE_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteReleaseEdit
    lazy_static! {
        pub static ref DELETE_RELEASE_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetRelease
    lazy_static! {
        pub static ref GET_RELEASE_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetRelease
    lazy_static! {
        pub static ref GET_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetRelease
    lazy_static! {
        pub static ref GET_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetRelease
    lazy_static! {
        pub static ref GET_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseEdit
    lazy_static! {
        pub static ref GET_RELEASE_EDIT_FOUND_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseEdit
    lazy_static! {
        pub static ref GET_RELEASE_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseEdit
    lazy_static! {
        pub static ref GET_RELEASE_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseEdit
    lazy_static! {
        pub static ref GET_RELEASE_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFiles
    lazy_static! {
        pub static ref GET_RELEASE_FILES_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFiles
    lazy_static! {
        pub static ref GET_RELEASE_FILES_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFiles
    lazy_static! {
        pub static ref GET_RELEASE_FILES_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFiles
    lazy_static! {
        pub static ref GET_RELEASE_FILES_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFilesets
    lazy_static! {
        pub static ref GET_RELEASE_FILESETS_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFilesets
    lazy_static! {
        pub static ref GET_RELEASE_FILESETS_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFilesets
    lazy_static! {
        pub static ref GET_RELEASE_FILESETS_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseFilesets
    lazy_static! {
        pub static ref GET_RELEASE_FILESETS_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseHistory
    lazy_static! {
        pub static ref GET_RELEASE_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseHistory
    lazy_static! {
        pub static ref GET_RELEASE_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseHistory
    lazy_static! {
        pub static ref GET_RELEASE_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseHistory
    lazy_static! {
        pub static ref GET_RELEASE_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseRedirects
    lazy_static! {
        pub static ref GET_RELEASE_REDIRECTS_FOUND_ENTITY_REDIRECTS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseRedirects
    lazy_static! {
        pub static ref GET_RELEASE_REDIRECTS_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseRedirects
    lazy_static! {
        pub static ref GET_RELEASE_REDIRECTS_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseRedirects
    lazy_static! {
        pub static ref GET_RELEASE_REDIRECTS_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseRevision
    lazy_static! {
        pub static ref GET_RELEASE_REVISION_FOUND_ENTITY_REVISION: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseRevision
    lazy_static! {
        pub static ref GET_RELEASE_REVISION_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseRevision
    lazy_static! {
        pub static ref GET_RELEASE_REVISION_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseRevision
    lazy_static! {
        pub static ref GET_RELEASE_REVISION_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseWebcaptures
    lazy_static! {
        pub static ref GET_RELEASE_WEBCAPTURES_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseWebcaptures
    lazy_static! {
        pub static ref GET_RELEASE_WEBCAPTURES_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseWebcaptures
    lazy_static! {
        pub static ref GET_RELEASE_WEBCAPTURES_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetReleaseWebcaptures
    lazy_static! {
        pub static ref GET_RELEASE_WEBCAPTURES_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupRelease
    lazy_static! {
        pub static ref LOOKUP_RELEASE_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupRelease
    lazy_static! {
        pub static ref LOOKUP_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupRelease
    lazy_static! {
        pub static ref LOOKUP_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for LookupRelease
    lazy_static! {
        pub static ref LOOKUP_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcapture
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcapture
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcapture
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcapture
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcapture
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcapture
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcaptureBatch
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcaptureBatch
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcaptureBatch
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_BATCH_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcaptureBatch
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_BATCH_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcaptureBatch
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWebcaptureBatch
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcapture
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcapture
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcapture
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcapture
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcapture
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcapture
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcaptureEdit
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_EDIT_DELETED_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcaptureEdit
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcaptureEdit
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_EDIT_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcaptureEdit
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_EDIT_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcaptureEdit
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWebcaptureEdit
    lazy_static! {
        pub static ref DELETE_WEBCAPTURE_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcapture
    lazy_static! {
        pub static ref GET_WEBCAPTURE_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcapture
    lazy_static! {
        pub static ref GET_WEBCAPTURE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcapture
    lazy_static! {
        pub static ref GET_WEBCAPTURE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcapture
    lazy_static! {
        pub static ref GET_WEBCAPTURE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureEdit
    lazy_static! {
        pub static ref GET_WEBCAPTURE_EDIT_FOUND_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureEdit
    lazy_static! {
        pub static ref GET_WEBCAPTURE_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureEdit
    lazy_static! {
        pub static ref GET_WEBCAPTURE_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureEdit
    lazy_static! {
        pub static ref GET_WEBCAPTURE_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureHistory
    lazy_static! {
        pub static ref GET_WEBCAPTURE_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureHistory
    lazy_static! {
        pub static ref GET_WEBCAPTURE_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureHistory
    lazy_static! {
        pub static ref GET_WEBCAPTURE_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureHistory
    lazy_static! {
        pub static ref GET_WEBCAPTURE_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureRedirects
    lazy_static! {
        pub static ref GET_WEBCAPTURE_REDIRECTS_FOUND_ENTITY_REDIRECTS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureRedirects
    lazy_static! {
        pub static ref GET_WEBCAPTURE_REDIRECTS_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureRedirects
    lazy_static! {
        pub static ref GET_WEBCAPTURE_REDIRECTS_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureRedirects
    lazy_static! {
        pub static ref GET_WEBCAPTURE_REDIRECTS_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureRevision
    lazy_static! {
        pub static ref GET_WEBCAPTURE_REVISION_FOUND_ENTITY_REVISION: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureRevision
    lazy_static! {
        pub static ref GET_WEBCAPTURE_REVISION_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureRevision
    lazy_static! {
        pub static ref GET_WEBCAPTURE_REVISION_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWebcaptureRevision
    lazy_static! {
        pub static ref GET_WEBCAPTURE_REVISION_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWebcapture
    lazy_static! {
        pub static ref UPDATE_WEBCAPTURE_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWebcapture
    lazy_static! {
        pub static ref UPDATE_WEBCAPTURE_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWebcapture
    lazy_static! {
        pub static ref UPDATE_WEBCAPTURE_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWebcapture
    lazy_static! {
        pub static ref UPDATE_WEBCAPTURE_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWebcapture
    lazy_static! {
        pub static ref UPDATE_WEBCAPTURE_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWebcapture
    lazy_static! {
        pub static ref UPDATE_WEBCAPTURE_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_CREATED_ENTITIES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_DELETED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWork
    lazy_static! {
        pub static ref DELETE_WORK_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWorkEdit
    lazy_static! {
        pub static ref DELETE_WORK_EDIT_DELETED_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWorkEdit
    lazy_static! {
        pub static ref DELETE_WORK_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWorkEdit
    lazy_static! {
        pub static ref DELETE_WORK_EDIT_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWorkEdit
    lazy_static! {
        pub static ref DELETE_WORK_EDIT_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWorkEdit
    lazy_static! {
        pub static ref DELETE_WORK_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for DeleteWorkEdit
    lazy_static! {
        pub static ref DELETE_WORK_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWork
    lazy_static! {
        pub static ref GET_WORK_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWork
    lazy_static! {
        pub static ref GET_WORK_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWork
    lazy_static! {
        pub static ref GET_WORK_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWork
    lazy_static! {
        pub static ref GET_WORK_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkEdit
    lazy_static! {
        pub static ref GET_WORK_EDIT_FOUND_EDIT: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkEdit
    lazy_static! {
        pub static ref GET_WORK_EDIT_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkEdit
    lazy_static! {
        pub static ref GET_WORK_EDIT_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkEdit
    lazy_static! {
        pub static ref GET_WORK_EDIT_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkHistory
    lazy_static! {
        pub static ref GET_WORK_HISTORY_FOUND_ENTITY_HISTORY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkHistory
    lazy_static! {
        pub static ref GET_WORK_HISTORY_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkHistory
    lazy_static! {
        pub static ref GET_WORK_HISTORY_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkHistory
    lazy_static! {
        pub static ref GET_WORK_HISTORY_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkRedirects
    lazy_static! {
        pub static ref GET_WORK_REDIRECTS_FOUND_ENTITY_REDIRECTS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkRedirects
    lazy_static! {
        pub static ref GET_WORK_REDIRECTS_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkRedirects
    lazy_static! {
        pub static ref GET_WORK_REDIRECTS_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkRedirects
    lazy_static! {
        pub static ref GET_WORK_REDIRECTS_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkReleases
    lazy_static! {
        pub static ref GET_WORK_RELEASES_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkReleases
    lazy_static! {
        pub static ref GET_WORK_RELEASES_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkReleases
    lazy_static! {
        pub static ref GET_WORK_RELEASES_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkReleases
    lazy_static! {
        pub static ref GET_WORK_RELEASES_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkRevision
    lazy_static! {
        pub static ref GET_WORK_REVISION_FOUND_ENTITY_REVISION: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkRevision
    lazy_static! {
        pub static ref GET_WORK_REVISION_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkRevision
    lazy_static! {
        pub static ref GET_WORK_REVISION_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for GetWorkRevision
    lazy_static! {
        pub static ref GET_WORK_REVISION_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_UPDATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_NOT_AUTHORIZED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_FORBIDDEN: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK_GENERIC_ERROR: Mime = mime!(Application / Json);
    }

}

pub mod requests {
    use hyper::mime::*;
    /// Create Mime objects for the request content types for CreateContainer
    lazy_static! {
        pub static ref CREATE_CONTAINER: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateContainerBatch
    lazy_static! {
        pub static ref CREATE_CONTAINER_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateContainer
    lazy_static! {
        pub static ref UPDATE_CONTAINER: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateCreator
    lazy_static! {
        pub static ref CREATE_CREATOR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateCreatorBatch
    lazy_static! {
        pub static ref CREATE_CREATOR_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateCreator
    lazy_static! {
        pub static ref UPDATE_CREATOR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for AuthOidc
    lazy_static! {
        pub static ref AUTH_OIDC: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateEditor
    lazy_static! {
        pub static ref UPDATE_EDITOR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateEditgroup
    lazy_static! {
        pub static ref CREATE_EDITGROUP: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateFile
    lazy_static! {
        pub static ref CREATE_FILE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateFileBatch
    lazy_static! {
        pub static ref CREATE_FILE_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateFile
    lazy_static! {
        pub static ref UPDATE_FILE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateFileset
    lazy_static! {
        pub static ref CREATE_FILESET: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateFilesetBatch
    lazy_static! {
        pub static ref CREATE_FILESET_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateFileset
    lazy_static! {
        pub static ref UPDATE_FILESET: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateRelease
    lazy_static! {
        pub static ref CREATE_RELEASE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateReleaseBatch
    lazy_static! {
        pub static ref CREATE_RELEASE_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateWork
    lazy_static! {
        pub static ref CREATE_WORK: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateRelease
    lazy_static! {
        pub static ref UPDATE_RELEASE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateWebcapture
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateWebcaptureBatch
    lazy_static! {
        pub static ref CREATE_WEBCAPTURE_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateWebcapture
    lazy_static! {
        pub static ref UPDATE_WEBCAPTURE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreateWorkBatch
    lazy_static! {
        pub static ref CREATE_WORK_BATCH: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for UpdateWork
    lazy_static! {
        pub static ref UPDATE_WORK: Mime = mime!(Application / Json);
    }

}
