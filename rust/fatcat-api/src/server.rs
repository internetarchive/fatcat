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
use {Api, ContainerIdGetResponse, ContainerLookupGetResponse, ContainerPostResponse, CreatorIdGetResponse, CreatorLookupGetResponse, CreatorPostResponse, EditgroupIdAcceptPostResponse,
     EditgroupIdGetResponse, EditgroupPostResponse, EditorUsernameChangelogGetResponse, EditorUsernameGetResponse, FileIdGetResponse, FileLookupGetResponse, FilePostResponse, ReleaseIdGetResponse,
     ReleaseLookupGetResponse, ReleasePostResponse, WorkIdGetResponse, WorkPostResponse};

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

                match api.container_id_get(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        ContainerIdGetResponse::FetchASingleContainerById(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CONTAINER_ID_GET_FETCH_A_SINGLE_CONTAINER_BY_ID.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ContainerIdGetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CONTAINER_ID_GET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ContainerIdGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CONTAINER_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "ContainerIdGet",
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
                let param_issn = query_params
                    .get("issn")
                    .ok_or_else(|| Response::with((status::BadRequest, "Missing required query parameter issn".to_string())))?
                    .first()
                    .ok_or_else(|| Response::with((status::BadRequest, "Required query parameter issn was empty".to_string())))?
                    .parse::<String>()
                    .map_err(|e| Response::with((status::BadRequest, format!("Couldn't parse query parameter issn - doesn't match schema: {}", e))))?;

                match api.container_lookup_get(param_issn, context).wait() {
                    Ok(rsp) => match rsp {
                        ContainerLookupGetResponse::FindASingleContainerByExternalIdentifer(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response
                                .headers
                                .set(ContentType(mimetypes::responses::CONTAINER_LOOKUP_GET_FIND_A_SINGLE_CONTAINER_BY_EXTERNAL_IDENTIFER.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ContainerLookupGetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CONTAINER_LOOKUP_GET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ContainerLookupGetResponse::NoSuchContainer(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CONTAINER_LOOKUP_GET_NO_SUCH_CONTAINER.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ContainerLookupGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CONTAINER_LOOKUP_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "ContainerLookupGet",
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

                let param_body = req.get::<bodyparser::Raw>().unwrap_or(None);

                let mut unused_elements = Vec::new();

                let param_body = if let Some(param_body_raw) = param_body {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_body_raw);

                    let param_body: Option<models::ContainerEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).unwrap_or(None);

                    param_body
                } else {
                    None
                };

                match api.container_post(param_body, context).wait() {
                    Ok(rsp) => match rsp {
                        ContainerPostResponse::Created(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CONTAINER_POST_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        ContainerPostResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CONTAINER_POST_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        ContainerPostResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CONTAINER_POST_GENERIC_ERROR_RESPONSE.clone()));

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
        "ContainerPost",
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

                match api.creator_id_get(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        CreatorIdGetResponse::FetchASingleCreatorById(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATOR_ID_GET_FETCH_A_SINGLE_CREATOR_BY_ID.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        CreatorIdGetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATOR_ID_GET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        CreatorIdGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATOR_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "CreatorIdGet",
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

                match api.creator_lookup_get(param_orcid, context).wait() {
                    Ok(rsp) => match rsp {
                        CreatorLookupGetResponse::FindASingleCreatorByExternalIdentifer(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response
                                .headers
                                .set(ContentType(mimetypes::responses::CREATOR_LOOKUP_GET_FIND_A_SINGLE_CREATOR_BY_EXTERNAL_IDENTIFER.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        CreatorLookupGetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATOR_LOOKUP_GET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        CreatorLookupGetResponse::NoSuchCreator(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATOR_LOOKUP_GET_NO_SUCH_CREATOR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        CreatorLookupGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATOR_LOOKUP_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "CreatorLookupGet",
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

                let param_body = req.get::<bodyparser::Raw>().unwrap_or(None);

                let mut unused_elements = Vec::new();

                let param_body = if let Some(param_body_raw) = param_body {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_body_raw);

                    let param_body: Option<models::CreatorEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).unwrap_or(None);

                    param_body
                } else {
                    None
                };

                match api.creator_post(param_body, context).wait() {
                    Ok(rsp) => match rsp {
                        CreatorPostResponse::Created(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATOR_POST_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreatorPostResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATOR_POST_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        CreatorPostResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::CREATOR_POST_GENERIC_ERROR_RESPONSE.clone()));

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
        "CreatorPost",
    );

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

                match api.editgroup_id_accept_post(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        EditgroupIdAcceptPostResponse::MergedEditgroupSuccessfully_(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITGROUP_ID_ACCEPT_POST_MERGED_EDITGROUP_SUCCESSFULLY_.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditgroupIdAcceptPostResponse::EditgroupIsInAnUnmergableState(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response
                                .headers
                                .set(ContentType(mimetypes::responses::EDITGROUP_ID_ACCEPT_POST_EDITGROUP_IS_IN_AN_UNMERGABLE_STATE.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditgroupIdAcceptPostResponse::NoSuchEditgroup(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITGROUP_ID_ACCEPT_POST_NO_SUCH_EDITGROUP.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditgroupIdAcceptPostResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITGROUP_ID_ACCEPT_POST_GENERIC_ERROR_RESPONSE.clone()));

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
        "EditgroupIdAcceptPost",
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

                match api.editgroup_id_get(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        EditgroupIdGetResponse::FetchEditgroupByIdentifier(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITGROUP_ID_GET_FETCH_EDITGROUP_BY_IDENTIFIER.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditgroupIdGetResponse::NoSuchEditgroup(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITGROUP_ID_GET_NO_SUCH_EDITGROUP.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditgroupIdGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITGROUP_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "EditgroupIdGet",
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

                match api.editgroup_post(context).wait() {
                    Ok(rsp) => match rsp {
                        EditgroupPostResponse::SuccessfullyCreated(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITGROUP_POST_SUCCESSFULLY_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditgroupPostResponse::InvalidRequestParameters(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITGROUP_POST_INVALID_REQUEST_PARAMETERS.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditgroupPostResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITGROUP_POST_GENERIC_ERROR_RESPONSE.clone()));

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
        "EditgroupPost",
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

                match api.editor_username_changelog_get(param_username, context).wait() {
                    Ok(rsp) => match rsp {
                        EditorUsernameChangelogGetResponse::FindChanges_(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITOR_USERNAME_CHANGELOG_GET_FIND_CHANGES_.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditorUsernameChangelogGetResponse::UsernameNotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITOR_USERNAME_CHANGELOG_GET_USERNAME_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditorUsernameChangelogGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITOR_USERNAME_CHANGELOG_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "EditorUsernameChangelogGet",
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

                match api.editor_username_get(param_username, context).wait() {
                    Ok(rsp) => match rsp {
                        EditorUsernameGetResponse::FetchGenericInformationAboutAnEditor(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response
                                .headers
                                .set(ContentType(mimetypes::responses::EDITOR_USERNAME_GET_FETCH_GENERIC_INFORMATION_ABOUT_AN_EDITOR.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditorUsernameGetResponse::UsernameNotFound(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITOR_USERNAME_GET_USERNAME_NOT_FOUND.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        EditorUsernameGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::EDITOR_USERNAME_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "EditorUsernameGet",
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

                match api.file_id_get(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        FileIdGetResponse::FetchASingleFileById(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::FILE_ID_GET_FETCH_A_SINGLE_FILE_BY_ID.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        FileIdGetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::FILE_ID_GET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        FileIdGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::FILE_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "FileIdGet",
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

                match api.file_lookup_get(param_sha1, context).wait() {
                    Ok(rsp) => match rsp {
                        FileLookupGetResponse::FindASingleFileByExternalIdentifer(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response
                                .headers
                                .set(ContentType(mimetypes::responses::FILE_LOOKUP_GET_FIND_A_SINGLE_FILE_BY_EXTERNAL_IDENTIFER.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        FileLookupGetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::FILE_LOOKUP_GET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        FileLookupGetResponse::NoSuchFile(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::FILE_LOOKUP_GET_NO_SUCH_FILE.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        FileLookupGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::FILE_LOOKUP_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "FileLookupGet",
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

                let param_body = req.get::<bodyparser::Raw>().unwrap_or(None);

                let mut unused_elements = Vec::new();

                let param_body = if let Some(param_body_raw) = param_body {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_body_raw);

                    let param_body: Option<models::FileEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).unwrap_or(None);

                    param_body
                } else {
                    None
                };

                match api.file_post(param_body, context).wait() {
                    Ok(rsp) => match rsp {
                        FilePostResponse::Created(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::FILE_POST_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        FilePostResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::FILE_POST_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        FilePostResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::FILE_POST_GENERIC_ERROR_RESPONSE.clone()));

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
        "FilePost",
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

                match api.release_id_get(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        ReleaseIdGetResponse::FetchASingleReleaseById(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::RELEASE_ID_GET_FETCH_A_SINGLE_RELEASE_BY_ID.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ReleaseIdGetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::RELEASE_ID_GET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ReleaseIdGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::RELEASE_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "ReleaseIdGet",
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

                match api.release_lookup_get(param_doi, context).wait() {
                    Ok(rsp) => match rsp {
                        ReleaseLookupGetResponse::FindASingleReleaseByExternalIdentifer(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response
                                .headers
                                .set(ContentType(mimetypes::responses::RELEASE_LOOKUP_GET_FIND_A_SINGLE_RELEASE_BY_EXTERNAL_IDENTIFER.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ReleaseLookupGetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::RELEASE_LOOKUP_GET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ReleaseLookupGetResponse::NoSuchRelease(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(404), body_string));
                            response.headers.set(ContentType(mimetypes::responses::RELEASE_LOOKUP_GET_NO_SUCH_RELEASE.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        ReleaseLookupGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::RELEASE_LOOKUP_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "ReleaseLookupGet",
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

                let param_body = req.get::<bodyparser::Raw>().unwrap_or(None);

                let mut unused_elements = Vec::new();

                let param_body = if let Some(param_body_raw) = param_body {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_body_raw);

                    let param_body: Option<models::ReleaseEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).unwrap_or(None);

                    param_body
                } else {
                    None
                };

                match api.release_post(param_body, context).wait() {
                    Ok(rsp) => match rsp {
                        ReleasePostResponse::Created(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::RELEASE_POST_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        ReleasePostResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::RELEASE_POST_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        ReleasePostResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::RELEASE_POST_GENERIC_ERROR_RESPONSE.clone()));

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
        "ReleasePost",
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

                match api.work_id_get(param_id, context).wait() {
                    Ok(rsp) => match rsp {
                        WorkIdGetResponse::FetchASingleWorkById(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(200), body_string));
                            response.headers.set(ContentType(mimetypes::responses::WORK_ID_GET_FETCH_A_SINGLE_WORK_BY_ID.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        WorkIdGetResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::WORK_ID_GET_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));

                            Ok(response)
                        }
                        WorkIdGetResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::WORK_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

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
        "WorkIdGet",
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

                let param_body = req.get::<bodyparser::Raw>().unwrap_or(None);

                let mut unused_elements = Vec::new();

                let param_body = if let Some(param_body_raw) = param_body {
                    let deserializer = &mut serde_json::Deserializer::from_str(&param_body_raw);

                    let param_body: Option<models::WorkEntity> = serde_ignored::deserialize(deserializer, |path| {
                        warn!("Ignoring unknown field in body: {}", path);
                        unused_elements.push(path.to_string());
                    }).unwrap_or(None);

                    param_body
                } else {
                    None
                };

                match api.work_post(param_body, context).wait() {
                    Ok(rsp) => match rsp {
                        WorkPostResponse::Created(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(201), body_string));
                            response.headers.set(ContentType(mimetypes::responses::WORK_POST_CREATED.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        WorkPostResponse::BadRequest(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(400), body_string));
                            response.headers.set(ContentType(mimetypes::responses::WORK_POST_BAD_REQUEST.clone()));

                            context.x_span_id.as_ref().map(|header| response.headers.set(XSpanId(header.clone())));
                            if !unused_elements.is_empty() {
                                response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));
                            }
                            Ok(response)
                        }
                        WorkPostResponse::GenericErrorResponse(body) => {
                            let body_string = serde_json::to_string(&body).expect("impossible to fail to serialize");

                            let mut response = Response::with((status::Status::from_u16(0), body_string));
                            response.headers.set(ContentType(mimetypes::responses::WORK_POST_GENERIC_ERROR_RESPONSE.clone()));

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
        "WorkPost",
    );
}

/// Middleware to extract authentication data from request
pub struct ExtractAuthData;

impl BeforeMiddleware for ExtractAuthData {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        Ok(())
    }
}
