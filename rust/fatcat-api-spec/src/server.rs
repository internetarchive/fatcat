#![allow(unused_extern_crates)]
extern crate bodyparser;
extern crate chrono;
extern crate iron;
extern crate router;
extern crate serde_ignored;
extern crate urlencoded;
extern crate uuid;

use self::iron::prelude::*;
use self::iron::url::percent_encoding::percent_decode;
use self::iron::{modifiers, status, BeforeMiddleware};
use self::router::Router;
use self::urlencoded::UrlEncodedQuery;
use futures::future;
use futures::Future;
use futures::{stream, Stream};
use hyper;
use hyper::header::{ContentType, Headers};
use mimetypes;

use serde_json;

#[allow(unused_imports)]
use std::collections::{BTreeMap, HashMap};
use std::io::Error;
#[allow(unused_imports)]
use swagger;

#[allow(unused_imports)]
use std::collections::BTreeSet;

pub use swagger::auth::Authorization;
use swagger::auth::{AuthData, Scopes};
use swagger::{ApiError, Context, XSpanId};

#[allow(unused_imports)]
use models;
use {
    AcceptEditgroupResponse, Api, AuthCheckResponse, AuthOidcResponse, CreateContainerBatchResponse, CreateContainerResponse, CreateCreatorBatchResponse, CreateCreatorResponse,
    CreateEditgroupAnnotationResponse, CreateEditgroupResponse, CreateFileBatchResponse, CreateFileResponse, CreateFilesetBatchResponse, CreateFilesetResponse, CreateReleaseBatchResponse,
    CreateReleaseResponse, CreateWebcaptureBatchResponse, CreateWebcaptureResponse, CreateWorkBatchResponse, CreateWorkResponse, DeleteContainerEditResponse, DeleteContainerResponse,
    DeleteCreatorEditResponse, DeleteCreatorResponse, DeleteFileEditResponse, DeleteFileResponse, DeleteFilesetEditResponse, DeleteFilesetResponse, DeleteReleaseEditResponse, DeleteReleaseResponse,
    DeleteWebcaptureEditResponse, DeleteWebcaptureResponse, DeleteWorkEditResponse, DeleteWorkResponse, GetChangelogEntryResponse, GetChangelogResponse, GetContainerEditResponse,
    GetContainerHistoryResponse, GetContainerRedirectsResponse, GetContainerResponse, GetContainerRevisionResponse, GetCreatorEditResponse, GetCreatorHistoryResponse, GetCreatorRedirectsResponse,
    GetCreatorReleasesResponse, GetCreatorResponse, GetCreatorRevisionResponse, GetEditgroupAnnotationsResponse, GetEditgroupResponse, GetEditgroupsReviewableResponse, GetEditorAnnotationsResponse,
    GetEditorEditgroupsResponse, GetEditorResponse, GetFileEditResponse, GetFileHistoryResponse, GetFileRedirectsResponse, GetFileResponse, GetFileRevisionResponse, GetFilesetEditResponse,
    GetFilesetHistoryResponse, GetFilesetRedirectsResponse, GetFilesetResponse, GetFilesetRevisionResponse, GetReleaseEditResponse, GetReleaseFilesResponse, GetReleaseFilesetsResponse,
    GetReleaseHistoryResponse, GetReleaseRedirectsResponse, GetReleaseResponse, GetReleaseRevisionResponse, GetReleaseWebcapturesResponse, GetWebcaptureEditResponse, GetWebcaptureHistoryResponse,
    GetWebcaptureRedirectsResponse, GetWebcaptureResponse, GetWebcaptureRevisionResponse, GetWorkEditResponse, GetWorkHistoryResponse, GetWorkRedirectsResponse, GetWorkReleasesResponse,
    GetWorkResponse, GetWorkRevisionResponse, LookupContainerResponse, LookupCreatorResponse, LookupFileResponse, LookupReleaseResponse, UpdateContainerResponse, UpdateCreatorResponse,
    UpdateEditgroupResponse, UpdateEditorResponse, UpdateFileResponse, UpdateFilesetResponse, UpdateReleaseResponse, UpdateWebcaptureResponse, UpdateWorkResponse,
};

header! { (Warning, "Warning") => [String] }

/// Create a new router for `Api`
pub fn router<T>(api: T) -> Router
where
    T: Api + Send + Sync + Clone + 'static,
{
    let mut router = Router::new();
    add_routes(&mut router, api);
    router
}

/// Add routes for `Api` to a provided router.
///
/// Note that these routes are added straight onto the router. This means that if the router
/// already has a route for an endpoint which clashes with those provided by this API, then the
/// old route will be lost.
///
/// It is generally a bad idea to add routes in this way to an existing router, which may have
/// routes on it for other APIs. Distinct APIs should be behind distinct paths to encourage
/// separation of interfaces, which this function does not enforce. APIs should not overlap.
///
/// Alternative approaches include:
///
/// - generate an `iron::middleware::Handler` (usually a `router::Router` or
///   `iron::middleware::chain`) for each interface, and add those handlers inside an existing
///   router, mounted at different paths - so the interfaces are separated by path
/// - use a different instance of `iron::Iron` for each interface - so the interfaces are
///   separated by the address/port they listen on
///
/// This function exists to allow legacy code, which doesn't separate its APIs properly, to make
/// use of this crate.
#[deprecated(note = "APIs should not overlap - only for use in legacy code.")]
pub fn route<T>(router: &mut Router, api: T)
where
    T: Api + Send + Sync + Clone + 'static,
{
    add_routes(router, api)
}

/// Add routes for `Api` to a provided router
fn add_routes<T>(router: &mut Router, api: T)
where
    T: Api + Send + Sync + Clone + 'static,
{
    let api_clone = api.clone();
    router.post(
        "/v0/container",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ContainerEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_container(param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateContainerResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateContainer",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/container/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params
                    .get("autoaccept")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.to_lowercase().parse::<bool>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;
                let param_editgroup_id = query_params.get("editgroup_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::ContainerEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_container_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateContainerBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerBatchResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerBatchResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateContainerBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CONTAINER_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateContainerBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/container/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                match api.delete_container(param_ident, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteContainerResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteContainer",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/container/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.delete_container_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteContainerEditResponse::DeletedEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_EDIT_DELETED_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerEditResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_EDIT_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerEditResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_EDIT_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteContainerEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CONTAINER_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteContainerEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/container/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_container(param_ident, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetContainerResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetContainer",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/container/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.get_container_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetContainerEditResponse::FoundEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_EDIT_FOUND_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetContainerEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/container/:ident/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;

                match api.get_container_history(param_ident, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetContainerHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetContainerHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/container/:ident/redirects",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                match api.get_container_redirects(param_ident, context).wait() {
                    Ok(rsp) => match rsp {
                        GetContainerRedirectsResponse::FoundEntityRedirects(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_REDIRECTS_FOUND_ENTITY_REDIRECTS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerRedirectsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_REDIRECTS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerRedirectsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_REDIRECTS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerRedirectsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_REDIRECTS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetContainerRedirects",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/container/rev/:rev_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_rev_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("rev_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter rev_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter rev_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_container_revision(param_rev_id, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetContainerRevisionResponse::FoundEntityRevision(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_REVISION_FOUND_ENTITY_REVISION.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerRevisionResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_REVISION_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerRevisionResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_REVISION_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetContainerRevisionResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CONTAINER_REVISION_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetContainerRevision",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/container/lookup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_issnl = query_params.get("issnl").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_wikidata_qid = query_params.get("wikidata_qid").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.lookup_container(param_issnl, param_wikidata_qid, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        LookupContainerResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CONTAINER_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "LookupContainer",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/container/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ContainerEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_container(param_ident, param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateContainerResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateContainerResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateContainerResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateContainerResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateContainerResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateContainerResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CONTAINER_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateContainer",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/creator",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::CreatorEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_creator(param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateCreatorResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateCreator",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/creator/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params
                    .get("autoaccept")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.to_lowercase().parse::<bool>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;
                let param_editgroup_id = query_params.get("editgroup_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::CreatorEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_creator_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateCreatorBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorBatchResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorBatchResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateCreatorBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_CREATOR_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateCreatorBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/creator/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                match api.delete_creator(param_ident, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteCreatorResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteCreator",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/creator/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.delete_creator_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteCreatorEditResponse::DeletedEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_EDIT_DELETED_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorEditResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_EDIT_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorEditResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_EDIT_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteCreatorEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_CREATOR_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteCreatorEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_creator(param_ident, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetCreator",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.get_creator_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorEditResponse::FoundEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_EDIT_FOUND_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetCreatorEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/:ident/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;

                match api.get_creator_history(param_ident, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetCreatorHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/:ident/redirects",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                match api.get_creator_redirects(param_ident, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorRedirectsResponse::FoundEntityRedirects(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_REDIRECTS_FOUND_ENTITY_REDIRECTS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorRedirectsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_REDIRECTS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorRedirectsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_REDIRECTS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorRedirectsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_REDIRECTS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetCreatorRedirects",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/:ident/releases",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_creator_releases(param_ident, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorReleasesResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_RELEASES_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorReleasesResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_RELEASES_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorReleasesResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_RELEASES_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorReleasesResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_RELEASES_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetCreatorReleases",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/rev/:rev_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_rev_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("rev_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter rev_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter rev_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_creator_revision(param_rev_id, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorRevisionResponse::FoundEntityRevision(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_REVISION_FOUND_ENTITY_REVISION.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorRevisionResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_REVISION_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorRevisionResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_REVISION_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetCreatorRevisionResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_REVISION_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetCreatorRevision",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/creator/lookup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_orcid = query_params.get("orcid").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_wikidata_qid = query_params.get("wikidata_qid").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.lookup_creator(param_orcid, param_wikidata_qid, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        LookupCreatorResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CREATOR_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "LookupCreator",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/creator/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::CreatorEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_creator(param_ident, param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateCreatorResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateCreatorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateCreatorResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateCreatorResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateCreatorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateCreatorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_CREATOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateCreator",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/auth/check",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_role = query_params.get("role").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.auth_check(param_role, context).wait() {
                    Ok(rsp) => match rsp {
                        AuthCheckResponse::Success(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_CHECK_SUCCESS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AuthCheckResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_CHECK_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AuthCheckResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::AUTH_CHECK_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AuthCheckResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_CHECK_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AuthCheckResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_CHECK_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "AuthCheck",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/auth/oidc",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_oidc_params = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter oidc_params - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_oidc_params = if let Some(param_oidc_params_raw) = param_oidc_params {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_oidc_params_raw);

                    let param_oidc_params: Option<models::AuthOidc> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter oidc_params - doesn't match schema: {}", e))))?;

                    param_oidc_params
                } else {
                    None
                };
                let param_oidc_params = param_oidc_params.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter oidc_params".to_string())))?;

                match api.auth_oidc(param_oidc_params, context).wait() {
                    Ok(rsp) => match rsp {
                        AuthOidcResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_OIDC_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        AuthOidcResponse::Created(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_OIDC_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        AuthOidcResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_OIDC_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        AuthOidcResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::AUTH_OIDC_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        AuthOidcResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_OIDC_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        AuthOidcResponse::Conflict(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(409), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_OIDC_CONFLICT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        AuthOidcResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::AUTH_OIDC_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "AuthOidc",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/editgroup/reviewable",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;
                let param_before = query_params
                    .get("before")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<chrono::DateTime<chrono::Utc>>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected UTC datetime in ISO/RFC format)".to_string())))?;
                let param_since = query_params
                    .get("since")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<chrono::DateTime<chrono::Utc>>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected UTC datetime in ISO/RFC format)".to_string())))?;

                match api.get_editgroups_reviewable(param_expand, param_limit, param_before, param_since, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditgroupsReviewableResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUPS_REVIEWABLE_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupsReviewableResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUPS_REVIEWABLE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupsReviewableResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUPS_REVIEWABLE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupsReviewableResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUPS_REVIEWABLE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetEditgroupsReviewable",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/editor/:editor_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_editor_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("editor_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter editor_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter editor_id: {}", e))))?
                };

                match api.get_editor(param_editor_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditorResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetEditor",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/editor/:editor_id/editgroups",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_editor_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("editor_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter editor_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter editor_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;
                let param_before = query_params
                    .get("before")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<chrono::DateTime<chrono::Utc>>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected UTC datetime in ISO/RFC format)".to_string())))?;
                let param_since = query_params
                    .get("since")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<chrono::DateTime<chrono::Utc>>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected UTC datetime in ISO/RFC format)".to_string())))?;

                match api.get_editor_editgroups(param_editor_id, param_limit, param_before, param_since, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditorEditgroupsResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_EDITGROUPS_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorEditgroupsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_EDITGROUPS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorEditgroupsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_EDITGROUPS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorEditgroupsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_EDITGROUPS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetEditorEditgroups",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/editgroup/:editgroup_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_editgroup_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("editgroup_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter editgroup_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter editgroup_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_submit = query_params
                    .get("submit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.to_lowercase().parse::<bool>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_editgroup = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter editgroup - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_editgroup = if let Some(param_editgroup_raw) = param_editgroup {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_editgroup_raw);

                    let param_editgroup: Option<models::Editgroup> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter editgroup - doesn't match schema: {}", e))))?;

                    param_editgroup
                } else {
                    None
                };
                let param_editgroup = param_editgroup.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter editgroup".to_string())))?;

                match api.update_editgroup(param_editgroup_id, param_editgroup, param_submit, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateEditgroupResponse::UpdatedEditgroup(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITGROUP_UPDATED_EDITGROUP.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditgroupResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITGROUP_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditgroupResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITGROUP_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditgroupResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITGROUP_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditgroupResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITGROUP_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditgroupResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITGROUP_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateEditgroup",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/editor/:editor_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_editor_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("editor_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter editor_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter editor_id: {}", e))))?
                };

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_editor = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter editor - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_editor = if let Some(param_editor_raw) = param_editor {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_editor_raw);

                    let param_editor: Option<models::Editor> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter editor - doesn't match schema: {}", e))))?;

                    param_editor
                } else {
                    None
                };
                let param_editor = param_editor.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter editor".to_string())))?;

                match api.update_editor(param_editor_id, param_editor, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateEditorResponse::UpdatedEditor(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITOR_UPDATED_EDITOR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditorResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITOR_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditorResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITOR_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditorResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITOR_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditorResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITOR_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateEditorResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_EDITOR_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateEditor",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/editgroup/:editgroup_id/accept",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_editgroup_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("editgroup_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter editgroup_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter editgroup_id: {}", e))))?
                };

                match api.accept_editgroup(param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        AcceptEditgroupResponse::MergedSuccessfully(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_MERGED_SUCCESSFULLY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::EditConflict(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(409), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_EDIT_CONFLICT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "AcceptEditgroup",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/editgroup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_editgroup = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter editgroup - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_editgroup = if let Some(param_editgroup_raw) = param_editgroup {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_editgroup_raw);

                    let param_editgroup: Option<models::Editgroup> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter editgroup - doesn't match schema: {}", e))))?;

                    param_editgroup
                } else {
                    None
                };
                let param_editgroup = param_editgroup.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter editgroup".to_string())))?;

                match api.create_editgroup(param_editgroup, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateEditgroupResponse::SuccessfullyCreated(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_SUCCESSFULLY_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateEditgroup",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/editgroup/:editgroup_id/annotation",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_editgroup_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("editgroup_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter editgroup_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter editgroup_id: {}", e))))?
                };

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_annotation = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter annotation - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_annotation = if let Some(param_annotation_raw) = param_annotation {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_annotation_raw);

                    let param_annotation: Option<models::EditgroupAnnotation> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter annotation - doesn't match schema: {}", e))))?;

                    param_annotation
                } else {
                    None
                };
                let param_annotation = param_annotation.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter annotation".to_string())))?;

                match api.create_editgroup_annotation(param_editgroup_id, param_annotation, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateEditgroupAnnotationResponse::Created(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_ANNOTATION_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupAnnotationResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_ANNOTATION_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupAnnotationResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_ANNOTATION_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupAnnotationResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_ANNOTATION_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupAnnotationResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_ANNOTATION_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateEditgroupAnnotationResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_EDITGROUP_ANNOTATION_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateEditgroupAnnotation",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/changelog",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;

                match api.get_changelog(param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetChangelogResponse::Success(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_SUCCESS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetChangelogResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetChangelogResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetChangelog",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/changelog/:index",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_index = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("index")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter index".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter index: {}", e))))?
                };

                match api.get_changelog_entry(param_index, context).wait() {
                    Ok(rsp) => match rsp {
                        GetChangelogEntryResponse::FoundChangelogEntry(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_ENTRY_FOUND_CHANGELOG_ENTRY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetChangelogEntryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_ENTRY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetChangelogEntryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_ENTRY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetChangelogEntryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CHANGELOG_ENTRY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetChangelogEntry",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/editgroup/:editgroup_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_editgroup_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("editgroup_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter editgroup_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter editgroup_id: {}", e))))?
                };

                match api.get_editgroup(param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditgroupResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetEditgroup",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/editgroup/:editgroup_id/annotations",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_editgroup_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("editgroup_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter editgroup_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter editgroup_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_editgroup_annotations(param_editgroup_id, param_expand, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditgroupAnnotationsResponse::Success(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_ANNOTATIONS_SUCCESS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupAnnotationsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_ANNOTATIONS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupAnnotationsResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_ANNOTATIONS_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupAnnotationsResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_ANNOTATIONS_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupAnnotationsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_ANNOTATIONS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditgroupAnnotationsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_ANNOTATIONS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetEditgroupAnnotations",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/editor/:editor_id/annotations",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_editor_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("editor_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter editor_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter editor_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;
                let param_before = query_params
                    .get("before")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<chrono::DateTime<chrono::Utc>>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected UTC datetime in ISO/RFC format)".to_string())))?;
                let param_since = query_params
                    .get("since")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<chrono::DateTime<chrono::Utc>>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected UTC datetime in ISO/RFC format)".to_string())))?;

                match api.get_editor_annotations(param_editor_id, param_limit, param_before, param_since, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditorAnnotationsResponse::Success(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_ANNOTATIONS_SUCCESS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorAnnotationsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_ANNOTATIONS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorAnnotationsResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_ANNOTATIONS_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorAnnotationsResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_ANNOTATIONS_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorAnnotationsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_ANNOTATIONS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorAnnotationsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_ANNOTATIONS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetEditorAnnotations",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/file",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::FileEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_file(param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateFileResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateFile",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/file/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params
                    .get("autoaccept")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.to_lowercase().parse::<bool>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;
                let param_editgroup_id = query_params.get("editgroup_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::FileEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_file_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateFileBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileBatchResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileBatchResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFileBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILE_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateFileBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/file/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                match api.delete_file(param_ident, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteFileResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteFile",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/file/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.delete_file_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteFileEditResponse::DeletedEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_EDIT_DELETED_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileEditResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_EDIT_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileEditResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_EDIT_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFileEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILE_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteFileEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_file(param_ident, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFileResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFile",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.get_file_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFileEditResponse::FoundEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_EDIT_FOUND_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFileEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/:ident/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;

                match api.get_file_history(param_ident, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFileHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFileHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/:ident/redirects",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                match api.get_file_redirects(param_ident, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFileRedirectsResponse::FoundEntityRedirects(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_REDIRECTS_FOUND_ENTITY_REDIRECTS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileRedirectsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_REDIRECTS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileRedirectsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_REDIRECTS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileRedirectsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_REDIRECTS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFileRedirects",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/rev/:rev_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_rev_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("rev_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter rev_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter rev_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_file_revision(param_rev_id, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFileRevisionResponse::FoundEntityRevision(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_REVISION_FOUND_ENTITY_REVISION.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileRevisionResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_REVISION_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileRevisionResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_REVISION_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFileRevisionResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILE_REVISION_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFileRevision",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/lookup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_md5 = query_params.get("md5").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_sha1 = query_params.get("sha1").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_sha256 = query_params.get("sha256").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.lookup_file(param_md5, param_sha1, param_sha256, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        LookupFileResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_FILE_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "LookupFile",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/file/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::FileEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_file(param_ident, param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateFileResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFileResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFileResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFileResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFileResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFileResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateFile",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/fileset",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::FilesetEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_fileset(param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateFilesetResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateFileset",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/fileset/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params
                    .get("autoaccept")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.to_lowercase().parse::<bool>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;
                let param_editgroup_id = query_params.get("editgroup_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::FilesetEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_fileset_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateFilesetBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetBatchResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_BATCH_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetBatchResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_BATCH_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateFilesetBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_FILESET_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateFilesetBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/fileset/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                match api.delete_fileset(param_ident, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteFilesetResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteFileset",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/fileset/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.delete_fileset_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteFilesetEditResponse::DeletedEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_EDIT_DELETED_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetEditResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_EDIT_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetEditResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_EDIT_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteFilesetEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_FILESET_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteFilesetEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/fileset/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_fileset(param_ident, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFilesetResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFileset",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/fileset/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.get_fileset_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFilesetEditResponse::FoundEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_EDIT_FOUND_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFilesetEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/fileset/:ident/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;

                match api.get_fileset_history(param_ident, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFilesetHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFilesetHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/fileset/:ident/redirects",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                match api.get_fileset_redirects(param_ident, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFilesetRedirectsResponse::FoundEntityRedirects(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_REDIRECTS_FOUND_ENTITY_REDIRECTS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetRedirectsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_REDIRECTS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetRedirectsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_REDIRECTS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetRedirectsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_REDIRECTS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFilesetRedirects",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/fileset/rev/:rev_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_rev_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("rev_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter rev_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter rev_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_fileset_revision(param_rev_id, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetFilesetRevisionResponse::FoundEntityRevision(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_REVISION_FOUND_ENTITY_REVISION.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetRevisionResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_REVISION_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetRevisionResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_REVISION_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetFilesetRevisionResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_FILESET_REVISION_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetFilesetRevision",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/fileset/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::FilesetEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_fileset(param_ident, param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateFilesetResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILESET_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFilesetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILESET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFilesetResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILESET_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFilesetResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILESET_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFilesetResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILESET_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateFilesetResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_FILESET_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateFileset",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/release",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ReleaseEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_release(param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateReleaseResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateRelease",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/release/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params
                    .get("autoaccept")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.to_lowercase().parse::<bool>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;
                let param_editgroup_id = query_params.get("editgroup_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::ReleaseEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_release_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateReleaseBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseBatchResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseBatchResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateReleaseBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_RELEASE_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateReleaseBatch",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/work",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::WorkEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_work(param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateWorkResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateWork",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/release/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                match api.delete_release(param_ident, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteReleaseResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteRelease",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/release/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.delete_release_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteReleaseEditResponse::DeletedEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_EDIT_DELETED_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseEditResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_EDIT_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseEditResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_EDIT_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteReleaseEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_RELEASE_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteReleaseEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_release(param_ident, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetRelease",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.get_release_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseEditResponse::FoundEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_EDIT_FOUND_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetReleaseEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/:ident/files",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_release_files(param_ident, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseFilesResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILES_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseFilesResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILES_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseFilesResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILES_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseFilesResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILES_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetReleaseFiles",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/:ident/filesets",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_release_filesets(param_ident, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseFilesetsResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILESETS_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseFilesetsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILESETS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseFilesetsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILESETS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseFilesetsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILESETS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetReleaseFilesets",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/:ident/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;

                match api.get_release_history(param_ident, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetReleaseHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/:ident/redirects",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                match api.get_release_redirects(param_ident, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseRedirectsResponse::FoundEntityRedirects(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_REDIRECTS_FOUND_ENTITY_REDIRECTS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseRedirectsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_REDIRECTS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseRedirectsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_REDIRECTS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseRedirectsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_REDIRECTS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetReleaseRedirects",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/rev/:rev_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_rev_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("rev_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter rev_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter rev_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_release_revision(param_rev_id, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseRevisionResponse::FoundEntityRevision(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_REVISION_FOUND_ENTITY_REVISION.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseRevisionResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_REVISION_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseRevisionResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_REVISION_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseRevisionResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_REVISION_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetReleaseRevision",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/:ident/webcaptures",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_release_webcaptures(param_ident, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseWebcapturesResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_WEBCAPTURES_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseWebcapturesResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_WEBCAPTURES_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseWebcapturesResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_WEBCAPTURES_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetReleaseWebcapturesResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_WEBCAPTURES_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetReleaseWebcaptures",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/release/lookup",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_doi = query_params.get("doi").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_wikidata_qid = query_params.get("wikidata_qid").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_isbn13 = query_params.get("isbn13").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_pmid = query_params.get("pmid").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_pmcid = query_params.get("pmcid").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_core_id = query_params.get("core_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_arxiv_id = query_params.get("arxiv_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_jstor_id = query_params.get("jstor_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api
                    .lookup_release(
                        param_doi,
                        param_wikidata_qid,
                        param_isbn13,
                        param_pmid,
                        param_pmcid,
                        param_core_id,
                        param_arxiv_id,
                        param_jstor_id,
                        param_expand,
                        param_hide,
                        context,
                    )
                    .wait()
                {
                    Ok(rsp) => match rsp {
                        LookupReleaseResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_RELEASE_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        LookupReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::LOOKUP_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "LookupRelease",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/release/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ReleaseEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_release(param_ident, param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateReleaseResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateReleaseResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateReleaseResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateReleaseResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateReleaseResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateReleaseResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_RELEASE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateRelease",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/webcapture",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::WebcaptureEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_webcapture(param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateWebcaptureResponse::CreatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_CREATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateWebcapture",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/webcapture/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params
                    .get("autoaccept")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.to_lowercase().parse::<bool>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;
                let param_editgroup_id = query_params.get("editgroup_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::WebcaptureEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_webcapture_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateWebcaptureBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureBatchResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_BATCH_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureBatchResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_BATCH_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWebcaptureBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WEBCAPTURE_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateWebcaptureBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/webcapture/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                match api.delete_webcapture(param_ident, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteWebcaptureResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteWebcapture",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/webcapture/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.delete_webcapture_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteWebcaptureEditResponse::DeletedEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_EDIT_DELETED_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureEditResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_EDIT_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureEditResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_EDIT_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWebcaptureEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WEBCAPTURE_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteWebcaptureEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/webcapture/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_webcapture(param_ident, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWebcaptureResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWebcapture",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/webcapture/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.get_webcapture_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWebcaptureEditResponse::FoundEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_EDIT_FOUND_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWebcaptureEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/webcapture/:ident/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;

                match api.get_webcapture_history(param_ident, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWebcaptureHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWebcaptureHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/webcapture/:ident/redirects",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                match api.get_webcapture_redirects(param_ident, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWebcaptureRedirectsResponse::FoundEntityRedirects(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_REDIRECTS_FOUND_ENTITY_REDIRECTS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureRedirectsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_REDIRECTS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureRedirectsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_REDIRECTS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureRedirectsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_REDIRECTS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWebcaptureRedirects",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/webcapture/rev/:rev_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_rev_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("rev_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter rev_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter rev_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_webcapture_revision(param_rev_id, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWebcaptureRevisionResponse::FoundEntityRevision(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_REVISION_FOUND_ENTITY_REVISION.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureRevisionResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_REVISION_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureRevisionResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_REVISION_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWebcaptureRevisionResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WEBCAPTURE_REVISION_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWebcaptureRevision",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/webcapture/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::WebcaptureEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_webcapture(param_ident, param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateWebcaptureResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WEBCAPTURE_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWebcaptureResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WEBCAPTURE_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWebcaptureResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WEBCAPTURE_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWebcaptureResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WEBCAPTURE_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWebcaptureResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WEBCAPTURE_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWebcaptureResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WEBCAPTURE_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateWebcapture",
    );

    let api_clone = api.clone();
    router.post(
        "/v0/work/batch",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_autoaccept = query_params
                    .get("autoaccept")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.to_lowercase().parse::<bool>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;
                let param_editgroup_id = query_params.get("editgroup_id").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::WorkEntity>> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_work_batch(param_entity_list.as_ref(), param_autoaccept, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreateWorkBatchResponse::CreatedEntities(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_CREATED_ENTITIES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkBatchResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkBatchResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkBatchResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkBatchResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreateWorkBatchResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATE_WORK_BATCH_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "CreateWorkBatch",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/work/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                match api.delete_work(param_ident, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteWorkResponse::DeletedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_DELETED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteWork",
    );

    let api_clone = api.clone();
    router.delete(
        "/v0/work/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.delete_work_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        DeleteWorkEditResponse::DeletedEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_EDIT_DELETED_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkEditResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_EDIT_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkEditResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_EDIT_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        DeleteWorkEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::DELETE_WORK_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "DeleteWorkEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/work/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_work(param_ident, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_FOUND_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWork",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/work/edit/:edit_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_edit_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("edit_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter edit_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter edit_id: {}", e))))?
                };

                match api.get_work_edit(param_edit_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkEditResponse::FoundEdit(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_EDIT_FOUND_EDIT.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkEditResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_EDIT_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkEditResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_EDIT_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkEditResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_EDIT_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWorkEdit",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/work/:ident/history",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_limit = query_params
                    .get("limit")
                    .and_then(|list| list.first())
                    .and_then(|x| Some(x.parse::<i64>()))
                    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
                    .map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;

                match api.get_work_history(param_ident, param_limit, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkHistoryResponse::FoundEntityHistory(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_HISTORY_FOUND_ENTITY_HISTORY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkHistoryResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_HISTORY_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkHistoryResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_HISTORY_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkHistoryResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_HISTORY_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWorkHistory",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/work/:ident/redirects",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                match api.get_work_redirects(param_ident, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkRedirectsResponse::FoundEntityRedirects(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_REDIRECTS_FOUND_ENTITY_REDIRECTS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkRedirectsResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_REDIRECTS_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkRedirectsResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_REDIRECTS_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkRedirectsResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_REDIRECTS_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWorkRedirects",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/work/:ident/releases",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_work_releases(param_ident, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkReleasesResponse::Found(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_RELEASES_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkReleasesResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_RELEASES_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkReleasesResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_RELEASES_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkReleasesResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_RELEASES_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWorkReleases",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/work/rev/:rev_id",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                // Path parameters
                let param_rev_id = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("rev_id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter rev_id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter rev_id: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_expand = query_params.get("expand").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());
                let param_hide = query_params.get("hide").and_then(|list| list.first()).and_then(|x| x.parse::<String>().ok());

                match api.get_work_revision(param_rev_id, param_expand, param_hide, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkRevisionResponse::FoundEntityRevision(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_REVISION_FOUND_ENTITY_REVISION.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkRevisionResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_REVISION_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkRevisionResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_REVISION_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetWorkRevisionResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_REVISION_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "GetWorkRevision",
    );

    let api_clone = api.clone();
    router.put(
        "/v0/work/:ident",
        move |req: &mut Request| {
            let mut context = Context::default();

            // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
            fn handle_request<T>(req: &mut Request, api: &T, context: &mut Context) -> Result<Response, Response>
            where
                T: Api,
            {
                context.x_span_id = Some(req.headers.get::<XSpanId>().map(XSpanId::to_string).unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()));
                context.auth_data = req.extensions.remove::<AuthData>();
                context.authorization = req.extensions.remove::<Authorization>();

                let authorization = context.authorization.as_ref().ok_or_else(|| Response::with((status::Forbidden, "Unauthenticated".to_string())))?;

                // Path parameters
                let param_ident = {
                    let param = req
                        .extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("ident")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter ident".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter ident: {}", e))))?
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = req.get::<UrlEncodedQuery>().unwrap_or_default();
                let param_editgroup_id = query_params
                    .get("editgroup_id")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter editgroup_id".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter editgroup_id was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter editgroup_id - doesn't match schema: {}", e))))?;

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req
                    .get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::WorkEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    })
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.update_work(param_ident, param_entity, param_editgroup_id, context).wait() {
                    Ok(rsp) => match rsp {
                        UpdateWorkResponse::UpdatedEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_UPDATED_ENTITY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWorkResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWorkResponse::NotAuthorized { body, www_authenticate } => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(401), body_string));
                            header! { (ResponseWwwAuthenticate, "WWW_Authenticate") => [String] }
                            response.headers.set(ResponseWwwAuthenticate(www_authenticate));

                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_NOT_AUTHORIZED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWorkResponse::Forbidden(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(403), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_FORBIDDEN.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWorkResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        UpdateWorkResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(500), body_string));
                            response.headers.set(ContentType(mimetypes::responses::UPDATE_WORK_GENERIC_ERROR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                    },
                    Err(_) => {
                        // Application code returned an error. This should not happen, as the implementation should
                        // return a valid response.
                        Err(Response::with((status::InternalServerError, "An internal error occurred".to_string())))
                    }
                }
            }

            handle_request(req, &api_clone, &mut context).or_else(|mut response| {
                context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                Ok(response)
            })
        },
        "UpdateWork",
    );
}

/// Middleware to extract authentication data from request
pub struct ExtractAuthData;

impl BeforeMiddleware for ExtractAuthData {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        {
            header! { (ApiKey1, "Authorization") => [String] }
            if let Some(header) = req.headers.get::<ApiKey1>() {
                req.extensions.insert::<AuthData>(AuthData::ApiKey(header.0.clone()));
                return Ok(());
            }
        }

        Ok(())
    }
}
