//! Server implementation of fatcat.

#![allow(unused_imports)]

use chrono;
use futures::{self, Future};

use std::collections::HashMap;

use swagger;

use fatcat::models;
use fatcat::{Api, ApiError, ContainerIdGetResponse, ContainerLookupGetResponse,
             ContainerPostResponse, Context, CreatorIdGetResponse, CreatorLookupGetResponse,
             CreatorPostResponse, EditgroupIdAcceptPostResponse, EditgroupIdGetResponse,
             EditgroupPostResponse, EditorUsernameChangelogGetResponse, EditorUsernameGetResponse,
             FileIdGetResponse, FileLookupGetResponse, FilePostResponse, ReleaseIdGetResponse,
             ReleaseLookupGetResponse, ReleasePostResponse, WorkIdGetResponse, WorkPostResponse};

#[derive(Copy, Clone)]
pub struct Server;

impl Api for Server {
    fn container_id_get(
        &self,
        id: String,
        context: &Context,
    ) -> Box<Future<Item = ContainerIdGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "container_id_get(\"{}\") - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn container_lookup_get(
        &self,
        issn: String,
        context: &Context,
    ) -> Box<Future<Item = ContainerLookupGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "container_lookup_get(\"{}\") - X-Span-ID: {:?}",
            issn,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn container_post(
        &self,
        body: Option<models::ContainerEntity>,
        context: &Context,
    ) -> Box<Future<Item = ContainerPostResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "container_post({:?}) - X-Span-ID: {:?}",
            body,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn creator_id_get(
        &self,
        id: String,
        context: &Context,
    ) -> Box<Future<Item = CreatorIdGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "creator_id_get(\"{}\") - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn creator_lookup_get(
        &self,
        orcid: String,
        context: &Context,
    ) -> Box<Future<Item = CreatorLookupGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "creator_lookup_get(\"{}\") - X-Span-ID: {:?}",
            orcid,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn creator_post(
        &self,
        body: Option<models::CreatorEntity>,
        context: &Context,
    ) -> Box<Future<Item = CreatorPostResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "creator_post({:?}) - X-Span-ID: {:?}",
            body,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editgroup_id_accept_post(
        &self,
        id: i32,
        context: &Context,
    ) -> Box<Future<Item = EditgroupIdAcceptPostResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "editgroup_id_accept_post({}) - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editgroup_id_get(
        &self,
        id: i32,
        context: &Context,
    ) -> Box<Future<Item = EditgroupIdGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "editgroup_id_get({}) - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editgroup_post(
        &self,
        context: &Context,
    ) -> Box<Future<Item = EditgroupPostResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "editgroup_post() - X-Span-ID: {:?}",
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editor_username_changelog_get(
        &self,
        username: String,
        context: &Context,
    ) -> Box<Future<Item = EditorUsernameChangelogGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "editor_username_changelog_get(\"{}\") - X-Span-ID: {:?}",
            username,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editor_username_get(
        &self,
        username: String,
        context: &Context,
    ) -> Box<Future<Item = EditorUsernameGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "editor_username_get(\"{}\") - X-Span-ID: {:?}",
            username,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn file_id_get(
        &self,
        id: String,
        context: &Context,
    ) -> Box<Future<Item = FileIdGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "file_id_get(\"{}\") - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn file_lookup_get(
        &self,
        sha1: String,
        context: &Context,
    ) -> Box<Future<Item = FileLookupGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "file_lookup_get(\"{}\") - X-Span-ID: {:?}",
            sha1,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn file_post(
        &self,
        body: Option<models::FileEntity>,
        context: &Context,
    ) -> Box<Future<Item = FilePostResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "file_post({:?}) - X-Span-ID: {:?}",
            body,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn release_id_get(
        &self,
        id: String,
        context: &Context,
    ) -> Box<Future<Item = ReleaseIdGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "release_id_get(\"{}\") - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn release_lookup_get(
        &self,
        doi: String,
        context: &Context,
    ) -> Box<Future<Item = ReleaseLookupGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "release_lookup_get(\"{}\") - X-Span-ID: {:?}",
            doi,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn release_post(
        &self,
        body: Option<models::ReleaseEntity>,
        context: &Context,
    ) -> Box<Future<Item = ReleasePostResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "release_post({:?}) - X-Span-ID: {:?}",
            body,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn work_id_get(
        &self,
        id: String,
        context: &Context,
    ) -> Box<Future<Item = WorkIdGetResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "work_id_get(\"{}\") - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn work_post(
        &self,
        body: Option<models::WorkEntity>,
        context: &Context,
    ) -> Box<Future<Item = WorkPostResponse, Error = ApiError>> {
        let context = context.clone();
        println!(
            "work_post({:?}) - X-Span-ID: {:?}",
            body,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }
}
