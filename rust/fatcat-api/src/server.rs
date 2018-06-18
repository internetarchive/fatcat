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
use futures::Future;
use futures::future;
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
use {AcceptEditgroupResponse, Api, CreateContainerBatchResponse, CreateContainerResponse, CreateCreatorBatchResponse, CreateCreatorResponse, CreateEditgroupResponse, CreateFileBatchResponse,
     CreateFileResponse, CreateReleaseBatchResponse, CreateReleaseResponse, CreateWorkBatchResponse, CreateWorkResponse, GetContainerResponse, GetCreatorReleasesResponse, GetCreatorResponse,
     GetEditgroupResponse, GetEditorChangelogResponse, GetEditorResponse, GetFileResponse, GetReleaseFilesResponse, GetReleaseResponse, GetWorkReleasesResponse, GetWorkResponse,
     LookupContainerResponse, LookupCreatorResponse, LookupFileResponse, LookupReleaseResponse};

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
        "/v0/editgroup/:id/accept",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.accept_editgroup(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        AcceptEditgroupResponse::MergedSuccessfully(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_MERGED_SUCCESSFULLY.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        AcceptEditgroupResponse::Unmergable(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::ACCEPT_EDITGROUP_UNMERGABLE.clone()));

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
                        AcceptEditgroupResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ContainerEntity> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_container(param_entity, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::ContainerEntity>> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_container_batch(param_entity_list.as_ref(), context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::CreatorEntity> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_creator(param_entity, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::CreatorEntity>> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_creator_batch(param_entity_list.as_ref(), context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::Editgroup> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_editgroup(param_entity, context).wait() {
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
                        CreateEditgroupResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::FileEntity> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_file(param_entity, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::FileEntity>> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_file_batch(param_entity_list.as_ref(), context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::ReleaseEntity> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_release(param_entity, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::ReleaseEntity>> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_release_batch(param_entity_list.as_ref(), context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity = if let Some(param_entity_raw) = param_entity {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_raw);

                    let param_entity: Option<models::WorkEntity> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity - doesn't match schema: {}", e))))?;

                    param_entity
                } else {
                    None
                };
                let param_entity = param_entity.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity".to_string())))?;

                match api.create_work(param_entity, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.

                let param_entity_list = req.get::<bodyparser::Raw>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - not valid UTF-8: {}", e))))?;

                let mut unused_elements = Vec::new();

                let param_entity_list = if let Some(param_entity_list_raw) = param_entity_list {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_entity_list_raw);

                    let param_entity_list: Option<Vec<models::WorkEntity>> =
                        serde_ignored::deserialize(deserializer, |path| {
                            warn!("Ignoring unknown field in body: {}", path);
                            unused_elements.push(path.to_string());
                        }).map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse body parameter entity_list - doesn't match schema: {}", e))))?;

                    param_entity_list
                } else {
                    None
                };
                let param_entity_list = param_entity_list.ok_or_else(|| Response::with((status::BadRequest, "Missing required body parameter entity_list".to_string())))?;

                match api.create_work_batch(param_entity_list.as_ref(), context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
    router.get(
        "/v0/container/:id",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_container(param_id, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
        "/v0/creator/:id",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_creator(param_id, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
        "/v0/creator/:id/releases",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_creator_releases(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetCreatorReleasesResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_CREATOR_RELEASES_FOUND_ENTITY.clone()));

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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
        "/v0/editgroup/:id",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_editgroup(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditgroupResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITGROUP_FOUND_ENTITY.clone()));

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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
        "/v0/editor/:username",
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
                let param_username = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("username")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter username".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter username: {}", e))))?
                };

                match api.get_editor(param_username, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditorResponse::FoundEditor(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_FOUND_EDITOR.clone()));

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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
        "/v0/editor/:username/changelog",
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
                let param_username = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("username")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter username".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter username: {}", e))))?
                };

                match api.get_editor_changelog(param_username, context).wait() {
                    Ok(rsp) => match rsp {
                        GetEditorChangelogResponse::FoundMergedChanges(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_CHANGELOG_FOUND_MERGED_CHANGES.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorChangelogResponse::NotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_CHANGELOG_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        GetEditorChangelogResponse::GenericError(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_EDITOR_CHANGELOG_GENERIC_ERROR.clone()));

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
        "GetEditorChangelog",
    );

    let api_clone = api.clone();
    router.get(
        "/v0/file/:id",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_file(param_id, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
        "/v0/release/:id",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_release(param_id, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
        "/v0/release/:id/files",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_release_files(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetReleaseFilesResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_RELEASE_FILES_FOUND_ENTITY.clone()));

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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
        "/v0/work/:id",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_work(param_id, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
        "/v0/work/:id/releases",
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
                let param_id = {
                    let param = req.extensions
                        .get::<Router>()
                        .ok_or_else(|| Response::with((status::InternalServerError, "An internal error occurred".to_string())))?
                        .find("id")
                        .ok_or_else(|| Response::with((status::BadRequest, "Missing path parameter id".to_string())))?;
                    percent_decode(param.as_bytes())
                        .decode_utf8()
                        .map_err(|_| Response::with((status::BadRequest, format!("Couldn't percent-decode path parameter as UTF-8: {}", param))))?
                        .parse()
                        .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse path parameter id: {}", e))))?
                };

                match api.get_work_releases(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        GetWorkReleasesResponse::FoundEntity(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::GET_WORK_RELEASES_FOUND_ENTITY.clone()));

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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
                let param_issnl = query_params
                    .get("issnl")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter issnl".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter issnl was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter issnl - doesn't match schema: {}", e))))?;

                match api.lookup_container(param_issnl, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
                let param_orcid = query_params
                    .get("orcid")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter orcid".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter orcid was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter orcid - doesn't match schema: {}", e))))?;

                match api.lookup_creator(param_orcid, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
                let param_sha1 = query_params
                    .get("sha1")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter sha1".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter sha1 was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter sha1 - doesn't match schema: {}", e))))?;

                match api.lookup_file(param_sha1, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
                let param_doi = query_params
                    .get("doi")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter doi".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter doi was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter doi - doesn't match schema: {}", e))))?;

                match api.lookup_release(param_doi, context).wait() {
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

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
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
}

/// Middleware to extract authentication data from request
pub struct ExtractAuthData;

impl BeforeMiddleware for ExtractAuthData {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        Ok(())
    }
}
