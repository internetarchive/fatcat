/// mime types for requests and responses

pub mod responses {
    use hyper::mime::*;

    // The macro is called per-operation to beat the recursion limit
    /// Create Mime objects for the response content types for ContainerIdGet
    lazy_static! {
        pub static ref CONTAINER_ID_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerIdGet
    lazy_static! {
        pub static ref CONTAINER_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerIdGet
    lazy_static! {
        pub static ref CONTAINER_ID_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerIdGet
    lazy_static! {
        pub static ref CONTAINER_ID_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerLookupGet
    lazy_static! {
        pub static ref CONTAINER_LOOKUP_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerLookupGet
    lazy_static! {
        pub static ref CONTAINER_LOOKUP_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerLookupGet
    lazy_static! {
        pub static ref CONTAINER_LOOKUP_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerLookupGet
    lazy_static! {
        pub static ref CONTAINER_LOOKUP_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerPost
    lazy_static! {
        pub static ref CONTAINER_POST_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerPost
    lazy_static! {
        pub static ref CONTAINER_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerPost
    lazy_static! {
        pub static ref CONTAINER_POST_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ContainerPost
    lazy_static! {
        pub static ref CONTAINER_POST_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorIdGet
    lazy_static! {
        pub static ref CREATOR_ID_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorIdGet
    lazy_static! {
        pub static ref CREATOR_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorIdGet
    lazy_static! {
        pub static ref CREATOR_ID_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorIdGet
    lazy_static! {
        pub static ref CREATOR_ID_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorLookupGet
    lazy_static! {
        pub static ref CREATOR_LOOKUP_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorLookupGet
    lazy_static! {
        pub static ref CREATOR_LOOKUP_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorLookupGet
    lazy_static! {
        pub static ref CREATOR_LOOKUP_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorLookupGet
    lazy_static! {
        pub static ref CREATOR_LOOKUP_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorPost
    lazy_static! {
        pub static ref CREATOR_POST_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorPost
    lazy_static! {
        pub static ref CREATOR_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorPost
    lazy_static! {
        pub static ref CREATOR_POST_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for CreatorPost
    lazy_static! {
        pub static ref CREATOR_POST_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdAcceptPost
    lazy_static! {
        pub static ref EDITGROUP_ID_ACCEPT_POST_MERGED_SUCCESSFULLY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdAcceptPost
    lazy_static! {
        pub static ref EDITGROUP_ID_ACCEPT_POST_UNMERGABLE: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdAcceptPost
    lazy_static! {
        pub static ref EDITGROUP_ID_ACCEPT_POST_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdAcceptPost
    lazy_static! {
        pub static ref EDITGROUP_ID_ACCEPT_POST_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdGet
    lazy_static! {
        pub static ref EDITGROUP_ID_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdGet
    lazy_static! {
        pub static ref EDITGROUP_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdGet
    lazy_static! {
        pub static ref EDITGROUP_ID_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupIdGet
    lazy_static! {
        pub static ref EDITGROUP_ID_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupPost
    lazy_static! {
        pub static ref EDITGROUP_POST_SUCCESSFULLY_CREATED: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupPost
    lazy_static! {
        pub static ref EDITGROUP_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditgroupPost
    lazy_static! {
        pub static ref EDITGROUP_POST_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameChangelogGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_CHANGELOG_GET_FOUND_MERGED_CHANGES: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameChangelogGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_CHANGELOG_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameChangelogGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_CHANGELOG_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_GET_FOUND_EDITOR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for EditorUsernameGet
    lazy_static! {
        pub static ref EDITOR_USERNAME_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileIdGet
    lazy_static! {
        pub static ref FILE_ID_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileIdGet
    lazy_static! {
        pub static ref FILE_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileIdGet
    lazy_static! {
        pub static ref FILE_ID_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileIdGet
    lazy_static! {
        pub static ref FILE_ID_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileLookupGet
    lazy_static! {
        pub static ref FILE_LOOKUP_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileLookupGet
    lazy_static! {
        pub static ref FILE_LOOKUP_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileLookupGet
    lazy_static! {
        pub static ref FILE_LOOKUP_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FileLookupGet
    lazy_static! {
        pub static ref FILE_LOOKUP_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FilePost
    lazy_static! {
        pub static ref FILE_POST_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FilePost
    lazy_static! {
        pub static ref FILE_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FilePost
    lazy_static! {
        pub static ref FILE_POST_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for FilePost
    lazy_static! {
        pub static ref FILE_POST_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseIdGet
    lazy_static! {
        pub static ref RELEASE_ID_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseIdGet
    lazy_static! {
        pub static ref RELEASE_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseIdGet
    lazy_static! {
        pub static ref RELEASE_ID_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseIdGet
    lazy_static! {
        pub static ref RELEASE_ID_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseLookupGet
    lazy_static! {
        pub static ref RELEASE_LOOKUP_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseLookupGet
    lazy_static! {
        pub static ref RELEASE_LOOKUP_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseLookupGet
    lazy_static! {
        pub static ref RELEASE_LOOKUP_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleaseLookupGet
    lazy_static! {
        pub static ref RELEASE_LOOKUP_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleasePost
    lazy_static! {
        pub static ref RELEASE_POST_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleasePost
    lazy_static! {
        pub static ref RELEASE_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleasePost
    lazy_static! {
        pub static ref RELEASE_POST_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for ReleasePost
    lazy_static! {
        pub static ref RELEASE_POST_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkIdGet
    lazy_static! {
        pub static ref WORK_ID_GET_FOUND_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkIdGet
    lazy_static! {
        pub static ref WORK_ID_GET_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkIdGet
    lazy_static! {
        pub static ref WORK_ID_GET_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkIdGet
    lazy_static! {
        pub static ref WORK_ID_GET_GENERIC_ERROR: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkPost
    lazy_static! {
        pub static ref WORK_POST_CREATED_ENTITY: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkPost
    lazy_static! {
        pub static ref WORK_POST_BAD_REQUEST: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkPost
    lazy_static! {
        pub static ref WORK_POST_NOT_FOUND: Mime = mime!(Application / Json);
    }
    /// Create Mime objects for the response content types for WorkPost
    lazy_static! {
        pub static ref WORK_POST_GENERIC_ERROR: Mime = mime!(Application / Json);
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
