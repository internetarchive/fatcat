#![allow(unused_extern_crates)]
extern crate chrono;
extern crate hyper_tls;
extern crate mime;
extern crate native_tls;
extern crate openssl;
extern crate serde_ignored;
extern crate tokio_core;
extern crate uuid;

extern crate percent_encoding;
extern crate url;

use self::url::form_urlencoded;
use futures::{future, stream, Future, Stream};
use hyper;
use hyper::header::{ContentType, Headers};
use hyper::{Error, Request, Response, StatusCode};
use mimetypes;
use std::sync::Arc;

use serde_json;

#[allow(unused_imports)]
use std::collections::{BTreeMap, HashMap};
use std::io;
#[allow(unused_imports)]
use swagger;

#[allow(unused_imports)]
use std::collections::BTreeSet;

pub use swagger::auth::Authorization;
use swagger::auth::Scopes;
use swagger::{ApiError, Context, XSpanId};

#[allow(unused_imports)]
use models;
use {Api, ContainerIdGetResponse, ContainerLookupGetResponse, ContainerPostResponse,
     CreatorIdGetResponse, CreatorLookupGetResponse, CreatorPostResponse,
     EditgroupIdAcceptPostResponse, EditgroupIdGetResponse, EditgroupPostResponse,
     EditorUsernameChangelogGetResponse, EditorUsernameGetResponse, FileIdGetResponse,
     FileLookupGetResponse, FilePostResponse, ReleaseIdGetResponse, ReleaseLookupGetResponse,
     ReleasePostResponse, WorkIdGetResponse, WorkPostResponse};

pub mod auth;

header! { (Warning, "Warning") => [String] }

mod paths {
    extern crate regex;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet = regex::RegexSet::new(&[
            r"^/v0/container$",
            r"^/v0/container/lookup$",
            r"^/v0/container/(?P<id>[^/?#]*)$",
            r"^/v0/creator$",
            r"^/v0/creator/lookup$",
            r"^/v0/creator/(?P<id>[^/?#]*)$",
            r"^/v0/editgroup$",
            r"^/v0/editgroup/(?P<id>[^/?#]*)$",
            r"^/v0/editgroup/(?P<id>[^/?#]*)/accept$",
            r"^/v0/editor/(?P<username>[^/?#]*)$",
            r"^/v0/editor/(?P<username>[^/?#]*)/changelog$",
            r"^/v0/file$",
            r"^/v0/file/lookup$",
            r"^/v0/file/(?P<id>[^/?#]*)$",
            r"^/v0/release$",
            r"^/v0/release/lookup$",
            r"^/v0/release/(?P<id>[^/?#]*)$",
            r"^/v0/work$",
            r"^/v0/work/(?P<id>[^/?#]*)$"
        ]).unwrap();
    }
    pub static ID_CONTAINER: usize = 0;
    pub static ID_CONTAINER_LOOKUP: usize = 1;
    pub static ID_CONTAINER_ID: usize = 2;
    lazy_static! {
        pub static ref REGEX_CONTAINER_ID: regex::Regex =
            regex::Regex::new(r"^/v0/container/(?P<id>[^/?#]*)$").unwrap();
    }
    pub static ID_CREATOR: usize = 3;
    pub static ID_CREATOR_LOOKUP: usize = 4;
    pub static ID_CREATOR_ID: usize = 5;
    lazy_static! {
        pub static ref REGEX_CREATOR_ID: regex::Regex =
            regex::Regex::new(r"^/v0/creator/(?P<id>[^/?#]*)$").unwrap();
    }
    pub static ID_EDITGROUP: usize = 6;
    pub static ID_EDITGROUP_ID: usize = 7;
    lazy_static! {
        pub static ref REGEX_EDITGROUP_ID: regex::Regex =
            regex::Regex::new(r"^/v0/editgroup/(?P<id>[^/?#]*)$").unwrap();
    }
    pub static ID_EDITGROUP_ID_ACCEPT: usize = 8;
    lazy_static! {
        pub static ref REGEX_EDITGROUP_ID_ACCEPT: regex::Regex =
            regex::Regex::new(r"^/v0/editgroup/(?P<id>[^/?#]*)/accept$").unwrap();
    }
    pub static ID_EDITOR_USERNAME: usize = 9;
    lazy_static! {
        pub static ref REGEX_EDITOR_USERNAME: regex::Regex =
            regex::Regex::new(r"^/v0/editor/(?P<username>[^/?#]*)$").unwrap();
    }
    pub static ID_EDITOR_USERNAME_CHANGELOG: usize = 10;
    lazy_static! {
        pub static ref REGEX_EDITOR_USERNAME_CHANGELOG: regex::Regex =
            regex::Regex::new(r"^/v0/editor/(?P<username>[^/?#]*)/changelog$").unwrap();
    }
    pub static ID_FILE: usize = 11;
    pub static ID_FILE_LOOKUP: usize = 12;
    pub static ID_FILE_ID: usize = 13;
    lazy_static! {
        pub static ref REGEX_FILE_ID: regex::Regex =
            regex::Regex::new(r"^/v0/file/(?P<id>[^/?#]*)$").unwrap();
    }
    pub static ID_RELEASE: usize = 14;
    pub static ID_RELEASE_LOOKUP: usize = 15;
    pub static ID_RELEASE_ID: usize = 16;
    lazy_static! {
        pub static ref REGEX_RELEASE_ID: regex::Regex =
            regex::Regex::new(r"^/v0/release/(?P<id>[^/?#]*)$").unwrap();
    }
    pub static ID_WORK: usize = 17;
    pub static ID_WORK_ID: usize = 18;
    lazy_static! {
        pub static ref REGEX_WORK_ID: regex::Regex =
            regex::Regex::new(r"^/v0/work/(?P<id>[^/?#]*)$").unwrap();
    }
}

pub struct NewService<T> {
    api_impl: Arc<T>,
}

impl<T> NewService<T>
where
    T: Api + Clone + 'static,
{
    pub fn new<U: Into<Arc<T>>>(api_impl: U) -> NewService<T> {
        NewService {
            api_impl: api_impl.into(),
        }
    }
}

impl<T> hyper::server::NewService for NewService<T>
where
    T: Api + Clone + 'static,
{
    type Request = (Request, Context);
    type Response = Response;
    type Error = Error;
    type Instance = Service<T>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        Ok(Service::new(self.api_impl.clone()))
    }
}

pub struct Service<T> {
    api_impl: Arc<T>,
}

impl<T> Service<T>
where
    T: Api + Clone + 'static,
{
    pub fn new<U: Into<Arc<T>>>(api_impl: U) -> Service<T> {
        Service {
            api_impl: api_impl.into(),
        }
    }
}

impl<T> hyper::server::Service for Service<T>
where
    T: Api + Clone + 'static,
{
    type Request = (Request, Context);
    type Response = Response;
    type Error = Error;
    type Future = Box<Future<Item = Response, Error = Error>>;

    fn call(&self, (req, mut context): Self::Request) -> Self::Future {
        let api_impl = self.api_impl.clone();
        let (method, uri, _, headers, body) = req.deconstruct();
        let path = paths::GLOBAL_REGEX_SET.matches(uri.path());
        match &method {
            // ContainerIdGet - GET /container/{id}
            &hyper::Method::Get if path.matched(paths::ID_CONTAINER_ID) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Path parameters
                let path = uri.path().to_string();
                let path_params = paths::REGEX_CONTAINER_ID
                    .captures(&path)
                    .unwrap_or_else(|| {
                        panic!("Path {} matched RE CONTAINER_ID in set but failed match against \"{}\"", path, paths::REGEX_CONTAINER_ID.as_str())
                    });

                let param_id = match percent_encoding::percent_decode(path_params["id"].as_bytes())
                    .decode_utf8()
                {
                    Ok(param_id) => match param_id.parse::<String>() {
                        Ok(param_id) => param_id,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't parse path parameter id: {}", e)),
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["id"]
                                )),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.container_id_get(param_id, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ContainerIdGetResponse::FetchASingleContainerById

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(200).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_ID_GET_FETCH_A_SINGLE_CONTAINER_BY_ID.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                ContainerIdGetResponse::BadRequest

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(400).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_ID_GET_BAD_REQUEST.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                ContainerIdGetResponse::GenericErrorResponse

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(0).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_ID_GET_GENERIC_ERROR_RESPONSE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            },
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // ContainerLookupGet - GET /container/lookup
            &hyper::Method::Get if path.matched(paths::ID_CONTAINER_LOOKUP) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = form_urlencoded::parse(
                    uri.query().unwrap_or_default().as_bytes(),
                ).collect::<Vec<_>>();
                let param_issn = query_params
                    .iter()
                    .filter(|e| e.0 == "issn")
                    .map(|e| e.1.to_owned())
                    .nth(0);
                let param_issn = match param_issn {
                    Some(param_issn) => match param_issn.parse::<String>() {
                        Ok(param_issn) => param_issn,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!(
                                "Couldn't parse query parameter issn - doesn't match schema: {}",
                                e
                            )),
                            ))
                        }
                    },
                    None => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body("Missing required query parameter issn"),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.container_lookup_get(param_issn, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ContainerLookupGetResponse::FindASingleContainerByExternalIdentifer

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(200).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_LOOKUP_GET_FIND_A_SINGLE_CONTAINER_BY_EXTERNAL_IDENTIFER.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                ContainerLookupGetResponse::BadRequest

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(400).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_LOOKUP_GET_BAD_REQUEST.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                ContainerLookupGetResponse::NoSuchContainer

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(404).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_LOOKUP_GET_NO_SUCH_CONTAINER.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                ContainerLookupGetResponse::GenericErrorResponse

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(0).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_LOOKUP_GET_GENERIC_ERROR_RESPONSE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            },
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // ContainerPost - POST /container
            &hyper::Method::Post if path.matched(paths::ID_CONTAINER) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                Box::new(body.concat2().then(
                    move |result| -> Box<Future<Item = Response, Error = Error>> {
                        match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_body: Option<
                                    models::ContainerEntity,
                                > = if !body.is_empty() {
                                    let deserializer =
                                        &mut serde_json::Deserializer::from_slice(&*body);

                                    match serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {}", path);
                                        unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_body) => param_body,

                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                Box::new(api_impl.container_post(param_body, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().set(Warning(format!(
                                                "Ignoring unknown fields in body: {:?}",
                                                unused_elements
                                            )));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ContainerPostResponse::Created(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(201).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_POST_CREATED.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                ContainerPostResponse::BadRequest(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(400).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_POST_BAD_REQUEST.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                ContainerPostResponse::GenericErrorResponse(
                                                    body,
                                                ) => {
                                                    response.set_status(
                                                        StatusCode::try_from(0).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CONTAINER_POST_GENERIC_ERROR_RESPONSE.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response
                                                    .set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            }
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                            Err(e) => Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't read body parameter body: {}", e)),
                            )),
                        }
                    },
                )) as Box<Future<Item = Response, Error = Error>>
            }

            // CreatorIdGet - GET /creator/{id}
            &hyper::Method::Get if path.matched(paths::ID_CREATOR_ID) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Path parameters
                let path = uri.path().to_string();
                let path_params = paths::REGEX_CREATOR_ID.captures(&path).unwrap_or_else(|| {
                    panic!(
                        "Path {} matched RE CREATOR_ID in set but failed match against \"{}\"",
                        path,
                        paths::REGEX_CREATOR_ID.as_str()
                    )
                });

                let param_id = match percent_encoding::percent_decode(path_params["id"].as_bytes())
                    .decode_utf8()
                {
                    Ok(param_id) => match param_id.parse::<String>() {
                        Ok(param_id) => param_id,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't parse path parameter id: {}", e)),
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["id"]
                                )),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.creator_id_get(param_id, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                CreatorIdGetResponse::FetchASingleCreatorById(
                                                    body,
                                                ) => {
                                                    response.set_status(
                                                        StatusCode::try_from(200).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CREATOR_ID_GET_FETCH_A_SINGLE_CREATOR_BY_ID.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                CreatorIdGetResponse::BadRequest(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(400).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CREATOR_ID_GET_BAD_REQUEST.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                CreatorIdGetResponse::GenericErrorResponse(
                                                    body,
                                                ) => {
                                                    response.set_status(
                                                        StatusCode::try_from(0).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CREATOR_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response
                                                    .set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            }
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // CreatorLookupGet - GET /creator/lookup
            &hyper::Method::Get if path.matched(paths::ID_CREATOR_LOOKUP) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = form_urlencoded::parse(
                    uri.query().unwrap_or_default().as_bytes(),
                ).collect::<Vec<_>>();
                let param_orcid = query_params
                    .iter()
                    .filter(|e| e.0 == "orcid")
                    .map(|e| e.1.to_owned())
                    .nth(0);
                let param_orcid = match param_orcid {
                    Some(param_orcid) => match param_orcid.parse::<String>() {
                        Ok(param_orcid) => param_orcid,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!(
                                "Couldn't parse query parameter orcid - doesn't match schema: {}",
                                e
                            )),
                            ))
                        }
                    },
                    None => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body("Missing required query parameter orcid"),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.creator_lookup_get(param_orcid, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                CreatorLookupGetResponse::FindASingleCreatorByExternalIdentifer

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(200).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CREATOR_LOOKUP_GET_FIND_A_SINGLE_CREATOR_BY_EXTERNAL_IDENTIFER.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                CreatorLookupGetResponse::BadRequest

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(400).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CREATOR_LOOKUP_GET_BAD_REQUEST.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                CreatorLookupGetResponse::NoSuchCreator

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(404).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CREATOR_LOOKUP_GET_NO_SUCH_CREATOR.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                CreatorLookupGetResponse::GenericErrorResponse

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(0).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CREATOR_LOOKUP_GET_GENERIC_ERROR_RESPONSE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            },
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // CreatorPost - POST /creator
            &hyper::Method::Post if path.matched(paths::ID_CREATOR) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                Box::new(body.concat2().then(
                    move |result| -> Box<Future<Item = Response, Error = Error>> {
                        match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_body: Option<
                                    models::CreatorEntity,
                                > = if !body.is_empty() {
                                    let deserializer =
                                        &mut serde_json::Deserializer::from_slice(&*body);

                                    match serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {}", path);
                                        unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_body) => param_body,

                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                Box::new(api_impl.creator_post(param_body, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().set(Warning(format!(
                                                "Ignoring unknown fields in body: {:?}",
                                                unused_elements
                                            )));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                CreatorPostResponse::Created(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(201).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(
                                                        mimetypes::responses::CREATOR_POST_CREATED
                                                            .clone(),
                                                    ));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                CreatorPostResponse::BadRequest(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(400).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CREATOR_POST_BAD_REQUEST.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                CreatorPostResponse::GenericErrorResponse(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(0).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::CREATOR_POST_GENERIC_ERROR_RESPONSE.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response
                                                    .set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            }
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                            Err(e) => Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't read body parameter body: {}", e)),
                            )),
                        }
                    },
                )) as Box<Future<Item = Response, Error = Error>>
            }

            // EditgroupIdAcceptPost - POST /editgroup/{id}/accept
            &hyper::Method::Post if path.matched(paths::ID_EDITGROUP_ID_ACCEPT) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Path parameters
                let path = uri.path().to_string();
                let path_params = paths::REGEX_EDITGROUP_ID_ACCEPT
                    .captures(&path)
                    .unwrap_or_else(|| {
                        panic!("Path {} matched RE EDITGROUP_ID_ACCEPT in set but failed match against \"{}\"", path, paths::REGEX_EDITGROUP_ID_ACCEPT.as_str())
                    });

                let param_id = match percent_encoding::percent_decode(path_params["id"].as_bytes())
                    .decode_utf8()
                {
                    Ok(param_id) => match param_id.parse::<i32>() {
                        Ok(param_id) => param_id,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't parse path parameter id: {}", e)),
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["id"]
                                )),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(
                                    api_impl.editgroup_id_accept_post(param_id, &context).then(
                                        move |result| {
                                            let mut response = Response::new();
                                            context.x_span_id.as_ref().map(|header| {
                                                response.headers_mut().set(XSpanId(header.clone()))
                                            });

                                            match result {
                                            Ok(rsp) => match rsp {
                                                EditgroupIdAcceptPostResponse::MergedEditgroupSuccessfully_

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(200).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_ID_ACCEPT_POST_MERGED_EDITGROUP_SUCCESSFULLY_.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                EditgroupIdAcceptPostResponse::EditgroupIsInAnUnmergableState

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(400).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_ID_ACCEPT_POST_EDITGROUP_IS_IN_AN_UNMERGABLE_STATE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                EditgroupIdAcceptPostResponse::NoSuchEditgroup

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(404).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_ID_ACCEPT_POST_NO_SUCH_EDITGROUP.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                EditgroupIdAcceptPostResponse::GenericErrorResponse

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(0).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_ID_ACCEPT_POST_GENERIC_ERROR_RESPONSE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            },
                                        }

                                            future::ok(response)
                                        },
                                    ),
                                )
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // EditgroupIdGet - GET /editgroup/{id}
            &hyper::Method::Get if path.matched(paths::ID_EDITGROUP_ID) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Path parameters
                let path = uri.path().to_string();
                let path_params = paths::REGEX_EDITGROUP_ID
                    .captures(&path)
                    .unwrap_or_else(|| {
                        panic!("Path {} matched RE EDITGROUP_ID in set but failed match against \"{}\"", path, paths::REGEX_EDITGROUP_ID.as_str())
                    });

                let param_id = match percent_encoding::percent_decode(path_params["id"].as_bytes())
                    .decode_utf8()
                {
                    Ok(param_id) => match param_id.parse::<i32>() {
                        Ok(param_id) => param_id,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't parse path parameter id: {}", e)),
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["id"]
                                )),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.editgroup_id_get(param_id, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                EditgroupIdGetResponse::FetchEditgroupByIdentifier

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(200).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_ID_GET_FETCH_EDITGROUP_BY_IDENTIFIER.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                EditgroupIdGetResponse::NoSuchEditgroup

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(404).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_ID_GET_NO_SUCH_EDITGROUP.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                EditgroupIdGetResponse::GenericErrorResponse

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(0).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_ID_GET_GENERIC_ERROR_RESPONSE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            },
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // EditgroupPost - POST /editgroup
            &hyper::Method::Post if path.matched(paths::ID_EDITGROUP) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.editgroup_post(&context).then(move |result| {
                                    let mut response = Response::new();
                                    context.x_span_id.as_ref().map(|header| {
                                        response.headers_mut().set(XSpanId(header.clone()))
                                    });

                                    match result {
                                        Ok(rsp) => match rsp {
                                            EditgroupPostResponse::SuccessfullyCreated(body) => {
                                                response
                                                    .set_status(StatusCode::try_from(201).unwrap());

                                                response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_POST_SUCCESSFULLY_CREATED.clone()));

                                                let body = serde_json::to_string(&body)
                                                    .expect("impossible to fail to serialize");

                                                response.set_body(body);
                                            }
                                            EditgroupPostResponse::InvalidRequestParameters(
                                                body,
                                            ) => {
                                                response
                                                    .set_status(StatusCode::try_from(400).unwrap());

                                                response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_POST_INVALID_REQUEST_PARAMETERS.clone()));

                                                let body = serde_json::to_string(&body)
                                                    .expect("impossible to fail to serialize");

                                                response.set_body(body);
                                            }
                                            EditgroupPostResponse::GenericErrorResponse(body) => {
                                                response
                                                    .set_status(StatusCode::try_from(0).unwrap());

                                                response.headers_mut().set(ContentType(mimetypes::responses::EDITGROUP_POST_GENERIC_ERROR_RESPONSE.clone()));

                                                let body = serde_json::to_string(&body)
                                                    .expect("impossible to fail to serialize");

                                                response.set_body(body);
                                            }
                                        },
                                        Err(_) => {
                                            // Application code returned an error. This should not happen, as the implementation should
                                            // return a valid response.
                                            response.set_status(StatusCode::InternalServerError);
                                            response.set_body("An internal error occurred");
                                        }
                                    }

                                    future::ok(response)
                                }))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // EditorUsernameChangelogGet - GET /editor/{username}/changelog
            &hyper::Method::Get if path.matched(paths::ID_EDITOR_USERNAME_CHANGELOG) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Path parameters
                let path = uri.path().to_string();
                let path_params = paths::REGEX_EDITOR_USERNAME_CHANGELOG
                    .captures(&path)
                    .unwrap_or_else(|| {
                        panic!("Path {} matched RE EDITOR_USERNAME_CHANGELOG in set but failed match against \"{}\"", path, paths::REGEX_EDITOR_USERNAME_CHANGELOG.as_str())
                    });

                let param_username = match percent_encoding::percent_decode(
                    path_params["username"].as_bytes(),
                ).decode_utf8()
                {
                    Ok(param_username) => match param_username.parse::<String>() {
                        Ok(param_username) => param_username,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!(
                                        "Couldn't parse path parameter username: {}",
                                        e
                                    )),
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["username"]
                                )),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(
                                    api_impl
                                        .editor_username_changelog_get(param_username, &context)
                                        .then(move |result| {
                                            let mut response = Response::new();
                                            context.x_span_id.as_ref().map(|header| {
                                                response.headers_mut().set(XSpanId(header.clone()))
                                            });

                                            match result {
                                            Ok(rsp) => match rsp {
                                                EditorUsernameChangelogGetResponse::FindChanges_

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(200).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITOR_USERNAME_CHANGELOG_GET_FIND_CHANGES_.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                EditorUsernameChangelogGetResponse::UsernameNotFound

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(404).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITOR_USERNAME_CHANGELOG_GET_USERNAME_NOT_FOUND.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                EditorUsernameChangelogGetResponse::GenericErrorResponse

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(0).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITOR_USERNAME_CHANGELOG_GET_GENERIC_ERROR_RESPONSE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            },
                                        }

                                            future::ok(response)
                                        }),
                                )
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // EditorUsernameGet - GET /editor/{username}
            &hyper::Method::Get if path.matched(paths::ID_EDITOR_USERNAME) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Path parameters
                let path = uri.path().to_string();
                let path_params = paths::REGEX_EDITOR_USERNAME.captures(&path).unwrap_or_else(
                    || {
                        panic!("Path {} matched RE EDITOR_USERNAME in set but failed match against \"{}\"", path, paths::REGEX_EDITOR_USERNAME.as_str())
                    },
                );

                let param_username = match percent_encoding::percent_decode(
                    path_params["username"].as_bytes(),
                ).decode_utf8()
                {
                    Ok(param_username) => match param_username.parse::<String>() {
                        Ok(param_username) => param_username,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!(
                                        "Couldn't parse path parameter username: {}",
                                        e
                                    )),
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["username"]
                                )),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(
                                    api_impl.editor_username_get(param_username, &context).then(
                                        move |result| {
                                            let mut response = Response::new();
                                            context.x_span_id.as_ref().map(|header| {
                                                response.headers_mut().set(XSpanId(header.clone()))
                                            });

                                            match result {
                                            Ok(rsp) => match rsp {
                                                EditorUsernameGetResponse::FetchGenericInformationAboutAnEditor

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(200).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITOR_USERNAME_GET_FETCH_GENERIC_INFORMATION_ABOUT_AN_EDITOR.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                EditorUsernameGetResponse::UsernameNotFound

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(404).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITOR_USERNAME_GET_USERNAME_NOT_FOUND.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                EditorUsernameGetResponse::GenericErrorResponse

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(0).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::EDITOR_USERNAME_GET_GENERIC_ERROR_RESPONSE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            },
                                        }

                                            future::ok(response)
                                        },
                                    ),
                                )
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // FileIdGet - GET /file/{id}
            &hyper::Method::Get if path.matched(paths::ID_FILE_ID) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Path parameters
                let path = uri.path().to_string();
                let path_params = paths::REGEX_FILE_ID.captures(&path).unwrap_or_else(|| {
                    panic!(
                        "Path {} matched RE FILE_ID in set but failed match against \"{}\"",
                        path,
                        paths::REGEX_FILE_ID.as_str()
                    )
                });

                let param_id = match percent_encoding::percent_decode(path_params["id"].as_bytes())
                    .decode_utf8()
                {
                    Ok(param_id) => match param_id.parse::<String>() {
                        Ok(param_id) => param_id,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't parse path parameter id: {}", e)),
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["id"]
                                )),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.file_id_get(param_id, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                FileIdGetResponse::FetchASingleFileById(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(200).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::FILE_ID_GET_FETCH_A_SINGLE_FILE_BY_ID.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                FileIdGetResponse::BadRequest(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(400).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::FILE_ID_GET_BAD_REQUEST.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                FileIdGetResponse::GenericErrorResponse(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(0).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::FILE_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response
                                                    .set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            }
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // FileLookupGet - GET /file/lookup
            &hyper::Method::Get if path.matched(paths::ID_FILE_LOOKUP) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = form_urlencoded::parse(
                    uri.query().unwrap_or_default().as_bytes(),
                ).collect::<Vec<_>>();
                let param_sha1 = query_params
                    .iter()
                    .filter(|e| e.0 == "sha1")
                    .map(|e| e.1.to_owned())
                    .nth(0);
                let param_sha1 = match param_sha1 {
                    Some(param_sha1) => match param_sha1.parse::<String>() {
                        Ok(param_sha1) => param_sha1,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!(
                                "Couldn't parse query parameter sha1 - doesn't match schema: {}",
                                e
                            )),
                            ))
                        }
                    },
                    None => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body("Missing required query parameter sha1"),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.file_lookup_get(param_sha1, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                FileLookupGetResponse::FindASingleFileByExternalIdentifer

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(200).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::FILE_LOOKUP_GET_FIND_A_SINGLE_FILE_BY_EXTERNAL_IDENTIFER.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                FileLookupGetResponse::BadRequest

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(400).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::FILE_LOOKUP_GET_BAD_REQUEST.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                FileLookupGetResponse::NoSuchFile

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(404).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::FILE_LOOKUP_GET_NO_SUCH_FILE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                FileLookupGetResponse::GenericErrorResponse

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(0).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::FILE_LOOKUP_GET_GENERIC_ERROR_RESPONSE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            },
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // FilePost - POST /file
            &hyper::Method::Post if path.matched(paths::ID_FILE) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                Box::new(body.concat2().then(
                    move |result| -> Box<Future<Item = Response, Error = Error>> {
                        match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_body: Option<
                                    models::FileEntity,
                                > = if !body.is_empty() {
                                    let deserializer =
                                        &mut serde_json::Deserializer::from_slice(&*body);

                                    match serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {}", path);
                                        unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_body) => param_body,

                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                Box::new(api_impl.file_post(param_body, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().set(Warning(format!(
                                                "Ignoring unknown fields in body: {:?}",
                                                unused_elements
                                            )));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                FilePostResponse::Created(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(201).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(
                                                        mimetypes::responses::FILE_POST_CREATED
                                                            .clone(),
                                                    ));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                FilePostResponse::BadRequest(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(400).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(
                                                        mimetypes::responses::FILE_POST_BAD_REQUEST
                                                            .clone(),
                                                    ));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                FilePostResponse::GenericErrorResponse(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(0).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::FILE_POST_GENERIC_ERROR_RESPONSE.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response
                                                    .set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            }
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                            Err(e) => Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't read body parameter body: {}", e)),
                            )),
                        }
                    },
                )) as Box<Future<Item = Response, Error = Error>>
            }

            // ReleaseIdGet - GET /release/{id}
            &hyper::Method::Get if path.matched(paths::ID_RELEASE_ID) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Path parameters
                let path = uri.path().to_string();
                let path_params = paths::REGEX_RELEASE_ID.captures(&path).unwrap_or_else(|| {
                    panic!(
                        "Path {} matched RE RELEASE_ID in set but failed match against \"{}\"",
                        path,
                        paths::REGEX_RELEASE_ID.as_str()
                    )
                });

                let param_id = match percent_encoding::percent_decode(path_params["id"].as_bytes())
                    .decode_utf8()
                {
                    Ok(param_id) => match param_id.parse::<String>() {
                        Ok(param_id) => param_id,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't parse path parameter id: {}", e)),
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["id"]
                                )),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.release_id_get(param_id, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ReleaseIdGetResponse::FetchASingleReleaseById(
                                                    body,
                                                ) => {
                                                    response.set_status(
                                                        StatusCode::try_from(200).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::RELEASE_ID_GET_FETCH_A_SINGLE_RELEASE_BY_ID.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                ReleaseIdGetResponse::BadRequest(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(400).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::RELEASE_ID_GET_BAD_REQUEST.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                ReleaseIdGetResponse::GenericErrorResponse(
                                                    body,
                                                ) => {
                                                    response.set_status(
                                                        StatusCode::try_from(0).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::RELEASE_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response
                                                    .set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            }
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // ReleaseLookupGet - GET /release/lookup
            &hyper::Method::Get if path.matched(paths::ID_RELEASE_LOOKUP) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = form_urlencoded::parse(
                    uri.query().unwrap_or_default().as_bytes(),
                ).collect::<Vec<_>>();
                let param_doi = query_params
                    .iter()
                    .filter(|e| e.0 == "doi")
                    .map(|e| e.1.to_owned())
                    .nth(0);
                let param_doi = match param_doi {
                    Some(param_doi) => match param_doi.parse::<String>() {
                        Ok(param_doi) => param_doi,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!(
                                    "Couldn't parse query parameter doi - doesn't match schema: {}",
                                    e
                                )),
                            ))
                        }
                    },
                    None => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body("Missing required query parameter doi"),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.release_lookup_get(param_doi, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ReleaseLookupGetResponse::FindASingleReleaseByExternalIdentifer

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(200).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::RELEASE_LOOKUP_GET_FIND_A_SINGLE_RELEASE_BY_EXTERNAL_IDENTIFER.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                ReleaseLookupGetResponse::BadRequest

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(400).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::RELEASE_LOOKUP_GET_BAD_REQUEST.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                ReleaseLookupGetResponse::NoSuchRelease

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(404).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::RELEASE_LOOKUP_GET_NO_SUCH_RELEASE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                                ReleaseLookupGetResponse::GenericErrorResponse

                                                    (body)


                                                => {
                                                    response.set_status(StatusCode::try_from(0).unwrap());

                                                    response.headers_mut().set(ContentType(mimetypes::responses::RELEASE_LOOKUP_GET_GENERIC_ERROR_RESPONSE.clone()));


                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response.set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            },
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // ReleasePost - POST /release
            &hyper::Method::Post if path.matched(paths::ID_RELEASE) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                Box::new(body.concat2().then(
                    move |result| -> Box<Future<Item = Response, Error = Error>> {
                        match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_body: Option<
                                    models::ReleaseEntity,
                                > = if !body.is_empty() {
                                    let deserializer =
                                        &mut serde_json::Deserializer::from_slice(&*body);

                                    match serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {}", path);
                                        unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_body) => param_body,

                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                Box::new(api_impl.release_post(param_body, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().set(Warning(format!(
                                                "Ignoring unknown fields in body: {:?}",
                                                unused_elements
                                            )));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ReleasePostResponse::Created(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(201).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(
                                                        mimetypes::responses::RELEASE_POST_CREATED
                                                            .clone(),
                                                    ));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                ReleasePostResponse::BadRequest(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(400).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::RELEASE_POST_BAD_REQUEST.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                ReleasePostResponse::GenericErrorResponse(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(0).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::RELEASE_POST_GENERIC_ERROR_RESPONSE.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response
                                                    .set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            }
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                            Err(e) => Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't read body parameter body: {}", e)),
                            )),
                        }
                    },
                )) as Box<Future<Item = Response, Error = Error>>
            }

            // WorkIdGet - GET /work/{id}
            &hyper::Method::Get if path.matched(paths::ID_WORK_ID) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Path parameters
                let path = uri.path().to_string();
                let path_params = paths::REGEX_WORK_ID.captures(&path).unwrap_or_else(|| {
                    panic!(
                        "Path {} matched RE WORK_ID in set but failed match against \"{}\"",
                        path,
                        paths::REGEX_WORK_ID.as_str()
                    )
                });

                let param_id = match percent_encoding::percent_decode(path_params["id"].as_bytes())
                    .decode_utf8()
                {
                    Ok(param_id) => match param_id.parse::<String>() {
                        Ok(param_id) => param_id,
                        Err(e) => {
                            return Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't parse path parameter id: {}", e)),
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::new(future::ok(
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!(
                                    "Couldn't percent-decode path parameter as UTF-8: {}",
                                    &path_params["id"]
                                )),
                        ))
                    }
                };

                Box::new(
                    ({
                        {
                            {
                                Box::new(api_impl.work_id_get(param_id, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        match result {
                                            Ok(rsp) => match rsp {
                                                WorkIdGetResponse::FetchASingleWorkById(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(200).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::WORK_ID_GET_FETCH_A_SINGLE_WORK_BY_ID.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                WorkIdGetResponse::BadRequest(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(400).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::WORK_ID_GET_BAD_REQUEST.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                WorkIdGetResponse::GenericErrorResponse(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(0).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::WORK_ID_GET_GENERIC_ERROR_RESPONSE.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response
                                                    .set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            }
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                        }
                    }),
                ) as Box<Future<Item = Response, Error = Error>>
            }

            // WorkPost - POST /work
            &hyper::Method::Post if path.matched(paths::ID_WORK) => {
                if context.x_span_id.is_none() {
                    context.x_span_id = Some(
                        headers
                            .get::<XSpanId>()
                            .map(XSpanId::to_string)
                            .unwrap_or_else(|| self::uuid::Uuid::new_v4().to_string()),
                    );
                }

                // Body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                Box::new(body.concat2().then(
                    move |result| -> Box<Future<Item = Response, Error = Error>> {
                        match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_body: Option<
                                    models::WorkEntity,
                                > = if !body.is_empty() {
                                    let deserializer =
                                        &mut serde_json::Deserializer::from_slice(&*body);

                                    match serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {}", path);
                                        unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_body) => param_body,

                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                Box::new(api_impl.work_post(param_body, &context).then(
                                    move |result| {
                                        let mut response = Response::new();
                                        context.x_span_id.as_ref().map(|header| {
                                            response.headers_mut().set(XSpanId(header.clone()))
                                        });

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().set(Warning(format!(
                                                "Ignoring unknown fields in body: {:?}",
                                                unused_elements
                                            )));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                WorkPostResponse::Created(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(201).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(
                                                        mimetypes::responses::WORK_POST_CREATED
                                                            .clone(),
                                                    ));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                WorkPostResponse::BadRequest(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(400).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(
                                                        mimetypes::responses::WORK_POST_BAD_REQUEST
                                                            .clone(),
                                                    ));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                                WorkPostResponse::GenericErrorResponse(body) => {
                                                    response.set_status(
                                                        StatusCode::try_from(0).unwrap(),
                                                    );

                                                    response.headers_mut().set(ContentType(mimetypes::responses::WORK_POST_GENERIC_ERROR_RESPONSE.clone()));

                                                    let body = serde_json::to_string(&body)
                                                        .expect("impossible to fail to serialize");

                                                    response.set_body(body);
                                                }
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                response
                                                    .set_status(StatusCode::InternalServerError);
                                                response.set_body("An internal error occurred");
                                            }
                                        }

                                        future::ok(response)
                                    },
                                ))
                            }
                            Err(e) => Box::new(future::ok(
                                Response::new()
                                    .with_status(StatusCode::BadRequest)
                                    .with_body(format!("Couldn't read body parameter body: {}", e)),
                            )),
                        }
                    },
                )) as Box<Future<Item = Response, Error = Error>>
            }

            _ => Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound),
            )) as Box<Future<Item = Response, Error = Error>>,
        }
    }
}
