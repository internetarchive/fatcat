#![allow(missing_docs, trivial_casts, unused_variables, unused_mut, unused_imports, unused_extern_crates, non_camel_case_types)]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate chrono;
extern crate futures;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

// Logically this should be in the client and server modules, but rust doesn't allow `macro_use` from a module.
#[cfg(any(feature = "client", feature = "server"))]
#[macro_use]
extern crate hyper;

extern crate swagger;

use futures::Stream;
use std::io::Error;

#[allow(unused_imports)]
use std::collections::HashMap;

pub use futures::Future;

#[cfg(any(feature = "client", feature = "server"))]
mod mimetypes;

pub use swagger::{ApiError, Context, ContextWrapper};

#[derive(Debug, PartialEq)]
pub enum CreateContainerResponse {
    /// Created Entity
    CreatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateContainerBatchResponse {
    /// Created Entities
    CreatedEntities(Vec<models::EntityEdit>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteContainerResponse {
    /// Deleted Entity
    DeletedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteContainerEditResponse {
    /// Deleted Edit
    DeletedEdit(models::Success),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetContainerResponse {
    /// Found Entity
    FoundEntity(models::ContainerEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetContainerEditResponse {
    /// Found Edit
    FoundEdit(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetContainerHistoryResponse {
    /// Found Entity History
    FoundEntityHistory(Vec<models::EntityHistoryEntry>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetContainerRedirectsResponse {
    /// Found Entity Redirects
    FoundEntityRedirects(Vec<String>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetContainerRevisionResponse {
    /// Found Entity Revision
    FoundEntityRevision(models::ContainerEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum LookupContainerResponse {
    /// Found Entity
    FoundEntity(models::ContainerEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum UpdateContainerResponse {
    /// Updated Entity
    UpdatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateCreatorResponse {
    /// Created Entity
    CreatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateCreatorBatchResponse {
    /// Created Entities
    CreatedEntities(Vec<models::EntityEdit>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteCreatorResponse {
    /// Deleted Entity
    DeletedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteCreatorEditResponse {
    /// Deleted Edit
    DeletedEdit(models::Success),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetCreatorResponse {
    /// Found Entity
    FoundEntity(models::CreatorEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetCreatorEditResponse {
    /// Found Edit
    FoundEdit(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetCreatorHistoryResponse {
    /// Found Entity History
    FoundEntityHistory(Vec<models::EntityHistoryEntry>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetCreatorRedirectsResponse {
    /// Found Entity Redirects
    FoundEntityRedirects(Vec<String>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetCreatorReleasesResponse {
    /// Found
    Found(Vec<models::ReleaseEntity>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetCreatorRevisionResponse {
    /// Found Entity Revision
    FoundEntityRevision(models::CreatorEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum LookupCreatorResponse {
    /// Found Entity
    FoundEntity(models::CreatorEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum UpdateCreatorResponse {
    /// Updated Entity
    UpdatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetEditorResponse {
    /// Found
    Found(models::Editor),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetEditorChangelogResponse {
    /// Found
    Found(Vec<models::ChangelogEntry>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum AcceptEditgroupResponse {
    /// Merged Successfully
    MergedSuccessfully(models::Success),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Edit Conflict
    EditConflict(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateEditgroupResponse {
    /// Successfully Created
    SuccessfullyCreated(models::Editgroup),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetChangelogResponse {
    /// Success
    Success(Vec<models::ChangelogEntry>),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetChangelogEntryResponse {
    /// Found Changelog Entry
    FoundChangelogEntry(models::ChangelogEntry),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetEditgroupResponse {
    /// Found
    Found(models::Editgroup),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateFileResponse {
    /// Created Entity
    CreatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateFileBatchResponse {
    /// Created Entities
    CreatedEntities(Vec<models::EntityEdit>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteFileResponse {
    /// Deleted Entity
    DeletedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteFileEditResponse {
    /// Deleted Edit
    DeletedEdit(models::Success),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFileResponse {
    /// Found Entity
    FoundEntity(models::FileEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFileEditResponse {
    /// Found Edit
    FoundEdit(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFileHistoryResponse {
    /// Found Entity History
    FoundEntityHistory(Vec<models::EntityHistoryEntry>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFileRedirectsResponse {
    /// Found Entity Redirects
    FoundEntityRedirects(Vec<String>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFileRevisionResponse {
    /// Found Entity Revision
    FoundEntityRevision(models::FileEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum LookupFileResponse {
    /// Found Entity
    FoundEntity(models::FileEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum UpdateFileResponse {
    /// Updated Entity
    UpdatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateFilesetResponse {
    /// Created Entity
    CreatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateFilesetBatchResponse {
    /// Created Entities
    CreatedEntities(Vec<models::EntityEdit>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteFilesetResponse {
    /// Deleted Entity
    DeletedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteFilesetEditResponse {
    /// Deleted Edit
    DeletedEdit(models::Success),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFilesetResponse {
    /// Found Entity
    FoundEntity(models::FilesetEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFilesetEditResponse {
    /// Found Edit
    FoundEdit(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFilesetHistoryResponse {
    /// Found Entity History
    FoundEntityHistory(Vec<models::EntityHistoryEntry>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFilesetRedirectsResponse {
    /// Found Entity Redirects
    FoundEntityRedirects(Vec<String>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetFilesetRevisionResponse {
    /// Found Entity Revision
    FoundEntityRevision(models::FilesetEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum UpdateFilesetResponse {
    /// Updated Entity
    UpdatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateReleaseResponse {
    /// Created Entity
    CreatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateReleaseBatchResponse {
    /// Created Entities
    CreatedEntities(Vec<models::EntityEdit>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateWorkResponse {
    /// Created Entity
    CreatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteReleaseResponse {
    /// Deleted Entity
    DeletedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteReleaseEditResponse {
    /// Deleted Edit
    DeletedEdit(models::Success),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetReleaseResponse {
    /// Found Entity
    FoundEntity(models::ReleaseEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetReleaseEditResponse {
    /// Found Edit
    FoundEdit(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetReleaseFilesResponse {
    /// Found
    Found(Vec<models::FileEntity>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetReleaseFilesetsResponse {
    /// Found
    Found(Vec<models::FilesetEntity>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetReleaseHistoryResponse {
    /// Found Entity History
    FoundEntityHistory(Vec<models::EntityHistoryEntry>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetReleaseRedirectsResponse {
    /// Found Entity Redirects
    FoundEntityRedirects(Vec<String>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetReleaseRevisionResponse {
    /// Found Entity Revision
    FoundEntityRevision(models::ReleaseEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetReleaseWebcapturesResponse {
    /// Found
    Found(Vec<models::WebcaptureEntity>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum LookupReleaseResponse {
    /// Found Entity
    FoundEntity(models::ReleaseEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum UpdateReleaseResponse {
    /// Updated Entity
    UpdatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateWebcaptureResponse {
    /// Created Entity
    CreatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateWebcaptureBatchResponse {
    /// Created Entities
    CreatedEntities(Vec<models::EntityEdit>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteWebcaptureResponse {
    /// Deleted Entity
    DeletedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteWebcaptureEditResponse {
    /// Deleted Edit
    DeletedEdit(models::Success),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWebcaptureResponse {
    /// Found Entity
    FoundEntity(models::WebcaptureEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWebcaptureEditResponse {
    /// Found Edit
    FoundEdit(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWebcaptureHistoryResponse {
    /// Found Entity History
    FoundEntityHistory(Vec<models::EntityHistoryEntry>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWebcaptureRedirectsResponse {
    /// Found Entity Redirects
    FoundEntityRedirects(Vec<String>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWebcaptureRevisionResponse {
    /// Found Entity Revision
    FoundEntityRevision(models::WebcaptureEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum UpdateWebcaptureResponse {
    /// Updated Entity
    UpdatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum CreateWorkBatchResponse {
    /// Created Entities
    CreatedEntities(Vec<models::EntityEdit>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteWorkResponse {
    /// Deleted Entity
    DeletedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum DeleteWorkEditResponse {
    /// Deleted Edit
    DeletedEdit(models::Success),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWorkResponse {
    /// Found Entity
    FoundEntity(models::WorkEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWorkEditResponse {
    /// Found Edit
    FoundEdit(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWorkHistoryResponse {
    /// Found Entity History
    FoundEntityHistory(Vec<models::EntityHistoryEntry>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWorkRedirectsResponse {
    /// Found Entity Redirects
    FoundEntityRedirects(Vec<String>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWorkReleasesResponse {
    /// Found
    Found(Vec<models::ReleaseEntity>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetWorkRevisionResponse {
    /// Found Entity Revision
    FoundEntityRevision(models::WorkEntity),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum UpdateWorkResponse {
    /// Updated Entity
    UpdatedEntity(models::EntityEdit),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

/// API
pub trait Api {
    fn create_container(&self, entity: models::ContainerEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send>;

    fn create_container_batch(
        &self,
        entity_list: &Vec<models::ContainerEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateContainerBatchResponse, Error = ApiError> + Send>;

    fn delete_container(&self, ident: String, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteContainerResponse, Error = ApiError> + Send>;

    fn delete_container_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = DeleteContainerEditResponse, Error = ApiError> + Send>;

    fn get_container(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send>;

    fn get_container_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetContainerEditResponse, Error = ApiError> + Send>;

    fn get_container_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send>;

    fn get_container_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetContainerRedirectsResponse, Error = ApiError> + Send>;

    fn get_container_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetContainerRevisionResponse, Error = ApiError> + Send>;

    fn lookup_container(
        &self,
        issnl: Option<String>,
        wikidata_qid: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send>;

    fn update_container(&self, ident: String, entity: models::ContainerEntity, editgroup_id: Option<String>, context: &Context)
        -> Box<Future<Item = UpdateContainerResponse, Error = ApiError> + Send>;

    fn create_creator(&self, entity: models::CreatorEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send>;

    fn create_creator_batch(
        &self,
        entity_list: &Vec<models::CreatorEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateCreatorBatchResponse, Error = ApiError> + Send>;

    fn delete_creator(&self, ident: String, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteCreatorResponse, Error = ApiError> + Send>;

    fn delete_creator_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = DeleteCreatorEditResponse, Error = ApiError> + Send>;

    fn get_creator(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send>;

    fn get_creator_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetCreatorEditResponse, Error = ApiError> + Send>;

    fn get_creator_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetCreatorHistoryResponse, Error = ApiError> + Send>;

    fn get_creator_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetCreatorRedirectsResponse, Error = ApiError> + Send>;

    fn get_creator_releases(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send>;

    fn get_creator_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorRevisionResponse, Error = ApiError> + Send>;

    fn lookup_creator(
        &self,
        orcid: Option<String>,
        wikidata_qid: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send>;

    fn update_creator(&self, ident: String, entity: models::CreatorEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = UpdateCreatorResponse, Error = ApiError> + Send>;

    fn get_editor(&self, editor_id: String, context: &Context) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send>;

    fn get_editor_changelog(&self, editor_id: String, context: &Context) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send>;

    fn accept_editgroup(&self, editgroup_id: String, context: &Context) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send>;

    fn create_editgroup(&self, editgroup: models::Editgroup, context: &Context) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send>;

    fn get_changelog(&self, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetChangelogResponse, Error = ApiError> + Send>;

    fn get_changelog_entry(&self, index: i64, context: &Context) -> Box<Future<Item = GetChangelogEntryResponse, Error = ApiError> + Send>;

    fn get_editgroup(&self, editgroup_id: String, context: &Context) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send>;

    fn create_file(&self, entity: models::FileEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send>;

    fn create_file_batch(
        &self,
        entity_list: &Vec<models::FileEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateFileBatchResponse, Error = ApiError> + Send>;

    fn delete_file(&self, ident: String, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteFileResponse, Error = ApiError> + Send>;

    fn delete_file_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = DeleteFileEditResponse, Error = ApiError> + Send>;

    fn get_file(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send>;

    fn get_file_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetFileEditResponse, Error = ApiError> + Send>;

    fn get_file_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetFileHistoryResponse, Error = ApiError> + Send>;

    fn get_file_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetFileRedirectsResponse, Error = ApiError> + Send>;

    fn get_file_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFileRevisionResponse, Error = ApiError> + Send>;

    fn lookup_file(
        &self,
        md5: Option<String>,
        sha1: Option<String>,
        sha256: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send>;

    fn update_file(&self, ident: String, entity: models::FileEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = UpdateFileResponse, Error = ApiError> + Send>;

    fn create_fileset(&self, entity: models::FilesetEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateFilesetResponse, Error = ApiError> + Send>;

    fn create_fileset_batch(
        &self,
        entity_list: &Vec<models::FilesetEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateFilesetBatchResponse, Error = ApiError> + Send>;

    fn delete_fileset(&self, ident: String, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteFilesetResponse, Error = ApiError> + Send>;

    fn delete_fileset_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = DeleteFilesetEditResponse, Error = ApiError> + Send>;

    fn get_fileset(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFilesetResponse, Error = ApiError> + Send>;

    fn get_fileset_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetFilesetEditResponse, Error = ApiError> + Send>;

    fn get_fileset_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetFilesetHistoryResponse, Error = ApiError> + Send>;

    fn get_fileset_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetFilesetRedirectsResponse, Error = ApiError> + Send>;

    fn get_fileset_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFilesetRevisionResponse, Error = ApiError> + Send>;

    fn update_fileset(&self, ident: String, entity: models::FilesetEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = UpdateFilesetResponse, Error = ApiError> + Send>;

    fn create_release(&self, entity: models::ReleaseEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send>;

    fn create_release_batch(
        &self,
        entity_list: &Vec<models::ReleaseEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateReleaseBatchResponse, Error = ApiError> + Send>;

    fn create_work(&self, entity: models::WorkEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send>;

    fn delete_release(&self, ident: String, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteReleaseResponse, Error = ApiError> + Send>;

    fn delete_release_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = DeleteReleaseEditResponse, Error = ApiError> + Send>;

    fn get_release(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send>;

    fn get_release_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetReleaseEditResponse, Error = ApiError> + Send>;

    fn get_release_files(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send>;

    fn get_release_filesets(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseFilesetsResponse, Error = ApiError> + Send>;

    fn get_release_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetReleaseHistoryResponse, Error = ApiError> + Send>;

    fn get_release_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetReleaseRedirectsResponse, Error = ApiError> + Send>;

    fn get_release_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseRevisionResponse, Error = ApiError> + Send>;

    fn get_release_webcaptures(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseWebcapturesResponse, Error = ApiError> + Send>;

    fn lookup_release(
        &self,
        doi: Option<String>,
        wikidata_qid: Option<String>,
        isbn13: Option<String>,
        pmid: Option<String>,
        pmcid: Option<String>,
        core_id: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send>;

    fn update_release(&self, ident: String, entity: models::ReleaseEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = UpdateReleaseResponse, Error = ApiError> + Send>;

    fn create_webcapture(&self, entity: models::WebcaptureEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateWebcaptureResponse, Error = ApiError> + Send>;

    fn create_webcapture_batch(
        &self,
        entity_list: &Vec<models::WebcaptureEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateWebcaptureBatchResponse, Error = ApiError> + Send>;

    fn delete_webcapture(&self, ident: String, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteWebcaptureResponse, Error = ApiError> + Send>;

    fn delete_webcapture_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = DeleteWebcaptureEditResponse, Error = ApiError> + Send>;

    fn get_webcapture(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWebcaptureResponse, Error = ApiError> + Send>;

    fn get_webcapture_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetWebcaptureEditResponse, Error = ApiError> + Send>;

    fn get_webcapture_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetWebcaptureHistoryResponse, Error = ApiError> + Send>;

    fn get_webcapture_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetWebcaptureRedirectsResponse, Error = ApiError> + Send>;

    fn get_webcapture_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWebcaptureRevisionResponse, Error = ApiError> + Send>;

    fn update_webcapture(
        &self,
        ident: String,
        entity: models::WebcaptureEntity,
        editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = UpdateWebcaptureResponse, Error = ApiError> + Send>;

    fn create_work_batch(
        &self,
        entity_list: &Vec<models::WorkEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateWorkBatchResponse, Error = ApiError> + Send>;

    fn delete_work(&self, ident: String, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteWorkResponse, Error = ApiError> + Send>;

    fn delete_work_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = DeleteWorkEditResponse, Error = ApiError> + Send>;

    fn get_work(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send>;

    fn get_work_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetWorkEditResponse, Error = ApiError> + Send>;

    fn get_work_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetWorkHistoryResponse, Error = ApiError> + Send>;

    fn get_work_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetWorkRedirectsResponse, Error = ApiError> + Send>;

    fn get_work_releases(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send>;

    fn get_work_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkRevisionResponse, Error = ApiError> + Send>;

    fn update_work(&self, ident: String, entity: models::WorkEntity, editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = UpdateWorkResponse, Error = ApiError> + Send>;
}

/// API without a `Context`
pub trait ApiNoContext {
    fn create_container(&self, entity: models::ContainerEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send>;

    fn create_container_batch(
        &self,
        entity_list: &Vec<models::ContainerEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateContainerBatchResponse, Error = ApiError> + Send>;

    fn delete_container(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteContainerResponse, Error = ApiError> + Send>;

    fn delete_container_edit(&self, edit_id: String) -> Box<Future<Item = DeleteContainerEditResponse, Error = ApiError> + Send>;

    fn get_container(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send>;

    fn get_container_edit(&self, edit_id: String) -> Box<Future<Item = GetContainerEditResponse, Error = ApiError> + Send>;

    fn get_container_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send>;

    fn get_container_redirects(&self, ident: String) -> Box<Future<Item = GetContainerRedirectsResponse, Error = ApiError> + Send>;

    fn get_container_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetContainerRevisionResponse, Error = ApiError> + Send>;

    fn lookup_container(
        &self,
        issnl: Option<String>,
        wikidata_qid: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
    ) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send>;

    fn update_container(&self, ident: String, entity: models::ContainerEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateContainerResponse, Error = ApiError> + Send>;

    fn create_creator(&self, entity: models::CreatorEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send>;

    fn create_creator_batch(
        &self,
        entity_list: &Vec<models::CreatorEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateCreatorBatchResponse, Error = ApiError> + Send>;

    fn delete_creator(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteCreatorResponse, Error = ApiError> + Send>;

    fn delete_creator_edit(&self, edit_id: String) -> Box<Future<Item = DeleteCreatorEditResponse, Error = ApiError> + Send>;

    fn get_creator(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send>;

    fn get_creator_edit(&self, edit_id: String) -> Box<Future<Item = GetCreatorEditResponse, Error = ApiError> + Send>;

    fn get_creator_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetCreatorHistoryResponse, Error = ApiError> + Send>;

    fn get_creator_redirects(&self, ident: String) -> Box<Future<Item = GetCreatorRedirectsResponse, Error = ApiError> + Send>;

    fn get_creator_releases(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send>;

    fn get_creator_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetCreatorRevisionResponse, Error = ApiError> + Send>;

    fn lookup_creator(&self, orcid: Option<String>, wikidata_qid: Option<String>, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send>;

    fn update_creator(&self, ident: String, entity: models::CreatorEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateCreatorResponse, Error = ApiError> + Send>;

    fn get_editor(&self, editor_id: String) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send>;

    fn get_editor_changelog(&self, editor_id: String) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send>;

    fn accept_editgroup(&self, editgroup_id: String) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send>;

    fn create_editgroup(&self, editgroup: models::Editgroup) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send>;

    fn get_changelog(&self, limit: Option<i64>) -> Box<Future<Item = GetChangelogResponse, Error = ApiError> + Send>;

    fn get_changelog_entry(&self, index: i64) -> Box<Future<Item = GetChangelogEntryResponse, Error = ApiError> + Send>;

    fn get_editgroup(&self, editgroup_id: String) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send>;

    fn create_file(&self, entity: models::FileEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send>;

    fn create_file_batch(&self, entity_list: &Vec<models::FileEntity>, autoaccept: Option<bool>, editgroup_id: Option<String>) -> Box<Future<Item = CreateFileBatchResponse, Error = ApiError> + Send>;

    fn delete_file(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteFileResponse, Error = ApiError> + Send>;

    fn delete_file_edit(&self, edit_id: String) -> Box<Future<Item = DeleteFileEditResponse, Error = ApiError> + Send>;

    fn get_file(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send>;

    fn get_file_edit(&self, edit_id: String) -> Box<Future<Item = GetFileEditResponse, Error = ApiError> + Send>;

    fn get_file_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetFileHistoryResponse, Error = ApiError> + Send>;

    fn get_file_redirects(&self, ident: String) -> Box<Future<Item = GetFileRedirectsResponse, Error = ApiError> + Send>;

    fn get_file_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetFileRevisionResponse, Error = ApiError> + Send>;

    fn lookup_file(
        &self,
        md5: Option<String>,
        sha1: Option<String>,
        sha256: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
    ) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send>;

    fn update_file(&self, ident: String, entity: models::FileEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateFileResponse, Error = ApiError> + Send>;

    fn create_fileset(&self, entity: models::FilesetEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateFilesetResponse, Error = ApiError> + Send>;

    fn create_fileset_batch(
        &self,
        entity_list: &Vec<models::FilesetEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateFilesetBatchResponse, Error = ApiError> + Send>;

    fn delete_fileset(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteFilesetResponse, Error = ApiError> + Send>;

    fn delete_fileset_edit(&self, edit_id: String) -> Box<Future<Item = DeleteFilesetEditResponse, Error = ApiError> + Send>;

    fn get_fileset(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetFilesetResponse, Error = ApiError> + Send>;

    fn get_fileset_edit(&self, edit_id: String) -> Box<Future<Item = GetFilesetEditResponse, Error = ApiError> + Send>;

    fn get_fileset_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetFilesetHistoryResponse, Error = ApiError> + Send>;

    fn get_fileset_redirects(&self, ident: String) -> Box<Future<Item = GetFilesetRedirectsResponse, Error = ApiError> + Send>;

    fn get_fileset_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetFilesetRevisionResponse, Error = ApiError> + Send>;

    fn update_fileset(&self, ident: String, entity: models::FilesetEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateFilesetResponse, Error = ApiError> + Send>;

    fn create_release(&self, entity: models::ReleaseEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send>;

    fn create_release_batch(
        &self,
        entity_list: &Vec<models::ReleaseEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateReleaseBatchResponse, Error = ApiError> + Send>;

    fn create_work(&self, entity: models::WorkEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send>;

    fn delete_release(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteReleaseResponse, Error = ApiError> + Send>;

    fn delete_release_edit(&self, edit_id: String) -> Box<Future<Item = DeleteReleaseEditResponse, Error = ApiError> + Send>;

    fn get_release(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send>;

    fn get_release_edit(&self, edit_id: String) -> Box<Future<Item = GetReleaseEditResponse, Error = ApiError> + Send>;

    fn get_release_files(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send>;

    fn get_release_filesets(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetReleaseFilesetsResponse, Error = ApiError> + Send>;

    fn get_release_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetReleaseHistoryResponse, Error = ApiError> + Send>;

    fn get_release_redirects(&self, ident: String) -> Box<Future<Item = GetReleaseRedirectsResponse, Error = ApiError> + Send>;

    fn get_release_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetReleaseRevisionResponse, Error = ApiError> + Send>;

    fn get_release_webcaptures(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetReleaseWebcapturesResponse, Error = ApiError> + Send>;

    fn lookup_release(
        &self,
        doi: Option<String>,
        wikidata_qid: Option<String>,
        isbn13: Option<String>,
        pmid: Option<String>,
        pmcid: Option<String>,
        core_id: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
    ) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send>;

    fn update_release(&self, ident: String, entity: models::ReleaseEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateReleaseResponse, Error = ApiError> + Send>;

    fn create_webcapture(&self, entity: models::WebcaptureEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateWebcaptureResponse, Error = ApiError> + Send>;

    fn create_webcapture_batch(
        &self,
        entity_list: &Vec<models::WebcaptureEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateWebcaptureBatchResponse, Error = ApiError> + Send>;

    fn delete_webcapture(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteWebcaptureResponse, Error = ApiError> + Send>;

    fn delete_webcapture_edit(&self, edit_id: String) -> Box<Future<Item = DeleteWebcaptureEditResponse, Error = ApiError> + Send>;

    fn get_webcapture(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetWebcaptureResponse, Error = ApiError> + Send>;

    fn get_webcapture_edit(&self, edit_id: String) -> Box<Future<Item = GetWebcaptureEditResponse, Error = ApiError> + Send>;

    fn get_webcapture_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetWebcaptureHistoryResponse, Error = ApiError> + Send>;

    fn get_webcapture_redirects(&self, ident: String) -> Box<Future<Item = GetWebcaptureRedirectsResponse, Error = ApiError> + Send>;

    fn get_webcapture_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetWebcaptureRevisionResponse, Error = ApiError> + Send>;

    fn update_webcapture(&self, ident: String, entity: models::WebcaptureEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateWebcaptureResponse, Error = ApiError> + Send>;

    fn create_work_batch(&self, entity_list: &Vec<models::WorkEntity>, autoaccept: Option<bool>, editgroup_id: Option<String>) -> Box<Future<Item = CreateWorkBatchResponse, Error = ApiError> + Send>;

    fn delete_work(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteWorkResponse, Error = ApiError> + Send>;

    fn delete_work_edit(&self, edit_id: String) -> Box<Future<Item = DeleteWorkEditResponse, Error = ApiError> + Send>;

    fn get_work(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send>;

    fn get_work_edit(&self, edit_id: String) -> Box<Future<Item = GetWorkEditResponse, Error = ApiError> + Send>;

    fn get_work_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetWorkHistoryResponse, Error = ApiError> + Send>;

    fn get_work_redirects(&self, ident: String) -> Box<Future<Item = GetWorkRedirectsResponse, Error = ApiError> + Send>;

    fn get_work_releases(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send>;

    fn get_work_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetWorkRevisionResponse, Error = ApiError> + Send>;

    fn update_work(&self, ident: String, entity: models::WorkEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateWorkResponse, Error = ApiError> + Send>;
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<'a>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self: &'a Self, context: Context) -> ContextWrapper<'a, Self>;
}

impl<'a, T: Api + Sized> ContextWrapperExt<'a> for T {
    fn with_context(self: &'a T, context: Context) -> ContextWrapper<'a, T> {
        ContextWrapper::<T>::new(self, context)
    }
}

impl<'a, T: Api> ApiNoContext for ContextWrapper<'a, T> {
    fn create_container(&self, entity: models::ContainerEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send> {
        self.api().create_container(entity, editgroup_id, &self.context())
    }

    fn create_container_batch(
        &self,
        entity_list: &Vec<models::ContainerEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateContainerBatchResponse, Error = ApiError> + Send> {
        self.api().create_container_batch(entity_list, autoaccept, editgroup_id, &self.context())
    }

    fn delete_container(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteContainerResponse, Error = ApiError> + Send> {
        self.api().delete_container(ident, editgroup_id, &self.context())
    }

    fn delete_container_edit(&self, edit_id: String) -> Box<Future<Item = DeleteContainerEditResponse, Error = ApiError> + Send> {
        self.api().delete_container_edit(edit_id, &self.context())
    }

    fn get_container(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send> {
        self.api().get_container(ident, expand, hide, &self.context())
    }

    fn get_container_edit(&self, edit_id: String) -> Box<Future<Item = GetContainerEditResponse, Error = ApiError> + Send> {
        self.api().get_container_edit(edit_id, &self.context())
    }

    fn get_container_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send> {
        self.api().get_container_history(ident, limit, &self.context())
    }

    fn get_container_redirects(&self, ident: String) -> Box<Future<Item = GetContainerRedirectsResponse, Error = ApiError> + Send> {
        self.api().get_container_redirects(ident, &self.context())
    }

    fn get_container_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetContainerRevisionResponse, Error = ApiError> + Send> {
        self.api().get_container_revision(rev_id, expand, hide, &self.context())
    }

    fn lookup_container(
        &self,
        issnl: Option<String>,
        wikidata_qid: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
    ) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send> {
        self.api().lookup_container(issnl, wikidata_qid, expand, hide, &self.context())
    }

    fn update_container(&self, ident: String, entity: models::ContainerEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateContainerResponse, Error = ApiError> + Send> {
        self.api().update_container(ident, entity, editgroup_id, &self.context())
    }

    fn create_creator(&self, entity: models::CreatorEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send> {
        self.api().create_creator(entity, editgroup_id, &self.context())
    }

    fn create_creator_batch(
        &self,
        entity_list: &Vec<models::CreatorEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateCreatorBatchResponse, Error = ApiError> + Send> {
        self.api().create_creator_batch(entity_list, autoaccept, editgroup_id, &self.context())
    }

    fn delete_creator(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteCreatorResponse, Error = ApiError> + Send> {
        self.api().delete_creator(ident, editgroup_id, &self.context())
    }

    fn delete_creator_edit(&self, edit_id: String) -> Box<Future<Item = DeleteCreatorEditResponse, Error = ApiError> + Send> {
        self.api().delete_creator_edit(edit_id, &self.context())
    }

    fn get_creator(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send> {
        self.api().get_creator(ident, expand, hide, &self.context())
    }

    fn get_creator_edit(&self, edit_id: String) -> Box<Future<Item = GetCreatorEditResponse, Error = ApiError> + Send> {
        self.api().get_creator_edit(edit_id, &self.context())
    }

    fn get_creator_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetCreatorHistoryResponse, Error = ApiError> + Send> {
        self.api().get_creator_history(ident, limit, &self.context())
    }

    fn get_creator_redirects(&self, ident: String) -> Box<Future<Item = GetCreatorRedirectsResponse, Error = ApiError> + Send> {
        self.api().get_creator_redirects(ident, &self.context())
    }

    fn get_creator_releases(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send> {
        self.api().get_creator_releases(ident, hide, &self.context())
    }

    fn get_creator_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetCreatorRevisionResponse, Error = ApiError> + Send> {
        self.api().get_creator_revision(rev_id, expand, hide, &self.context())
    }

    fn lookup_creator(&self, orcid: Option<String>, wikidata_qid: Option<String>, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send> {
        self.api().lookup_creator(orcid, wikidata_qid, expand, hide, &self.context())
    }

    fn update_creator(&self, ident: String, entity: models::CreatorEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateCreatorResponse, Error = ApiError> + Send> {
        self.api().update_creator(ident, entity, editgroup_id, &self.context())
    }

    fn get_editor(&self, editor_id: String) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send> {
        self.api().get_editor(editor_id, &self.context())
    }

    fn get_editor_changelog(&self, editor_id: String) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send> {
        self.api().get_editor_changelog(editor_id, &self.context())
    }

    fn accept_editgroup(&self, editgroup_id: String) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        self.api().accept_editgroup(editgroup_id, &self.context())
    }

    fn create_editgroup(&self, editgroup: models::Editgroup) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send> {
        self.api().create_editgroup(editgroup, &self.context())
    }

    fn get_changelog(&self, limit: Option<i64>) -> Box<Future<Item = GetChangelogResponse, Error = ApiError> + Send> {
        self.api().get_changelog(limit, &self.context())
    }

    fn get_changelog_entry(&self, index: i64) -> Box<Future<Item = GetChangelogEntryResponse, Error = ApiError> + Send> {
        self.api().get_changelog_entry(index, &self.context())
    }

    fn get_editgroup(&self, editgroup_id: String) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        self.api().get_editgroup(editgroup_id, &self.context())
    }

    fn create_file(&self, entity: models::FileEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send> {
        self.api().create_file(entity, editgroup_id, &self.context())
    }

    fn create_file_batch(&self, entity_list: &Vec<models::FileEntity>, autoaccept: Option<bool>, editgroup_id: Option<String>) -> Box<Future<Item = CreateFileBatchResponse, Error = ApiError> + Send> {
        self.api().create_file_batch(entity_list, autoaccept, editgroup_id, &self.context())
    }

    fn delete_file(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteFileResponse, Error = ApiError> + Send> {
        self.api().delete_file(ident, editgroup_id, &self.context())
    }

    fn delete_file_edit(&self, edit_id: String) -> Box<Future<Item = DeleteFileEditResponse, Error = ApiError> + Send> {
        self.api().delete_file_edit(edit_id, &self.context())
    }

    fn get_file(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send> {
        self.api().get_file(ident, expand, hide, &self.context())
    }

    fn get_file_edit(&self, edit_id: String) -> Box<Future<Item = GetFileEditResponse, Error = ApiError> + Send> {
        self.api().get_file_edit(edit_id, &self.context())
    }

    fn get_file_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetFileHistoryResponse, Error = ApiError> + Send> {
        self.api().get_file_history(ident, limit, &self.context())
    }

    fn get_file_redirects(&self, ident: String) -> Box<Future<Item = GetFileRedirectsResponse, Error = ApiError> + Send> {
        self.api().get_file_redirects(ident, &self.context())
    }

    fn get_file_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetFileRevisionResponse, Error = ApiError> + Send> {
        self.api().get_file_revision(rev_id, expand, hide, &self.context())
    }

    fn lookup_file(
        &self,
        md5: Option<String>,
        sha1: Option<String>,
        sha256: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
    ) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send> {
        self.api().lookup_file(md5, sha1, sha256, expand, hide, &self.context())
    }

    fn update_file(&self, ident: String, entity: models::FileEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateFileResponse, Error = ApiError> + Send> {
        self.api().update_file(ident, entity, editgroup_id, &self.context())
    }

    fn create_fileset(&self, entity: models::FilesetEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateFilesetResponse, Error = ApiError> + Send> {
        self.api().create_fileset(entity, editgroup_id, &self.context())
    }

    fn create_fileset_batch(
        &self,
        entity_list: &Vec<models::FilesetEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateFilesetBatchResponse, Error = ApiError> + Send> {
        self.api().create_fileset_batch(entity_list, autoaccept, editgroup_id, &self.context())
    }

    fn delete_fileset(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteFilesetResponse, Error = ApiError> + Send> {
        self.api().delete_fileset(ident, editgroup_id, &self.context())
    }

    fn delete_fileset_edit(&self, edit_id: String) -> Box<Future<Item = DeleteFilesetEditResponse, Error = ApiError> + Send> {
        self.api().delete_fileset_edit(edit_id, &self.context())
    }

    fn get_fileset(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetFilesetResponse, Error = ApiError> + Send> {
        self.api().get_fileset(ident, expand, hide, &self.context())
    }

    fn get_fileset_edit(&self, edit_id: String) -> Box<Future<Item = GetFilesetEditResponse, Error = ApiError> + Send> {
        self.api().get_fileset_edit(edit_id, &self.context())
    }

    fn get_fileset_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetFilesetHistoryResponse, Error = ApiError> + Send> {
        self.api().get_fileset_history(ident, limit, &self.context())
    }

    fn get_fileset_redirects(&self, ident: String) -> Box<Future<Item = GetFilesetRedirectsResponse, Error = ApiError> + Send> {
        self.api().get_fileset_redirects(ident, &self.context())
    }

    fn get_fileset_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetFilesetRevisionResponse, Error = ApiError> + Send> {
        self.api().get_fileset_revision(rev_id, expand, hide, &self.context())
    }

    fn update_fileset(&self, ident: String, entity: models::FilesetEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateFilesetResponse, Error = ApiError> + Send> {
        self.api().update_fileset(ident, entity, editgroup_id, &self.context())
    }

    fn create_release(&self, entity: models::ReleaseEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send> {
        self.api().create_release(entity, editgroup_id, &self.context())
    }

    fn create_release_batch(
        &self,
        entity_list: &Vec<models::ReleaseEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateReleaseBatchResponse, Error = ApiError> + Send> {
        self.api().create_release_batch(entity_list, autoaccept, editgroup_id, &self.context())
    }

    fn create_work(&self, entity: models::WorkEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send> {
        self.api().create_work(entity, editgroup_id, &self.context())
    }

    fn delete_release(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteReleaseResponse, Error = ApiError> + Send> {
        self.api().delete_release(ident, editgroup_id, &self.context())
    }

    fn delete_release_edit(&self, edit_id: String) -> Box<Future<Item = DeleteReleaseEditResponse, Error = ApiError> + Send> {
        self.api().delete_release_edit(edit_id, &self.context())
    }

    fn get_release(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send> {
        self.api().get_release(ident, expand, hide, &self.context())
    }

    fn get_release_edit(&self, edit_id: String) -> Box<Future<Item = GetReleaseEditResponse, Error = ApiError> + Send> {
        self.api().get_release_edit(edit_id, &self.context())
    }

    fn get_release_files(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send> {
        self.api().get_release_files(ident, hide, &self.context())
    }

    fn get_release_filesets(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetReleaseFilesetsResponse, Error = ApiError> + Send> {
        self.api().get_release_filesets(ident, hide, &self.context())
    }

    fn get_release_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetReleaseHistoryResponse, Error = ApiError> + Send> {
        self.api().get_release_history(ident, limit, &self.context())
    }

    fn get_release_redirects(&self, ident: String) -> Box<Future<Item = GetReleaseRedirectsResponse, Error = ApiError> + Send> {
        self.api().get_release_redirects(ident, &self.context())
    }

    fn get_release_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetReleaseRevisionResponse, Error = ApiError> + Send> {
        self.api().get_release_revision(rev_id, expand, hide, &self.context())
    }

    fn get_release_webcaptures(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetReleaseWebcapturesResponse, Error = ApiError> + Send> {
        self.api().get_release_webcaptures(ident, hide, &self.context())
    }

    fn lookup_release(
        &self,
        doi: Option<String>,
        wikidata_qid: Option<String>,
        isbn13: Option<String>,
        pmid: Option<String>,
        pmcid: Option<String>,
        core_id: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
    ) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send> {
        self.api().lookup_release(doi, wikidata_qid, isbn13, pmid, pmcid, core_id, expand, hide, &self.context())
    }

    fn update_release(&self, ident: String, entity: models::ReleaseEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateReleaseResponse, Error = ApiError> + Send> {
        self.api().update_release(ident, entity, editgroup_id, &self.context())
    }

    fn create_webcapture(&self, entity: models::WebcaptureEntity, editgroup_id: Option<String>) -> Box<Future<Item = CreateWebcaptureResponse, Error = ApiError> + Send> {
        self.api().create_webcapture(entity, editgroup_id, &self.context())
    }

    fn create_webcapture_batch(
        &self,
        entity_list: &Vec<models::WebcaptureEntity>,
        autoaccept: Option<bool>,
        editgroup_id: Option<String>,
    ) -> Box<Future<Item = CreateWebcaptureBatchResponse, Error = ApiError> + Send> {
        self.api().create_webcapture_batch(entity_list, autoaccept, editgroup_id, &self.context())
    }

    fn delete_webcapture(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteWebcaptureResponse, Error = ApiError> + Send> {
        self.api().delete_webcapture(ident, editgroup_id, &self.context())
    }

    fn delete_webcapture_edit(&self, edit_id: String) -> Box<Future<Item = DeleteWebcaptureEditResponse, Error = ApiError> + Send> {
        self.api().delete_webcapture_edit(edit_id, &self.context())
    }

    fn get_webcapture(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetWebcaptureResponse, Error = ApiError> + Send> {
        self.api().get_webcapture(ident, expand, hide, &self.context())
    }

    fn get_webcapture_edit(&self, edit_id: String) -> Box<Future<Item = GetWebcaptureEditResponse, Error = ApiError> + Send> {
        self.api().get_webcapture_edit(edit_id, &self.context())
    }

    fn get_webcapture_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetWebcaptureHistoryResponse, Error = ApiError> + Send> {
        self.api().get_webcapture_history(ident, limit, &self.context())
    }

    fn get_webcapture_redirects(&self, ident: String) -> Box<Future<Item = GetWebcaptureRedirectsResponse, Error = ApiError> + Send> {
        self.api().get_webcapture_redirects(ident, &self.context())
    }

    fn get_webcapture_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetWebcaptureRevisionResponse, Error = ApiError> + Send> {
        self.api().get_webcapture_revision(rev_id, expand, hide, &self.context())
    }

    fn update_webcapture(&self, ident: String, entity: models::WebcaptureEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateWebcaptureResponse, Error = ApiError> + Send> {
        self.api().update_webcapture(ident, entity, editgroup_id, &self.context())
    }

    fn create_work_batch(&self, entity_list: &Vec<models::WorkEntity>, autoaccept: Option<bool>, editgroup_id: Option<String>) -> Box<Future<Item = CreateWorkBatchResponse, Error = ApiError> + Send> {
        self.api().create_work_batch(entity_list, autoaccept, editgroup_id, &self.context())
    }

    fn delete_work(&self, ident: String, editgroup_id: Option<String>) -> Box<Future<Item = DeleteWorkResponse, Error = ApiError> + Send> {
        self.api().delete_work(ident, editgroup_id, &self.context())
    }

    fn delete_work_edit(&self, edit_id: String) -> Box<Future<Item = DeleteWorkEditResponse, Error = ApiError> + Send> {
        self.api().delete_work_edit(edit_id, &self.context())
    }

    fn get_work(&self, ident: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send> {
        self.api().get_work(ident, expand, hide, &self.context())
    }

    fn get_work_edit(&self, edit_id: String) -> Box<Future<Item = GetWorkEditResponse, Error = ApiError> + Send> {
        self.api().get_work_edit(edit_id, &self.context())
    }

    fn get_work_history(&self, ident: String, limit: Option<i64>) -> Box<Future<Item = GetWorkHistoryResponse, Error = ApiError> + Send> {
        self.api().get_work_history(ident, limit, &self.context())
    }

    fn get_work_redirects(&self, ident: String) -> Box<Future<Item = GetWorkRedirectsResponse, Error = ApiError> + Send> {
        self.api().get_work_redirects(ident, &self.context())
    }

    fn get_work_releases(&self, ident: String, hide: Option<String>) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send> {
        self.api().get_work_releases(ident, hide, &self.context())
    }

    fn get_work_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>) -> Box<Future<Item = GetWorkRevisionResponse, Error = ApiError> + Send> {
        self.api().get_work_revision(rev_id, expand, hide, &self.context())
    }

    fn update_work(&self, ident: String, entity: models::WorkEntity, editgroup_id: Option<String>) -> Box<Future<Item = UpdateWorkResponse, Error = ApiError> + Send> {
        self.api().update_work(ident, entity, editgroup_id, &self.context())
    }
}

#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use self::client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::router;

pub mod models;
