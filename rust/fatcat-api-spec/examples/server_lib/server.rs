//! Server implementation of fatcat.

#![allow(unused_imports)]

use chrono;
use futures::{self, Future};

use std::collections::HashMap;

use swagger;

use fatcat::models;
use fatcat::{
    AcceptEditgroupResponse, Api, ApiError, Context, CreateContainerBatchResponse, CreateContainerResponse, CreateCreatorBatchResponse, CreateCreatorResponse, CreateEditgroupResponse,
    CreateFileBatchResponse, CreateFileResponse, CreateReleaseBatchResponse, CreateReleaseResponse, CreateWorkBatchResponse, CreateWorkResponse, DeleteContainerEditResponse, DeleteContainerResponse,
    DeleteCreatorEditResponse, DeleteCreatorResponse, DeleteFileEditResponse, DeleteFileResponse, DeleteReleaseEditResponse, DeleteReleaseResponse, DeleteWorkEditResponse, DeleteWorkResponse,
    GetChangelogEntryResponse, GetChangelogResponse, GetContainerEditResponse, GetContainerHistoryResponse, GetContainerRedirectsResponse, GetContainerResponse, GetContainerRevisionResponse,
    GetCreatorEditResponse, GetCreatorHistoryResponse, GetCreatorRedirectsResponse, GetCreatorReleasesResponse, GetCreatorResponse, GetCreatorRevisionResponse, GetEditgroupResponse,
    GetEditorChangelogResponse, GetEditorResponse, GetFileEditResponse, GetFileHistoryResponse, GetFileRedirectsResponse, GetFileResponse, GetFileRevisionResponse, GetReleaseEditResponse,
    GetReleaseFilesResponse, GetReleaseHistoryResponse, GetReleaseRedirectsResponse, GetReleaseResponse, GetReleaseRevisionResponse, GetStatsResponse, GetWorkEditResponse, GetWorkHistoryResponse,
    GetWorkRedirectsResponse, GetWorkReleasesResponse, GetWorkResponse, GetWorkRevisionResponse, LookupContainerResponse, LookupCreatorResponse, LookupFileResponse, LookupReleaseResponse,
    UpdateContainerResponse, UpdateCreatorResponse, UpdateFileResponse, UpdateReleaseResponse, UpdateWorkResponse,
};

#[derive(Copy, Clone)]
pub struct Server;

impl Api for Server {
    fn create_container(&self, entity: models::ContainerEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_container({:?}, {:?}) - X-Span-ID: {:?}",
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_container_batch(
        &self,
        entity_list: &Vec<models::ContainerEntity>,
        autoaccept: Option<bool>,
        editgroup: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateContainerBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_container_batch({:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            entity_list,
            autoaccept,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_container(&self, id: String, editgroup: Option<String>, context: &Context) -> Box<Future<Item = DeleteContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_container(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_container_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = DeleteContainerEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("delete_container_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_container(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = GetContainerEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_container_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container_history(&self, id: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_container_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container_redirects(&self, id: String, context: &Context) -> Box<Future<Item = GetContainerRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_container_redirects(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container_revision(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetContainerRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_container_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_container(&self, issnl: Option<String>, wikidata_qid: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "lookup_container({:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            issnl,
            wikidata_qid,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_container(&self, id: String, entity: models::ContainerEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = UpdateContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_container(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_creator(&self, entity: models::CreatorEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_creator({:?}, {:?}) - X-Span-ID: {:?}",
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_creator_batch(
        &self,
        entity_list: &Vec<models::CreatorEntity>,
        autoaccept: Option<bool>,
        editgroup: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateCreatorBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_creator_batch({:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            entity_list,
            autoaccept,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_creator(&self, id: String, editgroup: Option<String>, context: &Context) -> Box<Future<Item = DeleteCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_creator(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_creator_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = DeleteCreatorEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("delete_creator_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_creator(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = GetCreatorEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_creator_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_history(&self, id: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetCreatorHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_creator_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_redirects(&self, id: String, context: &Context) -> Box<Future<Item = GetCreatorRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_creator_redirects(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_releases(&self, id: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_creator_releases(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_revision(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_creator_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_creator(&self, orcid: Option<String>, wikidata_qid: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "lookup_creator({:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            orcid,
            wikidata_qid,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_creator(&self, id: String, entity: models::CreatorEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = UpdateCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_creator(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editor(&self, id: String, context: &Context) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_editor(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editor_changelog(&self, id: String, context: &Context) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_editor_changelog(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_stats(&self, more: Option<String>, context: &Context) -> Box<Future<Item = GetStatsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_stats({:?}) - X-Span-ID: {:?}", more, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn accept_editgroup(&self, id: String, context: &Context) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("accept_editgroup(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_editgroup(&self, editgroup: models::Editgroup, context: &Context) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_editgroup({:?}) - X-Span-ID: {:?}", editgroup, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_changelog(&self, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetChangelogResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_changelog({:?}) - X-Span-ID: {:?}", limit, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_changelog_entry(&self, id: i64, context: &Context) -> Box<Future<Item = GetChangelogEntryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_changelog_entry({}) - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editgroup(&self, id: String, context: &Context) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_editgroup(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_file(&self, entity: models::FileEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_file({:?}, {:?}) - X-Span-ID: {:?}",
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_file_batch(
        &self,
        entity_list: &Vec<models::FileEntity>,
        autoaccept: Option<bool>,
        editgroup: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateFileBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_file_batch({:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            entity_list,
            autoaccept,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_file(&self, id: String, editgroup: Option<String>, context: &Context) -> Box<Future<Item = DeleteFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_file(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_file_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = DeleteFileEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("delete_file_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_file(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = GetFileEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_file_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file_history(&self, id: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetFileHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_file_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file_redirects(&self, id: String, context: &Context) -> Box<Future<Item = GetFileRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_file_redirects(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file_revision(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFileRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_file_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_file(
        &self,
        md5: Option<String>,
        sha1: Option<String>,
        sha256: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "lookup_file({:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            md5,
            sha1,
            sha256,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_file(&self, id: String, entity: models::FileEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = UpdateFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_file(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_release(&self, entity: models::ReleaseEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_release({:?}, {:?}) - X-Span-ID: {:?}",
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_release_batch(
        &self,
        entity_list: &Vec<models::ReleaseEntity>,
        autoaccept: Option<bool>,
        editgroup: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateReleaseBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_release_batch({:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            entity_list,
            autoaccept,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_work(&self, entity: models::WorkEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_work({:?}, {:?}) - X-Span-ID: {:?}",
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_release(&self, id: String, editgroup: Option<String>, context: &Context) -> Box<Future<Item = DeleteReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_release(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_release_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = DeleteReleaseEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("delete_release_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = GetReleaseEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_release_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_files(&self, id: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release_files(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_history(&self, id: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetReleaseHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_redirects(&self, id: String, context: &Context) -> Box<Future<Item = GetReleaseRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_release_redirects(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_revision(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_release(
        &self,
        doi: Option<String>,
        wikidata_qid: Option<String>,
        isbn13: Option<String>,
        pmid: Option<String>,
        pmcid: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "lookup_release({:?}, {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            doi,
            wikidata_qid,
            isbn13,
            pmid,
            pmcid,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_release(&self, id: String, entity: models::ReleaseEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = UpdateReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_release(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_work_batch(
        &self,
        entity_list: &Vec<models::WorkEntity>,
        autoaccept: Option<bool>,
        editgroup: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateWorkBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_work_batch({:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            entity_list,
            autoaccept,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_work(&self, id: String, editgroup: Option<String>, context: &Context) -> Box<Future<Item = DeleteWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_work(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_work_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = DeleteWorkEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("delete_work_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_work(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_edit(&self, edit_id: i64, context: &Context) -> Box<Future<Item = GetWorkEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_work_edit({}) - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_history(&self, id: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetWorkHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_work_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_redirects(&self, id: String, context: &Context) -> Box<Future<Item = GetWorkRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_work_redirects(\"{}\") - X-Span-ID: {:?}", id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_releases(&self, id: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_work_releases(\"{}\", {:?}) - X-Span-ID: {:?}",
            id,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_revision(&self, id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_work_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_work(&self, id: String, entity: models::WorkEntity, editgroup: Option<String>, context: &Context) -> Box<Future<Item = UpdateWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_work(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            id,
            entity,
            editgroup,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }
}
