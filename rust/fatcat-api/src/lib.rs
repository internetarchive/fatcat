#![allow(missing_docs, trivial_casts, unused_variables, unused_mut, unused_imports, unused_extern_crates, non_camel_case_types)]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate futures;
extern crate chrono;

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
pub enum ContainerIdGetResponse {
    /// fetch a single container by id
    FetchASingleContainerById ( models::ContainerEntity ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum ContainerLookupGetResponse {
    /// find a single container by external identifer
    FindASingleContainerByExternalIdentifer ( models::ContainerEntity ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// no such container
    NoSuchContainer ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum ContainerPostResponse {
    /// created
    Created ( models::EntityEdit ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum CreatorIdGetResponse {
    /// fetch a single creator by id
    FetchASingleCreatorById ( models::CreatorEntity ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum CreatorLookupGetResponse {
    /// find a single creator by external identifer
    FindASingleCreatorByExternalIdentifer ( models::CreatorEntity ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// no such creator
    NoSuchCreator ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum CreatorPostResponse {
    /// created
    Created ( models::EntityEdit ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum EditgroupIdAcceptPostResponse {
    /// merged editgroup successfully (\&quot;live\&quot;)
    MergedEditgroupSuccessfully_ ( models::Success ) ,
    /// editgroup is in an unmergable state
    EditgroupIsInAnUnmergableState ( models::Error ) ,
    /// no such editgroup
    NoSuchEditgroup ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum EditgroupIdGetResponse {
    /// fetch editgroup by identifier
    FetchEditgroupByIdentifier ( models::Editgroup ) ,
    /// no such editgroup
    NoSuchEditgroup ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum EditgroupPostResponse {
    /// successfully created
    SuccessfullyCreated ( models::Editgroup ) ,
    /// invalid request parameters
    InvalidRequestParameters ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum EditorUsernameChangelogGetResponse {
    /// find changes (editgroups) by this editor which have been merged
    FindChanges_ ( models::Changelogentry ) ,
    /// username not found
    UsernameNotFound ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum EditorUsernameGetResponse {
    /// fetch generic information about an editor
    FetchGenericInformationAboutAnEditor ( models::Editor ) ,
    /// username not found
    UsernameNotFound ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum FileIdGetResponse {
    /// fetch a single file by id
    FetchASingleFileById ( models::FileEntity ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum FileLookupGetResponse {
    /// find a single file by external identifer
    FindASingleFileByExternalIdentifer ( models::FileEntity ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// no such file
    NoSuchFile ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum FilePostResponse {
    /// created
    Created ( models::EntityEdit ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum ReleaseIdGetResponse {
    /// fetch a single release by id
    FetchASingleReleaseById ( models::ReleaseEntity ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum ReleaseLookupGetResponse {
    /// find a single release by external identifer
    FindASingleReleaseByExternalIdentifer ( models::ReleaseEntity ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// no such release
    NoSuchRelease ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum ReleasePostResponse {
    /// created
    Created ( models::EntityEdit ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum WorkIdGetResponse {
    /// fetch a single work by id
    FetchASingleWorkById ( models::WorkEntity ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}

#[derive(Debug, PartialEq)]
pub enum WorkPostResponse {
    /// created
    Created ( models::EntityEdit ) ,
    /// bad request
    BadRequest ( models::Error ) ,
    /// generic error response
    GenericErrorResponse ( models::Error ) ,
}


/// API
pub trait Api {


    fn container_id_get(&self, id: String, context: &Context) -> Box<Future<Item=ContainerIdGetResponse, Error=ApiError> + Send>;


    fn container_lookup_get(&self, issn: String, context: &Context) -> Box<Future<Item=ContainerLookupGetResponse, Error=ApiError> + Send>;


    fn container_post(&self, body: Option<models::ContainerEntity>, context: &Context) -> Box<Future<Item=ContainerPostResponse, Error=ApiError> + Send>;


    fn creator_id_get(&self, id: String, context: &Context) -> Box<Future<Item=CreatorIdGetResponse, Error=ApiError> + Send>;


    fn creator_lookup_get(&self, orcid: String, context: &Context) -> Box<Future<Item=CreatorLookupGetResponse, Error=ApiError> + Send>;


    fn creator_post(&self, body: Option<models::CreatorEntity>, context: &Context) -> Box<Future<Item=CreatorPostResponse, Error=ApiError> + Send>;


    fn editgroup_id_accept_post(&self, id: i32, context: &Context) -> Box<Future<Item=EditgroupIdAcceptPostResponse, Error=ApiError> + Send>;


    fn editgroup_id_get(&self, id: i32, context: &Context) -> Box<Future<Item=EditgroupIdGetResponse, Error=ApiError> + Send>;


    fn editgroup_post(&self, context: &Context) -> Box<Future<Item=EditgroupPostResponse, Error=ApiError> + Send>;


    fn editor_username_changelog_get(&self, username: String, context: &Context) -> Box<Future<Item=EditorUsernameChangelogGetResponse, Error=ApiError> + Send>;


    fn editor_username_get(&self, username: String, context: &Context) -> Box<Future<Item=EditorUsernameGetResponse, Error=ApiError> + Send>;


    fn file_id_get(&self, id: String, context: &Context) -> Box<Future<Item=FileIdGetResponse, Error=ApiError> + Send>;


    fn file_lookup_get(&self, sha1: String, context: &Context) -> Box<Future<Item=FileLookupGetResponse, Error=ApiError> + Send>;


    fn file_post(&self, body: Option<models::FileEntity>, context: &Context) -> Box<Future<Item=FilePostResponse, Error=ApiError> + Send>;


    fn release_id_get(&self, id: String, context: &Context) -> Box<Future<Item=ReleaseIdGetResponse, Error=ApiError> + Send>;


    fn release_lookup_get(&self, doi: String, context: &Context) -> Box<Future<Item=ReleaseLookupGetResponse, Error=ApiError> + Send>;


    fn release_post(&self, body: Option<models::ReleaseEntity>, context: &Context) -> Box<Future<Item=ReleasePostResponse, Error=ApiError> + Send>;


    fn work_id_get(&self, id: String, context: &Context) -> Box<Future<Item=WorkIdGetResponse, Error=ApiError> + Send>;


    fn work_post(&self, body: Option<models::WorkEntity>, context: &Context) -> Box<Future<Item=WorkPostResponse, Error=ApiError> + Send>;

}

/// API without a `Context`
pub trait ApiNoContext {


    fn container_id_get(&self, id: String) -> Box<Future<Item=ContainerIdGetResponse, Error=ApiError> + Send>;


    fn container_lookup_get(&self, issn: String) -> Box<Future<Item=ContainerLookupGetResponse, Error=ApiError> + Send>;


    fn container_post(&self, body: Option<models::ContainerEntity>) -> Box<Future<Item=ContainerPostResponse, Error=ApiError> + Send>;


    fn creator_id_get(&self, id: String) -> Box<Future<Item=CreatorIdGetResponse, Error=ApiError> + Send>;


    fn creator_lookup_get(&self, orcid: String) -> Box<Future<Item=CreatorLookupGetResponse, Error=ApiError> + Send>;


    fn creator_post(&self, body: Option<models::CreatorEntity>) -> Box<Future<Item=CreatorPostResponse, Error=ApiError> + Send>;


    fn editgroup_id_accept_post(&self, id: i32) -> Box<Future<Item=EditgroupIdAcceptPostResponse, Error=ApiError> + Send>;


    fn editgroup_id_get(&self, id: i32) -> Box<Future<Item=EditgroupIdGetResponse, Error=ApiError> + Send>;


    fn editgroup_post(&self) -> Box<Future<Item=EditgroupPostResponse, Error=ApiError> + Send>;


    fn editor_username_changelog_get(&self, username: String) -> Box<Future<Item=EditorUsernameChangelogGetResponse, Error=ApiError> + Send>;


    fn editor_username_get(&self, username: String) -> Box<Future<Item=EditorUsernameGetResponse, Error=ApiError> + Send>;


    fn file_id_get(&self, id: String) -> Box<Future<Item=FileIdGetResponse, Error=ApiError> + Send>;


    fn file_lookup_get(&self, sha1: String) -> Box<Future<Item=FileLookupGetResponse, Error=ApiError> + Send>;


    fn file_post(&self, body: Option<models::FileEntity>) -> Box<Future<Item=FilePostResponse, Error=ApiError> + Send>;


    fn release_id_get(&self, id: String) -> Box<Future<Item=ReleaseIdGetResponse, Error=ApiError> + Send>;


    fn release_lookup_get(&self, doi: String) -> Box<Future<Item=ReleaseLookupGetResponse, Error=ApiError> + Send>;


    fn release_post(&self, body: Option<models::ReleaseEntity>) -> Box<Future<Item=ReleasePostResponse, Error=ApiError> + Send>;


    fn work_id_get(&self, id: String) -> Box<Future<Item=WorkIdGetResponse, Error=ApiError> + Send>;


    fn work_post(&self, body: Option<models::WorkEntity>) -> Box<Future<Item=WorkPostResponse, Error=ApiError> + Send>;

}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<'a> where Self: Sized {
    /// Binds this API to a context.
    fn with_context(self: &'a Self, context: Context) -> ContextWrapper<'a, Self>;
}

impl<'a, T: Api + Sized> ContextWrapperExt<'a> for T {
    fn with_context(self: &'a T, context: Context) -> ContextWrapper<'a, T> {
         ContextWrapper::<T>::new(self, context)
    }
}

impl<'a, T: Api> ApiNoContext for ContextWrapper<'a, T> {


    fn container_id_get(&self, id: String) -> Box<Future<Item=ContainerIdGetResponse, Error=ApiError> + Send> {
        self.api().container_id_get(id, &self.context())
    }


    fn container_lookup_get(&self, issn: String) -> Box<Future<Item=ContainerLookupGetResponse, Error=ApiError> + Send> {
        self.api().container_lookup_get(issn, &self.context())
    }


    fn container_post(&self, body: Option<models::ContainerEntity>) -> Box<Future<Item=ContainerPostResponse, Error=ApiError> + Send> {
        self.api().container_post(body, &self.context())
    }


    fn creator_id_get(&self, id: String) -> Box<Future<Item=CreatorIdGetResponse, Error=ApiError> + Send> {
        self.api().creator_id_get(id, &self.context())
    }


    fn creator_lookup_get(&self, orcid: String) -> Box<Future<Item=CreatorLookupGetResponse, Error=ApiError> + Send> {
        self.api().creator_lookup_get(orcid, &self.context())
    }


    fn creator_post(&self, body: Option<models::CreatorEntity>) -> Box<Future<Item=CreatorPostResponse, Error=ApiError> + Send> {
        self.api().creator_post(body, &self.context())
    }


    fn editgroup_id_accept_post(&self, id: i32) -> Box<Future<Item=EditgroupIdAcceptPostResponse, Error=ApiError> + Send> {
        self.api().editgroup_id_accept_post(id, &self.context())
    }


    fn editgroup_id_get(&self, id: i32) -> Box<Future<Item=EditgroupIdGetResponse, Error=ApiError> + Send> {
        self.api().editgroup_id_get(id, &self.context())
    }


    fn editgroup_post(&self) -> Box<Future<Item=EditgroupPostResponse, Error=ApiError> + Send> {
        self.api().editgroup_post(&self.context())
    }


    fn editor_username_changelog_get(&self, username: String) -> Box<Future<Item=EditorUsernameChangelogGetResponse, Error=ApiError> + Send> {
        self.api().editor_username_changelog_get(username, &self.context())
    }


    fn editor_username_get(&self, username: String) -> Box<Future<Item=EditorUsernameGetResponse, Error=ApiError> + Send> {
        self.api().editor_username_get(username, &self.context())
    }


    fn file_id_get(&self, id: String) -> Box<Future<Item=FileIdGetResponse, Error=ApiError> + Send> {
        self.api().file_id_get(id, &self.context())
    }


    fn file_lookup_get(&self, sha1: String) -> Box<Future<Item=FileLookupGetResponse, Error=ApiError> + Send> {
        self.api().file_lookup_get(sha1, &self.context())
    }


    fn file_post(&self, body: Option<models::FileEntity>) -> Box<Future<Item=FilePostResponse, Error=ApiError> + Send> {
        self.api().file_post(body, &self.context())
    }


    fn release_id_get(&self, id: String) -> Box<Future<Item=ReleaseIdGetResponse, Error=ApiError> + Send> {
        self.api().release_id_get(id, &self.context())
    }


    fn release_lookup_get(&self, doi: String) -> Box<Future<Item=ReleaseLookupGetResponse, Error=ApiError> + Send> {
        self.api().release_lookup_get(doi, &self.context())
    }


    fn release_post(&self, body: Option<models::ReleaseEntity>) -> Box<Future<Item=ReleasePostResponse, Error=ApiError> + Send> {
        self.api().release_post(body, &self.context())
    }


    fn work_id_get(&self, id: String) -> Box<Future<Item=WorkIdGetResponse, Error=ApiError> + Send> {
        self.api().work_id_get(id, &self.context())
    }


    fn work_post(&self, body: Option<models::WorkEntity>) -> Box<Future<Item=WorkPostResponse, Error=ApiError> + Send> {
        self.api().work_post(body, &self.context())
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
