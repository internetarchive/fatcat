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
pub enum AcceptEditgroupResponse {
    /// Merged Successfully
    MergedSuccessfully(models::Success),
    /// Unmergable
    Unmergable(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

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
pub enum CreateEditgroupResponse {
    /// Successfully Created
    SuccessfullyCreated(models::Editgroup),
    /// Bad Request
    BadRequest(models::ErrorResponse),
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
pub enum GetCreatorReleasesResponse {
    /// Found Entity
    FoundEntity(Vec<models::ReleaseEntity>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetEditgroupResponse {
    /// Found Entity
    FoundEntity(models::Editgroup),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetEditorResponse {
    /// Found Editor
    FoundEditor(models::Editor),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetEditorChangelogResponse {
    /// Found Merged Changes
    FoundMergedChanges(Vec<models::ChangelogEntry>),
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
pub enum GetReleaseFilesResponse {
    /// Found Entity
    FoundEntity(Vec<models::FileEntity>),
    /// Bad Request
    BadRequest(models::ErrorResponse),
    /// Not Found
    NotFound(models::ErrorResponse),
    /// Generic Error
    GenericError(models::ErrorResponse),
}

#[derive(Debug, PartialEq)]
pub enum GetStatsResponse {
    /// Success
    Success(models::StatsResponse),
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
pub enum GetWorkReleasesResponse {
    /// Found Entity
    FoundEntity(Vec<models::ReleaseEntity>),
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

/// API
pub trait Api {
    fn accept_editgroup(&self, id: i64, context: &Context) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send>;

    fn create_container(&self, entity: models::ContainerEntity, context: &Context) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send>;

    fn create_container_batch(&self, entity_list: &Vec<models::ContainerEntity>, context: &Context) -> Box<Future<Item = CreateContainerBatchResponse, Error = ApiError> + Send>;

    fn create_creator(&self, entity: models::CreatorEntity, context: &Context) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send>;

    fn create_creator_batch(&self, entity_list: &Vec<models::CreatorEntity>, context: &Context) -> Box<Future<Item = CreateCreatorBatchResponse, Error = ApiError> + Send>;

    fn create_editgroup(&self, entity: models::Editgroup, context: &Context) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send>;

    fn create_file(&self, entity: models::FileEntity, context: &Context) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send>;

    fn create_file_batch(&self, entity_list: &Vec<models::FileEntity>, context: &Context) -> Box<Future<Item = CreateFileBatchResponse, Error = ApiError> + Send>;

    fn create_release(&self, entity: models::ReleaseEntity, context: &Context) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send>;

    fn create_release_batch(&self, entity_list: &Vec<models::ReleaseEntity>, context: &Context) -> Box<Future<Item = CreateReleaseBatchResponse, Error = ApiError> + Send>;

    fn create_work(&self, entity: models::WorkEntity, context: &Context) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send>;

    fn create_work_batch(&self, entity_list: &Vec<models::WorkEntity>, context: &Context) -> Box<Future<Item = CreateWorkBatchResponse, Error = ApiError> + Send>;

    fn get_container(&self, id: String, context: &Context) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send>;

    fn get_container_history(&self, id: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send>;

    fn get_creator(&self, id: String, context: &Context) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send>;

    fn get_creator_releases(&self, id: String, context: &Context) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send>;

    fn get_editgroup(&self, id: i64, context: &Context) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send>;

    fn get_editor(&self, username: String, context: &Context) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send>;

    fn get_editor_changelog(&self, username: String, context: &Context) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send>;

    fn get_file(&self, id: String, context: &Context) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send>;

    fn get_release(&self, id: String, context: &Context) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send>;

    fn get_release_files(&self, id: String, context: &Context) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send>;

    fn get_stats(&self, more: Option<String>, context: &Context) -> Box<Future<Item = GetStatsResponse, Error = ApiError> + Send>;

    fn get_work(&self, id: String, context: &Context) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send>;

    fn get_work_releases(&self, id: String, context: &Context) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send>;

    fn lookup_container(&self, issnl: String, context: &Context) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send>;

    fn lookup_creator(&self, orcid: String, context: &Context) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send>;

    fn lookup_file(&self, sha1: String, context: &Context) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send>;

    fn lookup_release(&self, doi: String, context: &Context) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send>;
}

/// API without a `Context`
pub trait ApiNoContext {
    fn accept_editgroup(&self, id: i64) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send>;

    fn create_container(&self, entity: models::ContainerEntity) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send>;

    fn create_container_batch(&self, entity_list: &Vec<models::ContainerEntity>) -> Box<Future<Item = CreateContainerBatchResponse, Error = ApiError> + Send>;

    fn create_creator(&self, entity: models::CreatorEntity) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send>;

    fn create_creator_batch(&self, entity_list: &Vec<models::CreatorEntity>) -> Box<Future<Item = CreateCreatorBatchResponse, Error = ApiError> + Send>;

    fn create_editgroup(&self, entity: models::Editgroup) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send>;

    fn create_file(&self, entity: models::FileEntity) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send>;

    fn create_file_batch(&self, entity_list: &Vec<models::FileEntity>) -> Box<Future<Item = CreateFileBatchResponse, Error = ApiError> + Send>;

    fn create_release(&self, entity: models::ReleaseEntity) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send>;

    fn create_release_batch(&self, entity_list: &Vec<models::ReleaseEntity>) -> Box<Future<Item = CreateReleaseBatchResponse, Error = ApiError> + Send>;

    fn create_work(&self, entity: models::WorkEntity) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send>;

    fn create_work_batch(&self, entity_list: &Vec<models::WorkEntity>) -> Box<Future<Item = CreateWorkBatchResponse, Error = ApiError> + Send>;

    fn get_container(&self, id: String) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send>;

    fn get_container_history(&self, id: String, limit: Option<i64>) -> Box<Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send>;

    fn get_creator(&self, id: String) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send>;

    fn get_creator_releases(&self, id: String) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send>;

    fn get_editgroup(&self, id: i64) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send>;

    fn get_editor(&self, username: String) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send>;

    fn get_editor_changelog(&self, username: String) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send>;

    fn get_file(&self, id: String) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send>;

    fn get_release(&self, id: String) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send>;

    fn get_release_files(&self, id: String) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send>;

    fn get_stats(&self, more: Option<String>) -> Box<Future<Item = GetStatsResponse, Error = ApiError> + Send>;

    fn get_work(&self, id: String) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send>;

    fn get_work_releases(&self, id: String) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send>;

    fn lookup_container(&self, issnl: String) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send>;

    fn lookup_creator(&self, orcid: String) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send>;

    fn lookup_file(&self, sha1: String) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send>;

    fn lookup_release(&self, doi: String) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send>;
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
    fn accept_editgroup(&self, id: i64) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        self.api().accept_editgroup(id, &self.context())
    }

    fn create_container(&self, entity: models::ContainerEntity) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send> {
        self.api().create_container(entity, &self.context())
    }

    fn create_container_batch(&self, entity_list: &Vec<models::ContainerEntity>) -> Box<Future<Item = CreateContainerBatchResponse, Error = ApiError> + Send> {
        self.api().create_container_batch(entity_list, &self.context())
    }

    fn create_creator(&self, entity: models::CreatorEntity) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send> {
        self.api().create_creator(entity, &self.context())
    }

    fn create_creator_batch(&self, entity_list: &Vec<models::CreatorEntity>) -> Box<Future<Item = CreateCreatorBatchResponse, Error = ApiError> + Send> {
        self.api().create_creator_batch(entity_list, &self.context())
    }

    fn create_editgroup(&self, entity: models::Editgroup) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send> {
        self.api().create_editgroup(entity, &self.context())
    }

    fn create_file(&self, entity: models::FileEntity) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send> {
        self.api().create_file(entity, &self.context())
    }

    fn create_file_batch(&self, entity_list: &Vec<models::FileEntity>) -> Box<Future<Item = CreateFileBatchResponse, Error = ApiError> + Send> {
        self.api().create_file_batch(entity_list, &self.context())
    }

    fn create_release(&self, entity: models::ReleaseEntity) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send> {
        self.api().create_release(entity, &self.context())
    }

    fn create_release_batch(&self, entity_list: &Vec<models::ReleaseEntity>) -> Box<Future<Item = CreateReleaseBatchResponse, Error = ApiError> + Send> {
        self.api().create_release_batch(entity_list, &self.context())
    }

    fn create_work(&self, entity: models::WorkEntity) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send> {
        self.api().create_work(entity, &self.context())
    }

    fn create_work_batch(&self, entity_list: &Vec<models::WorkEntity>) -> Box<Future<Item = CreateWorkBatchResponse, Error = ApiError> + Send> {
        self.api().create_work_batch(entity_list, &self.context())
    }

    fn get_container(&self, id: String) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send> {
        self.api().get_container(id, &self.context())
    }

    fn get_container_history(&self, id: String, limit: Option<i64>) -> Box<Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send> {
        self.api().get_container_history(id, limit, &self.context())
    }

    fn get_creator(&self, id: String) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send> {
        self.api().get_creator(id, &self.context())
    }

    fn get_creator_releases(&self, id: String) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send> {
        self.api().get_creator_releases(id, &self.context())
    }

    fn get_editgroup(&self, id: i64) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        self.api().get_editgroup(id, &self.context())
    }

    fn get_editor(&self, username: String) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send> {
        self.api().get_editor(username, &self.context())
    }

    fn get_editor_changelog(&self, username: String) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send> {
        self.api().get_editor_changelog(username, &self.context())
    }

    fn get_file(&self, id: String) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send> {
        self.api().get_file(id, &self.context())
    }

    fn get_release(&self, id: String) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send> {
        self.api().get_release(id, &self.context())
    }

    fn get_release_files(&self, id: String) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send> {
        self.api().get_release_files(id, &self.context())
    }

    fn get_stats(&self, more: Option<String>) -> Box<Future<Item = GetStatsResponse, Error = ApiError> + Send> {
        self.api().get_stats(more, &self.context())
    }

    fn get_work(&self, id: String) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send> {
        self.api().get_work(id, &self.context())
    }

    fn get_work_releases(&self, id: String) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send> {
        self.api().get_work_releases(id, &self.context())
    }

    fn lookup_container(&self, issnl: String) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send> {
        self.api().lookup_container(issnl, &self.context())
    }

    fn lookup_creator(&self, orcid: String) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send> {
        self.api().lookup_creator(orcid, &self.context())
    }

    fn lookup_file(&self, sha1: String) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send> {
        self.api().lookup_file(sha1, &self.context())
    }

    fn lookup_release(&self, doi: String) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send> {
        self.api().lookup_release(doi, &self.context())
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
