/// mime types for requests and responses

pub mod responses {
    use hyper::mime::*;

    // The macro is called per-operation to beat the recursion limit
    /// Create Mime objects for the response content types for ContainerIdGet
    lazy_static! {
        pub static ref CONTAINER_ID_GET_FETCH_A_SINGLE_CONTAINER_BY_ID: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerIdGet
    lazy_static! {
        pub static ref CONTAINER_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerIdGet
    lazy_static! {
        pub static ref CONTAINER_ID_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerLookupGet
    lazy_static! {
        pub static ref CONTAINER_LOOKUP_GET_FIND_A_SINGLE_CONTAINER_BY_EXTERNAL_IDENTIFER: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerLookupGet
    lazy_static! {
        pub static ref CONTAINER_LOOKUP_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerLookupGet
    lazy_static! {
        pub static ref CONTAINER_LOOKUP_GET_NO_SUCH_CONTAINER: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerLookupGet
    lazy_static! {
        pub static ref CONTAINER_LOOKUP_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerPost
    lazy_static! {
        pub static ref CONTAINER_POST_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerPost
    lazy_static! {
        pub static ref CONTAINER_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerPost
    lazy_static! {
        pub static ref CONTAINER_POST_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorIdGet
    lazy_static! {
        pub static ref CREATOR_ID_GET_FETCH_A_SINGLE_CREATOR_BY_ID: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorIdGet
    lazy_static! {
        pub static ref CREATOR_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorIdGet
    lazy_static! {
        pub static ref CREATOR_ID_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorLookupGet
    lazy_static! {
        pub static ref CREATOR_LOOKUP_GET_FIND_A_SINGLE_CREATOR_BY_EXTERNAL_IDENTIFER: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorLookupGet
    lazy_static! {
        pub static ref CREATOR_LOOKUP_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorLookupGet
    lazy_static! {
        pub static ref CREATOR_LOOKUP_GET_NO_SUCH_CREATOR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorLookupGet
    lazy_static! {
        pub static ref CREATOR_LOOKUP_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorPost
    lazy_static! {
        pub static ref CREATOR_POST_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorPost
    lazy_static! {
        pub static ref CREATOR_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorPost
    lazy_static! {
        pub static ref CREATOR_POST_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdAcceptPost
    lazy_static! {
        pub static ref EDITGROUP_ID_ACCEPT_POST_MERGED_EDITGROUP_SUCCESSFULLY_: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdAcceptPost
    lazy_static! {
        pub static ref EDITGROUP_ID_ACCEPT_POST_EDITGROUP_IS_IN_AN_UNMERGABLE_STATE: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdAcceptPost
    lazy_static! {
        pub static ref EDITGROUP_ID_ACCEPT_POST_NO_SUCH_EDITGROUP: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdAcceptPost
    lazy_static! {
        pub static ref EDITGROUP_ID_ACCEPT_POST_GENERIC_ERROR_RESPONSE: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdGet
    lazy_static! {
        pub static ref EDITGROUP_ID_GET_FETCH_EDITGROUP_BY_IDENTIFIER: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdGet
    lazy_static! {
        pub static ref EDITGROUP_ID_GET_NO_SUCH_EDITGROUP: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdGet
    lazy_static! {
        pub static ref EDITGROUP_ID_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupPost
    lazy_static! {
        pub static ref EDITGROUP_POST_SUCCESSFULLY_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupPost
    lazy_static! {
        pub static ref EDITGROUP_POST_INVALID_REQUEST_PARAMETERS: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupPost
    lazy_static! {
        pub static ref EDITGROUP_POST_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameChangelogGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_CHANGELOG_GET_FIND_CHANGES_: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameChangelogGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_CHANGELOG_GET_USERNAME_NOT_FOUND: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameChangelogGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_CHANGELOG_GET_GENERIC_ERROR_RESPONSE: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_GET_FETCH_GENERIC_INFORMATION_ABOUT_AN_EDITOR: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_GET_USERNAME_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileIdGet
    lazy_static! {
        pub static ref FILE_ID_GET_FETCH_A_SINGLE_FILE_BY_ID: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileIdGet
    lazy_static! {
        pub static ref FILE_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileIdGet
    lazy_static! {
        pub static ref FILE_ID_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileLookupGet
    lazy_static! {
        pub static ref FILE_LOOKUP_GET_FIND_A_SINGLE_FILE_BY_EXTERNAL_IDENTIFER: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileLookupGet
    lazy_static! {
        pub static ref FILE_LOOKUP_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileLookupGet
    lazy_static! {
        pub static ref FILE_LOOKUP_GET_NO_SUCH_FILE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileLookupGet
    lazy_static! {
        pub static ref FILE_LOOKUP_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FilePost
    lazy_static! {
        pub static ref FILE_POST_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FilePost
    lazy_static! {
        pub static ref FILE_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FilePost
    lazy_static! {
        pub static ref FILE_POST_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseIdGet
    lazy_static! {
        pub static ref RELEASE_ID_GET_FETCH_A_SINGLE_RELEASE_BY_ID: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseIdGet
    lazy_static! {
        pub static ref RELEASE_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseIdGet
    lazy_static! {
        pub static ref RELEASE_ID_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseLookupGet
    lazy_static! {
        pub static ref RELEASE_LOOKUP_GET_FIND_A_SINGLE_RELEASE_BY_EXTERNAL_IDENTIFER: Mime =
            mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseLookupGet
    lazy_static! {
        pub static ref RELEASE_LOOKUP_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseLookupGet
    lazy_static! {
        pub static ref RELEASE_LOOKUP_GET_NO_SUCH_RELEASE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseLookupGet
    lazy_static! {
        pub static ref RELEASE_LOOKUP_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleasePost
    lazy_static! {
        pub static ref RELEASE_POST_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleasePost
    lazy_static! {
        pub static ref RELEASE_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleasePost
    lazy_static! {
        pub static ref RELEASE_POST_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkIdGet
    lazy_static! {
        pub static ref WORK_ID_GET_FETCH_A_SINGLE_WORK_BY_ID: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkIdGet
    lazy_static! {
        pub static ref WORK_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkIdGet
    lazy_static! {
        pub static ref WORK_ID_GET_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkPost
    lazy_static! {
        pub static ref WORK_POST_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkPost
    lazy_static! {
        pub static ref WORK_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkPost
    lazy_static! {
        pub static ref WORK_POST_GENERIC_ERROR_RESPONSE: Mime = mime!(Application / Json);
    }

}

pub mod requests {
    use hyper::mime::*;
    /// Create Mime objects for the request content types for ContainerPost
    lazy_static! {
        pub static ref CONTAINER_POST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for CreatorPost
    lazy_static! {
        pub static ref CREATOR_POST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for FilePost
    lazy_static! {
        pub static ref FILE_POST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for ReleasePost
    lazy_static! {
        pub static ref RELEASE_POST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the request content types for WorkPost
    lazy_static! {
        pub static ref WORK_POST: Mime = mime!(Application / Json);
    }

}
