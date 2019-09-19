//! Main library entry point for fatcat_openapi implementation.

#![allow(unused_imports)]

mod errors {
    error_chain::error_chain! {}
}

pub use self::errors::*;

use chrono;
use futures::{future, Future, Stream};
use hyper::server::conn::Http;
use hyper::service::MakeService as _;
use log::info;
use openssl::ssl::SslAcceptorBuilder;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use swagger;
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use swagger::{Has, XSpanIdString};
use tokio::net::TcpListener;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use tokio_openssl::SslAcceptorExt;

use fatcat_openapi::models;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub fn create(addr: &str, https: bool) -> Box<dyn Future<Item = (), Error = ()> + Send> {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new();

    let service_fn = MakeService::new(server);

    let service_fn = MakeAllowAllAuthenticator::new(service_fn, "cosmo");

    let service_fn =
        fatcat_openapi::server::context::MakeAddContext::<_, EmptyContext>::new(service_fn);

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls())
                .expect("Failed to create SSL Acceptor");

            // Server authentication
            ssl.set_private_key_file("examples/server-key.pem", SslFiletype::PEM)
                .expect("Failed to set private key");
            ssl.set_certificate_chain_file("examples/server-chain.pem")
                .expect("Failed to set cerificate chain");
            ssl.check_private_key()
                .expect("Failed to check private key");

            let tls_acceptor = ssl.build();
            let service_fn = Arc::new(Mutex::new(service_fn));
            let tls_listener = TcpListener::bind(&addr)
                .unwrap()
                .incoming()
                .for_each(move |tcp| {
                    let addr = tcp.peer_addr().expect("Unable to get remote address");

                    let service_fn = service_fn.clone();

                    hyper::rt::spawn(tls_acceptor.accept_async(tcp).map_err(|_| ()).and_then(
                        move |tls| {
                            let ms = {
                                let mut service_fn = service_fn.lock().unwrap();
                                service_fn.make_service(&addr)
                            };

                            ms.and_then(move |service| Http::new().serve_connection(tls, service))
                                .map_err(|_| ())
                        },
                    ));

                    Ok(())
                })
                .map_err(|_| ());

            Box::new(tls_listener)
        }
    } else {
        // Using HTTP
        Box::new(
            hyper::server::Server::bind(&addr)
                .serve(service_fn)
                .map_err(|e| panic!("{:?}", e)),
        )
    }
}

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server {
            marker: PhantomData,
        }
    }
}

use fatcat_openapi::server::MakeService;
use fatcat_openapi::{
    AcceptEditgroupResponse, Api, ApiError, AuthCheckResponse, AuthOidcResponse,
    CreateAuthTokenResponse, CreateContainerAutoBatchResponse, CreateContainerResponse,
    CreateCreatorAutoBatchResponse, CreateCreatorResponse, CreateEditgroupAnnotationResponse,
    CreateEditgroupResponse, CreateFileAutoBatchResponse, CreateFileResponse,
    CreateFilesetAutoBatchResponse, CreateFilesetResponse, CreateReleaseAutoBatchResponse,
    CreateReleaseResponse, CreateWebcaptureAutoBatchResponse, CreateWebcaptureResponse,
    CreateWorkAutoBatchResponse, CreateWorkResponse, DeleteContainerEditResponse,
    DeleteContainerResponse, DeleteCreatorEditResponse, DeleteCreatorResponse,
    DeleteFileEditResponse, DeleteFileResponse, DeleteFilesetEditResponse, DeleteFilesetResponse,
    DeleteReleaseEditResponse, DeleteReleaseResponse, DeleteWebcaptureEditResponse,
    DeleteWebcaptureResponse, DeleteWorkEditResponse, DeleteWorkResponse,
    GetChangelogEntryResponse, GetChangelogResponse, GetContainerEditResponse,
    GetContainerHistoryResponse, GetContainerRedirectsResponse, GetContainerResponse,
    GetContainerRevisionResponse, GetCreatorEditResponse, GetCreatorHistoryResponse,
    GetCreatorRedirectsResponse, GetCreatorReleasesResponse, GetCreatorResponse,
    GetCreatorRevisionResponse, GetEditgroupAnnotationsResponse, GetEditgroupResponse,
    GetEditgroupsReviewableResponse, GetEditorAnnotationsResponse, GetEditorEditgroupsResponse,
    GetEditorResponse, GetFileEditResponse, GetFileHistoryResponse, GetFileRedirectsResponse,
    GetFileResponse, GetFileRevisionResponse, GetFilesetEditResponse, GetFilesetHistoryResponse,
    GetFilesetRedirectsResponse, GetFilesetResponse, GetFilesetRevisionResponse,
    GetReleaseEditResponse, GetReleaseFilesResponse, GetReleaseFilesetsResponse,
    GetReleaseHistoryResponse, GetReleaseRedirectsResponse, GetReleaseResponse,
    GetReleaseRevisionResponse, GetReleaseWebcapturesResponse, GetWebcaptureEditResponse,
    GetWebcaptureHistoryResponse, GetWebcaptureRedirectsResponse, GetWebcaptureResponse,
    GetWebcaptureRevisionResponse, GetWorkEditResponse, GetWorkHistoryResponse,
    GetWorkRedirectsResponse, GetWorkReleasesResponse, GetWorkResponse, GetWorkRevisionResponse,
    LookupContainerResponse, LookupCreatorResponse, LookupFileResponse, LookupReleaseResponse,
    UpdateContainerResponse, UpdateCreatorResponse, UpdateEditgroupResponse, UpdateEditorResponse,
    UpdateFileResponse, UpdateFilesetResponse, UpdateReleaseResponse, UpdateWebcaptureResponse,
    UpdateWorkResponse,
};

impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString>,
{
    fn accept_editgroup(
        &self,
        editgroup_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "accept_editgroup(\"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn auth_check(
        &self,
        role: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = AuthCheckResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "auth_check({:?}) - X-Span-ID: {:?}",
            role,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn auth_oidc(
        &self,
        oidc_params: models::AuthOidc,
        context: &C,
    ) -> Box<dyn Future<Item = AuthOidcResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "auth_oidc({:?}) - X-Span-ID: {:?}",
            oidc_params,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_auth_token(
        &self,
        editor_id: String,
        duration_seconds: Option<i32>,
        context: &C,
    ) -> Box<dyn Future<Item = CreateAuthTokenResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_auth_token(\"{}\", {:?}) - X-Span-ID: {:?}",
            editor_id,
            duration_seconds,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_container(
        &self,
        editgroup_id: String,
        entity: models::ContainerEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_container(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_container_auto_batch(
        &self,
        auto_batch: models::ContainerAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateContainerAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_container_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_creator(
        &self,
        editgroup_id: String,
        entity: models::CreatorEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_creator(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_creator_auto_batch(
        &self,
        auto_batch: models::CreatorAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateCreatorAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_creator_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_editgroup(
        &self,
        editgroup: models::Editgroup,
        context: &C,
    ) -> Box<dyn Future<Item = CreateEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_editgroup({:?}) - X-Span-ID: {:?}",
            editgroup,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_editgroup_annotation(
        &self,
        editgroup_id: String,
        annotation: models::EditgroupAnnotation,
        context: &C,
    ) -> Box<dyn Future<Item = CreateEditgroupAnnotationResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_editgroup_annotation(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            annotation,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_file(
        &self,
        editgroup_id: String,
        entity: models::FileEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_file(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_file_auto_batch(
        &self,
        auto_batch: models::FileAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateFileAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_file_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_fileset(
        &self,
        editgroup_id: String,
        entity: models::FilesetEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateFilesetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_fileset(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_fileset_auto_batch(
        &self,
        auto_batch: models::FilesetAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateFilesetAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_fileset_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_release(
        &self,
        editgroup_id: String,
        entity: models::ReleaseEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_release(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_release_auto_batch(
        &self,
        auto_batch: models::ReleaseAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateReleaseAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_release_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_webcapture(
        &self,
        editgroup_id: String,
        entity: models::WebcaptureEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateWebcaptureResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_webcapture(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_webcapture_auto_batch(
        &self,
        auto_batch: models::WebcaptureAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateWebcaptureAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_webcapture_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_work(
        &self,
        editgroup_id: String,
        entity: models::WorkEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_work(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn create_work_auto_batch(
        &self,
        auto_batch: models::WorkAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateWorkAutoBatchResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "create_work_auto_batch({:?}) - X-Span-ID: {:?}",
            auto_batch,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_container(
        &self,
        editgroup_id: String,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_container(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_container_edit(
        &self,
        editgroup_id: String,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteContainerEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_container_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_creator(
        &self,
        editgroup_id: String,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_creator(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_creator_edit(
        &self,
        editgroup_id: String,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteCreatorEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_creator_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_file(
        &self,
        editgroup_id: String,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_file(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_file_edit(
        &self,
        editgroup_id: String,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteFileEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_file_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_fileset(
        &self,
        editgroup_id: String,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteFilesetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_fileset(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_fileset_edit(
        &self,
        editgroup_id: String,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteFilesetEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_fileset_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_release(
        &self,
        editgroup_id: String,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_release(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_release_edit(
        &self,
        editgroup_id: String,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteReleaseEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_release_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_webcapture(
        &self,
        editgroup_id: String,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteWebcaptureResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_webcapture(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_webcapture_edit(
        &self,
        editgroup_id: String,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteWebcaptureEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_webcapture_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_work(
        &self,
        editgroup_id: String,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_work(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn delete_work_edit(
        &self,
        editgroup_id: String,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteWorkEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "delete_work_edit(\"{}\", \"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_changelog(
        &self,
        limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetChangelogResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_changelog({:?}) - X-Span-ID: {:?}",
            limit,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_changelog_entry(
        &self,
        index: i64,
        context: &C,
    ) -> Box<dyn Future<Item = GetChangelogEntryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_changelog_entry({}) - X-Span-ID: {:?}",
            index,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_container(
        &self,
        ident: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_container(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_container_edit(
        &self,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_container_edit(\"{}\") - X-Span-ID: {:?}",
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_container_history(
        &self,
        ident: String,
        limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_container_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_container_redirects(
        &self,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_container_redirects(\"{}\") - X-Span-ID: {:?}",
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_container_revision(
        &self,
        rev_id: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_container_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_creator(
        &self,
        ident: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_creator(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_creator_edit(
        &self,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_creator_edit(\"{}\") - X-Span-ID: {:?}",
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_creator_history(
        &self,
        ident: String,
        limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_creator_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_creator_redirects(
        &self,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_creator_redirects(\"{}\") - X-Span-ID: {:?}",
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_creator_releases(
        &self,
        ident: String,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_creator_releases(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_creator_revision(
        &self,
        rev_id: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_creator_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_editgroup(
        &self,
        editgroup_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_editgroup(\"{}\") - X-Span-ID: {:?}",
            editgroup_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_editgroup_annotations(
        &self,
        editgroup_id: String,
        expand: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditgroupAnnotationsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_editgroup_annotations(\"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            expand,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_editgroups_reviewable(
        &self,
        expand: Option<String>,
        limit: Option<i64>,
        before: Option<chrono::DateTime<chrono::Utc>>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditgroupsReviewableResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_editgroups_reviewable({:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            expand,
            limit,
            before,
            since,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_editor(
        &self,
        editor_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_editor(\"{}\") - X-Span-ID: {:?}",
            editor_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_editor_annotations(
        &self,
        editor_id: String,
        limit: Option<i64>,
        before: Option<chrono::DateTime<chrono::Utc>>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditorAnnotationsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_editor_annotations(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            editor_id,
            limit,
            before,
            since,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_editor_editgroups(
        &self,
        editor_id: String,
        limit: Option<i64>,
        before: Option<chrono::DateTime<chrono::Utc>>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditorEditgroupsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_editor_editgroups(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            editor_id,
            limit,
            before,
            since,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_file(
        &self,
        ident: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_file(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_file_edit(
        &self,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_file_edit(\"{}\") - X-Span-ID: {:?}",
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_file_history(
        &self,
        ident: String,
        limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_file_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_file_redirects(
        &self,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_file_redirects(\"{}\") - X-Span-ID: {:?}",
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_file_revision(
        &self,
        rev_id: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_file_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_fileset(
        &self,
        ident: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_fileset(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_fileset_edit(
        &self,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_fileset_edit(\"{}\") - X-Span-ID: {:?}",
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_fileset_history(
        &self,
        ident: String,
        limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_fileset_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_fileset_redirects(
        &self,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_fileset_redirects(\"{}\") - X-Span-ID: {:?}",
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_fileset_revision(
        &self,
        rev_id: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_fileset_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_release(
        &self,
        ident: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_release(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_release_edit(
        &self,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_release_edit(\"{}\") - X-Span-ID: {:?}",
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_release_files(
        &self,
        ident: String,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_release_files(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_release_filesets(
        &self,
        ident: String,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseFilesetsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_release_filesets(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_release_history(
        &self,
        ident: String,
        limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_release_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_release_redirects(
        &self,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_release_redirects(\"{}\") - X-Span-ID: {:?}",
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_release_revision(
        &self,
        rev_id: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_release_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_release_webcaptures(
        &self,
        ident: String,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseWebcapturesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_release_webcaptures(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_webcapture(
        &self,
        ident: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_webcapture(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_webcapture_edit(
        &self,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_webcapture_edit(\"{}\") - X-Span-ID: {:?}",
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_webcapture_history(
        &self,
        ident: String,
        limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_webcapture_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_webcapture_redirects(
        &self,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_webcapture_redirects(\"{}\") - X-Span-ID: {:?}",
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_webcapture_revision(
        &self,
        rev_id: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_webcapture_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_work(
        &self,
        ident: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_work(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ident,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_work_edit(
        &self,
        edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkEditResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_work_edit(\"{}\") - X-Span-ID: {:?}",
            edit_id,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_work_history(
        &self,
        ident: String,
        limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkHistoryResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_work_history(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            limit,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_work_redirects(
        &self,
        ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkRedirectsResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_work_redirects(\"{}\") - X-Span-ID: {:?}",
            ident,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_work_releases(
        &self,
        ident: String,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_work_releases(\"{}\", {:?}) - X-Span-ID: {:?}",
            ident,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn get_work_revision(
        &self,
        rev_id: String,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkRevisionResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "get_work_revision(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            rev_id,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn lookup_container(
        &self,
        issnl: Option<String>,
        wikidata_qid: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = LookupContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "lookup_container({:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            issnl,
            wikidata_qid,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn lookup_creator(
        &self,
        orcid: Option<String>,
        wikidata_qid: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = LookupCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "lookup_creator({:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            orcid,
            wikidata_qid,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn lookup_file(
        &self,
        md5: Option<String>,
        sha1: Option<String>,
        sha256: Option<String>,
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = LookupFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "lookup_file({:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            md5,
            sha1,
            sha256,
            expand,
            hide,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
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
        expand: Option<String>,
        hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = LookupReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!("lookup_release({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}", doi, wikidata_qid, isbn13, pmid, pmcid, core, arxiv, jstor, ark, mag, expand, hide, context.get().0.clone());
        Box::new(future::err("Generic failure".into()))
    }

    fn update_container(
        &self,
        editgroup_id: String,
        ident: String,
        entity: models::ContainerEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateContainerResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "update_container(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn update_creator(
        &self,
        editgroup_id: String,
        ident: String,
        entity: models::CreatorEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateCreatorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "update_creator(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn update_editgroup(
        &self,
        editgroup_id: String,
        editgroup: models::Editgroup,
        submit: Option<bool>,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateEditgroupResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "update_editgroup(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            editgroup,
            submit,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn update_editor(
        &self,
        editor_id: String,
        editor: models::Editor,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateEditorResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "update_editor(\"{}\", {:?}) - X-Span-ID: {:?}",
            editor_id,
            editor,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn update_file(
        &self,
        editgroup_id: String,
        ident: String,
        entity: models::FileEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateFileResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "update_file(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn update_fileset(
        &self,
        editgroup_id: String,
        ident: String,
        entity: models::FilesetEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateFilesetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "update_fileset(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn update_release(
        &self,
        editgroup_id: String,
        ident: String,
        entity: models::ReleaseEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateReleaseResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "update_release(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn update_webcapture(
        &self,
        editgroup_id: String,
        ident: String,
        entity: models::WebcaptureEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateWebcaptureResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "update_webcapture(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }

    fn update_work(
        &self,
        editgroup_id: String,
        ident: String,
        entity: models::WorkEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateWorkResponse, Error = ApiError> + Send> {
        let context = context.clone();
        info!(
            "update_work(\"{}\", \"{}\", {:?}) - X-Span-ID: {:?}",
            editgroup_id,
            ident,
            entity,
            context.get().0.clone()
        );
        Box::new(future::err("Generic failure".into()))
    }
}
