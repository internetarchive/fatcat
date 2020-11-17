//! Server implementation of fatcat.

#![allow(unused_imports)]

use chrono;
use futures::{self, Future};

use std::collections::HashMap;

use swagger;

use fatcat_openapi::models;
use fatcat_openapi::{
    AcceptEditgroupResponse, Api, ApiError, AuthCheckResponse, AuthOidcResponse, Context, CreateAuthTokenResponse, CreateContainerAutoBatchResponse, CreateContainerResponse,
    CreateCreatorAutoBatchResponse, CreateCreatorResponse, CreateEditgroupAnnotationResponse, CreateEditgroupResponse, CreateFileAutoBatchResponse, CreateFileResponse, CreateFilesetAutoBatchResponse,
    CreateFilesetResponse, CreateReleaseAutoBatchResponse, CreateReleaseResponse, CreateWebcaptureAutoBatchResponse, CreateWebcaptureResponse, CreateWorkAutoBatchResponse, CreateWorkResponse,
    DeleteContainerEditResponse, DeleteContainerResponse, DeleteCreatorEditResponse, DeleteCreatorResponse, DeleteFileEditResponse, DeleteFileResponse, DeleteFilesetEditResponse,
    DeleteFilesetResponse, DeleteReleaseEditResponse, DeleteReleaseResponse, DeleteWebcaptureEditResponse, DeleteWebcaptureResponse, DeleteWorkEditResponse, DeleteWorkResponse,
    GetChangelogEntryResponse, GetChangelogResponse, GetContainerEditResponse, GetContainerHistoryResponse, GetContainerRedirectsResponse, GetContainerResponse, GetContainerRevisionResponse,
    GetCreatorEditResponse, GetCreatorHistoryResponse, GetCreatorRedirectsResponse, GetCreatorReleasesResponse, GetCreatorResponse, GetCreatorRevisionResponse, GetEditgroupAnnotationsResponse,
    GetEditgroupResponse, GetEditgroupsReviewableResponse, GetEditorAnnotationsResponse, GetEditorEditgroupsResponse, GetEditorResponse, GetFileEditResponse, GetFileHistoryResponse,
    GetFileRedirectsResponse, GetFileResponse, GetFileRevisionResponse, GetFilesetEditResponse, GetFilesetHistoryResponse, GetFilesetRedirectsResponse, GetFilesetResponse, GetFilesetRevisionResponse,
    GetReleaseEditResponse, GetReleaseFilesResponse, GetReleaseFilesetsResponse, GetReleaseHistoryResponse, GetReleaseRedirectsResponse, GetReleaseResponse, GetReleaseRevisionResponse,
    GetReleaseWebcapturesResponse, GetWebcaptureEditResponse, GetWebcaptureHistoryResponse, GetWebcaptureRedirectsResponse, GetWebcaptureResponse, GetWebcaptureRevisionResponse, GetWorkEditResponse,
    GetWorkHistoryResponse, GetWorkRedirectsResponse, GetWorkReleasesResponse, GetWorkResponse, GetWorkRevisionResponse, LookupContainerResponse, LookupCreatorResponse, LookupFileResponse,
    LookupReleaseResponse, UpdateContainerResponse, UpdateCreatorResponse, UpdateEditgroupResponse, UpdateEditorResponse, UpdateFileResponse, UpdateFilesetResponse, UpdateReleaseResponse,
    UpdateWebcaptureResponse, UpdateWorkResponse,
};

#[derive(Copy, Clone)]
pub struct Server;

impl Api for Server {
    fn auth_check(&self, role: Option<String>, context: &Context) -> Box<Future<Item = AuthCheckResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("auth_check({:?}) - X-Span-ID: {:?}", role, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn auth_oidc(&self, oidc_params: models::AuthOidc, context: &Context) -> Box<Future<Item = AuthOidcResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("auth_oidc({:?}) - X-Span-ID: {:?}", oidc_params, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_auth_token(&self, editor_id: String, duration_seconds: Option<i32>, context: &Context) -> Box<Future<Item = CreateAuthTokenResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_auth_token(\"{}\", {:?}) - X-Span-ID: {:?}",
            editor_id,
            duration_seconds,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_changelog(&self, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetChangelogResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_changelog({:?}) - X-Span-ID: {:?}", limit, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_changelog_entry(&self, index: i64, context: &Context) -> Box<Future<Item = GetChangelogEntryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_changelog_entry({}) - X-Span-ID: {:?}", index, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_container(&self, editgroup_id: String, entity: models::ContainerEntity, context: &Context) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_container(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_container_auto_batch(&self, auto_batch: models::ContainerAutoBatch, context: &Context) -> Box<Future<Item = CreateContainerAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_container_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_container(&self, editgroup_id: String, ident: String, context: &Context) -> Box<Future<Item = DeleteContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_container(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_container_edit(&self, editgroup_id: String, edit_id: String, context: &Context) -> Box<Future<Item = DeleteContainerEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_container_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_container(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetContainerEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_container_edit(\"{}\") - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_container_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetContainerRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_container_redirects(\"{}\") - X-Span-ID: {:?}", ident, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_container_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetContainerRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_container_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_container(
        &self,
        issnl: Option<String>,
        wikidata_qid: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "lookup_container({:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            issnl,
            wikidata_qid,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_container(&self, editgroup_id: String, ident: String, entity: models::ContainerEntity, context: &Context) -> Box<Future<Item = UpdateContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_container(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_creator(&self, editgroup_id: String, entity: models::CreatorEntity, context: &Context) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_creator(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_creator_auto_batch(&self, auto_batch: models::CreatorAutoBatch, context: &Context) -> Box<Future<Item = CreateCreatorAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_creator_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_creator(&self, editgroup_id: String, ident: String, context: &Context) -> Box<Future<Item = DeleteCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_creator(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_creator_edit(&self, editgroup_id: String, edit_id: String, context: &Context) -> Box<Future<Item = DeleteCreatorEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_creator_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_creator(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetCreatorEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_creator_edit(\"{}\") - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetCreatorHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_creator_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetCreatorRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_creator_redirects(\"{}\") - X-Span-ID: {:?}", ident, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_releases(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_creator_releases(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_creator_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_creator_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn lookup_creator(
        &self,
        orcid: Option<String>,
        wikidata_qid: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "lookup_creator({:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            orcid,
            wikidata_qid,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_creator(&self, editgroup_id: String, ident: String, entity: models::CreatorEntity, context: &Context) -> Box<Future<Item = UpdateCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_creator(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn accept_editgroup(&self, editgroup_id: String, context: &Context) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("accept_editgroup(\"{}\") - X-Span-ID: {:?}", editgroup_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_editgroup(&self, editgroup: models::Editgroup, context: &Context) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("create_editgroup({:?}) - X-Span-ID: {:?}", editgroup, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_editgroup_annotation(
        &self,
        editgroup_id: String,
        annotation: models::EditgroupAnnotation,
        context: &Context,
    ) -> Box<Future<Item = CreateEditgroupAnnotationResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_editgroup_annotation(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            annotation,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editgroup(&self, editgroup_id: String, context: &Context) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_editgroup(\"{}\") - X-Span-ID: {:?}", editgroup_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editgroup_annotations(&self, editgroup_id: String, expand: Option<String>, context: &Context) -> Box<Future<Item = GetEditgroupAnnotationsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_editgroup_annotations(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            expand,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editgroups_reviewable(
        &self,
        expand: Option<String>,
        limit: Option<i64>,
        before: Option<chrono::DateTime<chrono::Utc>>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        context: &Context,
    ) -> Box<Future<Item = GetEditgroupsReviewableResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_editgroups_reviewable({:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            expand,
            limit,
            before,
            since,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_editgroup(&self, editgroup_id: String, editgroup: models::Editgroup, submit: Option<bool>, context: &Context) -> Box<Future<Item = UpdateEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_editgroup(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            editgroup,
            submit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editor(&self, editor_id: String, context: &Context) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_editor(\"{}\") - X-Span-ID: {:?}", editor_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editor_annotations(
        &self,
        editor_id: String,
        limit: Option<i64>,
        before: Option<chrono::DateTime<chrono::Utc>>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        context: &Context,
    ) -> Box<Future<Item = GetEditorAnnotationsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_editor_annotations(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            editor_id,
            limit,
            before,
            since,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_editor_editgroups(
        &self,
        editor_id: String,
        limit: Option<i64>,
        before: Option<chrono::DateTime<chrono::Utc>>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        context: &Context,
    ) -> Box<Future<Item = GetEditorEditgroupsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_editor_editgroups(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            editor_id,
            limit,
            before,
            since,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_editor(&self, editor_id: String, editor: models::Editor, context: &Context) -> Box<Future<Item = UpdateEditorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_editor(\"{}\", {:?}) - X-Span-ID: {:?}",
            editor_id,
            editor,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_file(&self, editgroup_id: String, entity: models::FileEntity, context: &Context) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_file(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_file_auto_batch(&self, auto_batch: models::FileAutoBatch, context: &Context) -> Box<Future<Item = CreateFileAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_file_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_file(&self, editgroup_id: String, ident: String, context: &Context) -> Box<Future<Item = DeleteFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_file(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_file_edit(&self, editgroup_id: String, edit_id: String, context: &Context) -> Box<Future<Item = DeleteFileEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_file_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_file(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetFileEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_file_edit(\"{}\") - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetFileHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_file_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetFileRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_file_redirects(\"{}\") - X-Span-ID: {:?}", ident, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_file_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFileRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_file_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
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
        expand: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "lookup_file({:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            md5,
            sha1,
            sha256,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_file(&self, editgroup_id: String, ident: String, entity: models::FileEntity, context: &Context) -> Box<Future<Item = UpdateFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_file(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_fileset(&self, editgroup_id: String, entity: models::FilesetEntity, context: &Context) -> Box<Future<Item = CreateFilesetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_fileset(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_fileset_auto_batch(&self, auto_batch: models::FilesetAutoBatch, context: &Context) -> Box<Future<Item = CreateFilesetAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_fileset_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_fileset(&self, editgroup_id: String, ident: String, context: &Context) -> Box<Future<Item = DeleteFilesetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_fileset(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_fileset_edit(&self, editgroup_id: String, edit_id: String, context: &Context) -> Box<Future<Item = DeleteFilesetEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_fileset_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_fileset(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFilesetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_fileset(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_fileset_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetFilesetEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_fileset_edit(\"{}\") - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_fileset_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetFilesetHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_fileset_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_fileset_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetFilesetRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_fileset_redirects(\"{}\") - X-Span-ID: {:?}", ident, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_fileset_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetFilesetRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_fileset_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_fileset(&self, editgroup_id: String, ident: String, entity: models::FilesetEntity, context: &Context) -> Box<Future<Item = UpdateFilesetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_fileset(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_release(&self, editgroup_id: String, entity: models::ReleaseEntity, context: &Context) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_release(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_release_auto_batch(&self, auto_batch: models::ReleaseAutoBatch, context: &Context) -> Box<Future<Item = CreateReleaseAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_release_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_release(&self, editgroup_id: String, ident: String, context: &Context) -> Box<Future<Item = DeleteReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_release(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_release_edit(&self, editgroup_id: String, edit_id: String, context: &Context) -> Box<Future<Item = DeleteReleaseEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_release_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetReleaseEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_release_edit(\"{}\") - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_files(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release_files(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_filesets(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseFilesetsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release_filesets(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetReleaseHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetReleaseRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_release_redirects(\"{}\") - X-Span-ID: {:?}", ident, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_release_webcaptures(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseWebcapturesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_release_webcaptures(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
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
        core: Option<String>,
        arxiv: Option<String>,
        jstor: Option<String>,
        ark: Option<String>,
        mag: Option<String>,
        doaj: Option<String>,
        dblp: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "lookup_release({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            doi,
            wikidata_qid,
            isbn13,
            pmid,
            pmcid,
            core,
            arxiv,
            jstor,
            ark,
            mag,
            doaj,
            dblp,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_release(&self, editgroup_id: String, ident: String, entity: models::ReleaseEntity, context: &Context) -> Box<Future<Item = UpdateReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_release(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_webcapture(&self, editgroup_id: String, entity: models::WebcaptureEntity, context: &Context) -> Box<Future<Item = CreateWebcaptureResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_webcapture(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_webcapture_auto_batch(&self, auto_batch: models::WebcaptureAutoBatch, context: &Context) -> Box<Future<Item = CreateWebcaptureAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_webcapture_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_webcapture(&self, editgroup_id: String, ident: String, context: &Context) -> Box<Future<Item = DeleteWebcaptureResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_webcapture(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_webcapture_edit(&self, editgroup_id: String, edit_id: String, context: &Context) -> Box<Future<Item = DeleteWebcaptureEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_webcapture_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_webcapture(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWebcaptureResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_webcapture(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_webcapture_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetWebcaptureEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_webcapture_edit(\"{}\") - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_webcapture_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetWebcaptureHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_webcapture_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_webcapture_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetWebcaptureRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_webcapture_redirects(\"{}\") - X-Span-ID: {:?}", ident, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_webcapture_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWebcaptureRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_webcapture_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_webcapture(&self, editgroup_id: String, ident: String, entity: models::WebcaptureEntity, context: &Context) -> Box<Future<Item = UpdateWebcaptureResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_webcapture(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_work(&self, editgroup_id: String, entity: models::WorkEntity, context: &Context) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_work(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn create_work_auto_batch(&self, auto_batch: models::WorkAutoBatch, context: &Context) -> Box<Future<Item = CreateWorkAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "create_work_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_work(&self, editgroup_id: String, ident: String, context: &Context) -> Box<Future<Item = DeleteWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_work(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn delete_work_edit(&self, editgroup_id: String, edit_id: String, context: &Context) -> Box<Future<Item = DeleteWorkEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "delete_work_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work(&self, ident: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_work(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_edit(&self, edit_id: String, context: &Context) -> Box<Future<Item = GetWorkEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_work_edit(\"{}\") - X-Span-ID: {:?}", edit_id, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_history(&self, ident: String, limit: Option<i64>, context: &Context) -> Box<Future<Item = GetWorkHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_work_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_redirects(&self, ident: String, context: &Context) -> Box<Future<Item = GetWorkRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!("get_work_redirects(\"{}\") - X-Span-ID: {:?}", ident, context.x_span_id.unwrap_or(String::from("<none>")).clone());
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_releases(&self, ident: String, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_work_releases(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn get_work_revision(&self, rev_id: String, expand: Option<String>, hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "get_work_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn update_work(&self, editgroup_id: String, ident: String, entity: models::WorkEntity, context: &Context) -> Box<Future<Item = UpdateWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "update_work(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }
}
