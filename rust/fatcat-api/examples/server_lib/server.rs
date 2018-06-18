//! Server implementation of fatcat.

#![allow(unused_imports)]

use chrono;
use futures::{self, Future};

use std::collections::HashMap;

use swagger;

use fatcat::models;
use fatcat::{AcceptEditgroupResponse, Api, ApiError, Context, CreateContainerBatchResponse, CreateContainerResponse, CreateCreatorBatchResponse, CreateCreatorResponse, CreateEditgroupResponse,
             CreateFileBatchResponse, CreateFileResponse, CreateReleaseBatchResponse, CreateReleaseResponse, CreateWorkBatchResponse, CreateWorkResponse, GetContainerResponse,
             GetCreatorReleasesResponse, GetCreatorResponse, GetEditgroupResponse, GetEditorChangelogResponse, GetEditorResponse, GetFileResponse, GetReleaseFilesResponse, GetReleaseResponse,
             GetWorkReleasesResponse, GetWorkResponse, LookupContainerResponse, LookupCreatorResponse, LookupFileResponse, LookupReleaseResponse};

#[derive(Copy, Clone)]
pub struct Server;

impl Api for Server {
    fn accept_editgroup(&self, id: i64, context: &Context) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("accept_editgroup({}) - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_container(&self, entity: models::ContainerEntity, context: &Context) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_container({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_container_batch(&self, entity_list: &Vec<models::ContainerEntity>, context: &Context) -> Box<Future<Item = CreateContainerBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_container_batch({:?}) - X-Span-ID: {:?}",
            entity_list,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_creator(&self, entity: models::CreatorEntity, context: &Context) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_creator({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_creator_batch(&self, entity_list: &Vec<models::CreatorEntity>, context: &Context) -> Box<Future<Item = CreateCreatorBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_creator_batch({:?}) - X-Span-ID: {:?}", entity_list, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_editgroup(&self, entity: models::Editgroup, context: &Context) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_editgroup({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_file(&self, entity: models::FileEntity, context: &Context) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_file({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_file_batch(&self, entity_list: &Vec<models::FileEntity>, context: &Context) -> Box<Future<Item = CreateFileBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_file_batch({:?}) - X-Span-ID: {:?}", entity_list, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_release(&self, entity: models::ReleaseEntity, context: &Context) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_release({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_release_batch(&self, entity_list: &Vec<models::ReleaseEntity>, context: &Context) -> Box<Future<Item = CreateReleaseBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_release_batch({:?}) - X-Span-ID: {:?}", entity_list, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_work(&self, entity: models::WorkEntity, context: &Context) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_work({:?}) - X-Span-ID: {:?}", entity, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_work_batch(&self, entity_list: &Vec<models::WorkEntity>, context: &Context) -> Box<Future<Item = CreateWorkBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_work_batch({:?}) - X-Span-ID: {:?}", entity_list, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container(&self, id: String, context: &Context) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_container(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator(&self, id: String, context: &Context) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_creator(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_releases(&self, id: String, context: &Context) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_creator_releases(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editgroup(&self, id: i64, context: &Context) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_editgroup({}) - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editor(&self, username: String, context: &Context) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_editor(\"{}\") - X-Span-ID: {:?}", username, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editor_changelog(&self, username: String, context: &Context) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_editor_changelog(\"{}\") - X-Span-ID: {:?}", username, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file(&self, id: String, context: &Context) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_file(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release(&self, id: String, context: &Context) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_release(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_files(&self, id: String, context: &Context) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_release_files(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work(&self, id: String, context: &Context) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_work(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_releases(&self, id: String, context: &Context) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_work_releases(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_container(&self, issnl: String, context: &Context) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("lookup_container(\"{}\") - X-Span-ID: {:?}", issnl, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_creator(&self, orcid: String, context: &Context) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("lookup_creator(\"{}\") - X-Span-ID: {:?}", orcid, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_file(&self, sha1: String, context: &Context) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("lookup_file(\"{}\") - X-Span-ID: {:?}", sha1, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_release(&self, doi: String, context: &Context) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("lookup_release(\"{}\") - X-Span-ID: {:?}", doi, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }
}
