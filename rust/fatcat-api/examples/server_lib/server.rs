//! Server implementation of fatcat.

#![allow(unused_imports)]

use chrono;
use futures::{self, Future};

use std::collections::HashMap;

use swagger;

use fatcat::models;
use fatcat::{Api, ApiError, ContainerBatchPostResponse, ContainerIdGetResponse, ContainerLookupGetResponse, ContainerPostResponse, Context, CreatorBatchPostResponse, CreatorIdGetResponse,
             CreatorLookupGetResponse, CreatorPostResponse, EditgroupIdAcceptPostResponse, EditgroupIdGetResponse, EditgroupPostResponse, EditorUsernameChangelogGetResponse,
             EditorUsernameGetResponse, FileBatchPostResponse, FileIdGetResponse, FileLookupGetResponse, FilePostResponse, ReleaseBatchPostResponse, ReleaseIdGetResponse, ReleaseLookupGetResponse,
             ReleasePostResponse, WorkBatchPostResponse, WorkIdGetResponse, WorkPostResponse};

#[derive(Copy, Clone)]
pub struct Server;

impl Api for Server {
    fn container_batch_post(&self, entity_list: &Vec<models::ContainerEntity>, context: &Context) -> Box<Future<Item = ContainerBatchPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("container_batch_post({:?}) - X-Span-ID: {:?}", entity_list, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn container_id_get(&self, id: String, context: &Context) -> Box<Future<Item = ContainerIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("container_id_get(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn container_lookup_get(&self, issnl: String, context: &Context) -> Box<Future<Item = ContainerLookupGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("container_lookup_get(\"{}\") - X-Span-ID: {:?}", issnl, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn container_post(&self, entity: models::ContainerEntity, context: &Context) -> Box<Future<Item = ContainerPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("container_post({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn creator_batch_post(&self, entity_list: &Vec<models::CreatorEntity>, context: &Context) -> Box<Future<Item = CreatorBatchPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("creator_batch_post({:?}) - X-Span-ID: {:?}", entity_list, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn creator_id_get(&self, id: String, context: &Context) -> Box<Future<Item = CreatorIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("creator_id_get(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn creator_lookup_get(&self, orcid: String, context: &Context) -> Box<Future<Item = CreatorLookupGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("creator_lookup_get(\"{}\") - X-Span-ID: {:?}", orcid, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn creator_post(&self, entity: models::CreatorEntity, context: &Context) -> Box<Future<Item = CreatorPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("creator_post({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editgroup_id_accept_post(&self, id: i64, context: &Context) -> Box<Future<Item = EditgroupIdAcceptPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("editgroup_id_accept_post({}) - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editgroup_id_get(&self, id: i64, context: &Context) -> Box<Future<Item = EditgroupIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("editgroup_id_get({}) - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editgroup_post(&self, entity: models::Editgroup, context: &Context) -> Box<Future<Item = EditgroupPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("editgroup_post({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editor_username_changelog_get(&self, username: String, context: &Context) -> Box<Future<Item = EditorUsernameChangelogGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "editor_username_changelog_get(\"{}\") - X-Span-ID: {:?}",
            username,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editor_username_get(&self, username: String, context: &Context) -> Box<Future<Item = EditorUsernameGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("editor_username_get(\"{}\") - X-Span-ID: {:?}", username, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn file_batch_post(&self, entity_list: &Vec<models::FileEntity>, context: &Context) -> Box<Future<Item = FileBatchPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("file_batch_post({:?}) - X-Span-ID: {:?}", entity_list, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn file_id_get(&self, id: String, context: &Context) -> Box<Future<Item = FileIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("file_id_get(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn file_lookup_get(&self, sha1: String, context: &Context) -> Box<Future<Item = FileLookupGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("file_lookup_get(\"{}\") - X-Span-ID: {:?}", sha1, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn file_post(&self, entity: models::FileEntity, context: &Context) -> Box<Future<Item = FilePostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("file_post({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn release_batch_post(&self, entity_list: &Vec<models::ReleaseEntity>, context: &Context) -> Box<Future<Item = ReleaseBatchPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("release_batch_post({:?}) - X-Span-ID: {:?}", entity_list, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn release_id_get(&self, id: String, context: &Context) -> Box<Future<Item = ReleaseIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("release_id_get(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn release_lookup_get(&self, doi: String, context: &Context) -> Box<Future<Item = ReleaseLookupGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("release_lookup_get(\"{}\") - X-Span-ID: {:?}", doi, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn release_post(&self, entity: models::ReleaseEntity, context: &Context) -> Box<Future<Item = ReleasePostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("release_post({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn work_batch_post(&self, entity_list: &Vec<models::WorkEntity>, context: &Context) -> Box<Future<Item = WorkBatchPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("work_batch_post({:?}) - X-Span-ID: {:?}", entity_list, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn work_id_get(&self, id: String, context: &Context) -> Box<Future<Item = WorkIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("work_id_get(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn work_post(&self, entity: models::WorkEntity, context: &Context) -> Box<Future<Item = WorkPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("work_post({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }
}
