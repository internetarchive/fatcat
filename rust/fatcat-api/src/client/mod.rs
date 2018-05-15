#![allow(unused_extern_crates)]
extern crate chrono;
extern crate hyper_tls;
extern crate mime;
extern crate native_tls;
extern crate openssl;
extern crate tokio_core;
extern crate url;

use self::tokio_core::reactor::Handle;
use self::url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET, QUERY_ENCODE_SET};
use futures;
use futures::{Future, Stream};
use futures::{future, stream};
use hyper;
use hyper::Uri;
use hyper::header::{ContentType, Headers};
use std::borrow::Cow;
use std::error;
use std::fmt;
use std::io::{Error, ErrorKind, Read};
use std::path::Path;
use std::str;
use std::str::FromStr;
use std::sync::Arc;

use mimetypes;

use serde_json;

#[allow(unused_imports)]
use std::collections::{BTreeMap, HashMap};
#[allow(unused_imports)]
use swagger;

use swagger::{ApiError, Context, XSpanId};

use models;
use {Api, ContainerIdGetResponse, ContainerLookupGetResponse, ContainerPostResponse,
     CreatorIdGetResponse, CreatorLookupGetResponse, CreatorPostResponse,
     EditgroupIdAcceptPostResponse, EditgroupIdGetResponse, EditgroupPostResponse,
     EditorUsernameChangelogGetResponse, EditorUsernameGetResponse, FileIdGetResponse,
     FileLookupGetResponse, FilePostResponse, ReleaseIdGetResponse, ReleaseLookupGetResponse,
     ReleasePostResponse, WorkIdGetResponse, WorkPostResponse};

/// Convert input into a base path, e.g. "http://example:123". Also checks the scheme as it goes.
fn into_base_path(
    input: &str,
    correct_scheme: Option<&'static str>,
) -> Result<String, ClientInitError> {
    // First convert to Uri, since a base path is a subset of Uri.
    let uri = Uri::from_str(input)?;

    let scheme = uri.scheme().ok_or(ClientInitError::InvalidScheme)?;

    // Check the scheme if necessary
    if let Some(correct_scheme) = correct_scheme {
        if scheme != correct_scheme {
            return Err(ClientInitError::InvalidScheme);
        }
    }

    let host = uri.host().ok_or_else(|| ClientInitError::MissingHost)?;
    let port = uri.port().map(|x| format!(":{}", x)).unwrap_or_default();
    Ok(format!("{}://{}{}", scheme, host, port))
}

/// A client that implements the API by making HTTP calls out to a server.
#[derive(Clone)]
pub struct Client {
    hyper_client: Arc<
        Fn(
                &Handle
            ) -> Box<
                hyper::client::Service<
                    Request = hyper::Request<hyper::Body>,
                    Response = hyper::Response,
                    Error = hyper::Error,
                    Future = hyper::client::FutureResponse,
                >,
            >
            + Sync
            + Send,
    >,
    handle: Arc<Handle>,
    base_path: String,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client {{ base_path: {} }}", self.base_path)
    }
}

impl Client {
    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `handle` - tokio reactor handle to use for execution
    /// * `base_path` - base path of the client API, i.e. "www.my-api-implementation.com"
    pub fn try_new_http(handle: Handle, base_path: &str) -> Result<Client, ClientInitError> {
        let http_connector = swagger::http_connector();
        Self::try_new_with_connector::<hyper::client::HttpConnector>(
            handle,
            base_path,
            Some("http"),
            http_connector,
        )
    }

    /// Create a client with a TLS connection to the server.
    ///
    /// # Arguments
    /// * `handle` - tokio reactor handle to use for execution
    /// * `base_path` - base path of the client API, i.e. "www.my-api-implementation.com"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    pub fn try_new_https<CA>(
        handle: Handle,
        base_path: &str,
        ca_certificate: CA,
    ) -> Result<Client, ClientInitError>
    where
        CA: AsRef<Path>,
    {
        let https_connector = swagger::https_connector(ca_certificate);
        Self::try_new_with_connector::<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>(
            handle,
            base_path,
            Some("https"),
            https_connector,
        )
    }

    /// Create a client with a mutually authenticated TLS connection to the server.
    ///
    /// # Arguments
    /// * `handle` - tokio reactor handle to use for execution
    /// * `base_path` - base path of the client API, i.e. "www.my-api-implementation.com"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    /// * `client_key` - Path to the client private key
    /// * `client_certificate` - Path to the client's public certificate associated with the private key
    pub fn try_new_https_mutual<CA, K, C, T>(
        handle: Handle,
        base_path: &str,
        ca_certificate: CA,
        client_key: K,
        client_certificate: C,
    ) -> Result<Client, ClientInitError>
    where
        CA: AsRef<Path>,
        K: AsRef<Path>,
        C: AsRef<Path>,
    {
        let https_connector =
            swagger::https_mutual_connector(ca_certificate, client_key, client_certificate);
        Self::try_new_with_connector::<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>(
            handle,
            base_path,
            Some("https"),
            https_connector,
        )
    }

    /// Create a client with a custom implementation of hyper::client::Connect.
    ///
    /// Intended for use with custom implementations of connect for e.g. protocol logging
    /// or similar functionality which requires wrapping the transport layer. When wrapping a TCP connection,
    /// this function should be used in conjunction with
    /// `swagger::{http_connector, https_connector, https_mutual_connector}`.
    ///
    /// For ordinary tcp connections, prefer the use of `try_new_http`, `try_new_https`
    /// and `try_new_https_mutual`, to avoid introducing a dependency on the underlying transport layer.
    ///
    /// # Arguments
    ///
    /// * `handle` - tokio reactor handle to use for execution
    /// * `base_path` - base path of the client API, i.e. "www.my-api-implementation.com"
    /// * `protocol` - Which protocol to use when constructing the request url, e.g. `Some("http")`
    /// * `connector_fn` - Function which returns an implementation of `hyper::client::Connect`
    pub fn try_new_with_connector<C>(
        handle: Handle,
        base_path: &str,
        protocol: Option<&'static str>,
        connector_fn: Box<Fn(&Handle) -> C + Send + Sync>,
    ) -> Result<Client, ClientInitError>
    where
        C: hyper::client::Connect + hyper::client::Service,
    {
        let hyper_client = {
            move |handle: &Handle| -> Box<
                hyper::client::Service<
                    Request = hyper::Request<hyper::Body>,
                    Response = hyper::Response,
                    Error = hyper::Error,
                    Future = hyper::client::FutureResponse,
                >,
            > {
                let connector = connector_fn(handle);
                Box::new(
                    hyper::Client::configure()
                        .connector(connector)
                        .build(handle),
                )
            }
        };

        Ok(Client {
            hyper_client: Arc::new(hyper_client),
            handle: Arc::new(handle),
            base_path: into_base_path(base_path, protocol)?,
        })
    }

    /// Constructor for creating a `Client` by passing in a pre-made `hyper` client.
    ///
    /// One should avoid relying on this function if possible, since it adds a dependency on the underlying transport
    /// implementation, which it would be better to abstract away. Therefore, using this function may lead to a loss of
    /// code generality, which may make it harder to move the application to a serverless environment, for example.
    ///
    /// The reason for this function's existence is to support legacy test code, which did mocking at the hyper layer.
    /// This is not a recommended way to write new tests. If other reasons are found for using this function, they
    /// should be mentioned here.
    pub fn try_new_with_hyper_client(
        hyper_client: Arc<
            Fn(
                    &Handle
                ) -> Box<
                    hyper::client::Service<
                        Request = hyper::Request<hyper::Body>,
                        Response = hyper::Response,
                        Error = hyper::Error,
                        Future = hyper::client::FutureResponse,
                    >,
                >
                + Sync
                + Send,
        >,
        handle: Handle,
        base_path: &str,
    ) -> Result<Client, ClientInitError> {
        Ok(Client {
            hyper_client: hyper_client,
            handle: Arc::new(handle),
            base_path: into_base_path(base_path, None)?,
        })
    }
}

impl Api for Client {
    fn container_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = ContainerIdGetResponse, Error = ApiError>> {
        let uri = format!(
            "{}/v0/container/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ContainerEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    ContainerIdGetResponse::FetchASingleContainerById(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ContainerIdGetResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    ContainerIdGetResponse::GenericErrorResponse(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn container_lookup_get(
        &self,
        param_issn: String,
        context: &Context,
    ) -> Box<Future<Item = ContainerLookupGetResponse, Error = ApiError>> {
        // Query parameters
        let query_issn = format!("issn={issn}&", issn = param_issn.to_string());

        let uri = format!(
            "{}/v0/container/lookup?{issn}",
            self.base_path,
            issn = utf8_percent_encode(&query_issn, QUERY_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ContainerEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    ContainerLookupGetResponse::FindASingleContainerByExternalIdentifer(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ContainerLookupGetResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    404 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ContainerLookupGetResponse::NoSuchContainer(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    ContainerLookupGetResponse::GenericErrorResponse(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn container_post(
        &self,
        param_body: Option<models::ContainerEntity>,
        context: &Context,
    ) -> Box<Future<Item = ContainerPostResponse, Error = ApiError>> {
        let uri = format!("{}/v0/container", self.base_path);

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Post, uri);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));

        if let Some(body) = body {
            request.set_body(body.into_bytes());
        }

        request
            .headers_mut()
            .set(ContentType(mimetypes::requests::CONTAINER_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    201 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::EntityEdit>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ContainerPostResponse::Created(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ContainerPostResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ContainerPostResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn creator_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = CreatorIdGetResponse, Error = ApiError>> {
        let uri = format!(
            "{}/v0/creator/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::CreatorEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    CreatorIdGetResponse::FetchASingleCreatorById(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| CreatorIdGetResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| CreatorIdGetResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn creator_lookup_get(
        &self,
        param_orcid: String,
        context: &Context,
    ) -> Box<Future<Item = CreatorLookupGetResponse, Error = ApiError>> {
        // Query parameters
        let query_orcid = format!("orcid={orcid}&", orcid = param_orcid.to_string());

        let uri = format!(
            "{}/v0/creator/lookup?{orcid}",
            self.base_path,
            orcid = utf8_percent_encode(&query_orcid, QUERY_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::CreatorEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    CreatorLookupGetResponse::FindASingleCreatorByExternalIdentifer(
                                        body,
                                    )
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| CreatorLookupGetResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    404 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| CreatorLookupGetResponse::NoSuchCreator(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    CreatorLookupGetResponse::GenericErrorResponse(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn creator_post(
        &self,
        param_body: Option<models::CreatorEntity>,
        context: &Context,
    ) -> Box<Future<Item = CreatorPostResponse, Error = ApiError>> {
        let uri = format!("{}/v0/creator", self.base_path);

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Post, uri);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));

        if let Some(body) = body {
            request.set_body(body.into_bytes());
        }

        request
            .headers_mut()
            .set(ContentType(mimetypes::requests::CREATOR_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    201 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::EntityEdit>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| CreatorPostResponse::Created(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| CreatorPostResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| CreatorPostResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn editgroup_id_accept_post(
        &self,
        param_id: i32,
        context: &Context,
    ) -> Box<Future<Item = EditgroupIdAcceptPostResponse, Error = ApiError>> {
        let uri = format!(
            "{}/v0/editgroup/{id}/accept",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Post, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Success>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditgroupIdAcceptPostResponse::MergedEditgroupSuccessfully_(
                                        body,
                                    )
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditgroupIdAcceptPostResponse::EditgroupIsInAnUnmergableState(
                                        body,
                                    )
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    404 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditgroupIdAcceptPostResponse::NoSuchEditgroup(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditgroupIdAcceptPostResponse::GenericErrorResponse(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn editgroup_id_get(
        &self,
        param_id: i32,
        context: &Context,
    ) -> Box<Future<Item = EditgroupIdGetResponse, Error = ApiError>> {
        let uri = format!(
            "{}/v0/editgroup/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Editgroup>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditgroupIdGetResponse::FetchEditgroupByIdentifier(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    404 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| EditgroupIdGetResponse::NoSuchEditgroup(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditgroupIdGetResponse::GenericErrorResponse(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn editgroup_post(
        &self,
        context: &Context,
    ) -> Box<Future<Item = EditgroupPostResponse, Error = ApiError>> {
        let uri = format!("{}/v0/editgroup", self.base_path);

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Post, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    201 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Editgroup>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| EditgroupPostResponse::SuccessfullyCreated(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditgroupPostResponse::InvalidRequestParameters(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| EditgroupPostResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn editor_username_changelog_get(
        &self,
        param_username: String,
        context: &Context,
    ) -> Box<Future<Item = EditorUsernameChangelogGetResponse, Error = ApiError>> {
        let uri = format!(
            "{}/v0/editor/{username}/changelog",
            self.base_path,
            username = utf8_percent_encode(&param_username.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Changelogentry>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditorUsernameChangelogGetResponse::FindChanges_(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    404 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditorUsernameChangelogGetResponse::UsernameNotFound(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditorUsernameChangelogGetResponse::GenericErrorResponse(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn editor_username_get(
        &self,
        param_username: String,
        context: &Context,
    ) -> Box<Future<Item = EditorUsernameGetResponse, Error = ApiError>> {
        let uri = format!(
            "{}/v0/editor/{username}",
            self.base_path,
            username = utf8_percent_encode(&param_username.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Editor>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditorUsernameGetResponse::FetchGenericInformationAboutAnEditor(
                                        body,
                                    )
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    404 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| EditorUsernameGetResponse::UsernameNotFound(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    EditorUsernameGetResponse::GenericErrorResponse(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn file_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = FileIdGetResponse, Error = ApiError>> {
        let uri = format!(
            "{}/v0/file/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::FileEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| FileIdGetResponse::FetchASingleFileById(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| FileIdGetResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| FileIdGetResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn file_lookup_get(
        &self,
        param_sha1: String,
        context: &Context,
    ) -> Box<Future<Item = FileLookupGetResponse, Error = ApiError>> {
        // Query parameters
        let query_sha1 = format!("sha1={sha1}&", sha1 = param_sha1.to_string());

        let uri = format!(
            "{}/v0/file/lookup?{sha1}",
            self.base_path,
            sha1 = utf8_percent_encode(&query_sha1, QUERY_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::FileEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    FileLookupGetResponse::FindASingleFileByExternalIdentifer(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| FileLookupGetResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    404 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| FileLookupGetResponse::NoSuchFile(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| FileLookupGetResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn file_post(
        &self,
        param_body: Option<models::FileEntity>,
        context: &Context,
    ) -> Box<Future<Item = FilePostResponse, Error = ApiError>> {
        let uri = format!("{}/v0/file", self.base_path);

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Post, uri);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));

        if let Some(body) = body {
            request.set_body(body.into_bytes());
        }

        request
            .headers_mut()
            .set(ContentType(mimetypes::requests::FILE_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    201 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::EntityEdit>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| FilePostResponse::Created(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| FilePostResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| FilePostResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn release_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = ReleaseIdGetResponse, Error = ApiError>> {
        let uri = format!(
            "{}/v0/release/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ReleaseEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    ReleaseIdGetResponse::FetchASingleReleaseById(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ReleaseIdGetResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ReleaseIdGetResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn release_lookup_get(
        &self,
        param_doi: String,
        context: &Context,
    ) -> Box<Future<Item = ReleaseLookupGetResponse, Error = ApiError>> {
        // Query parameters
        let query_doi = format!("doi={doi}&", doi = param_doi.to_string());

        let uri = format!(
            "{}/v0/release/lookup?{doi}",
            self.base_path,
            doi = utf8_percent_encode(&query_doi, QUERY_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ReleaseEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    ReleaseLookupGetResponse::FindASingleReleaseByExternalIdentifer(
                                        body,
                                    )
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ReleaseLookupGetResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    404 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ReleaseLookupGetResponse::NoSuchRelease(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    ReleaseLookupGetResponse::GenericErrorResponse(body)
                                }),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn release_post(
        &self,
        param_body: Option<models::ReleaseEntity>,
        context: &Context,
    ) -> Box<Future<Item = ReleasePostResponse, Error = ApiError>> {
        let uri = format!("{}/v0/release", self.base_path);

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Post, uri);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));

        if let Some(body) = body {
            request.set_body(body.into_bytes());
        }

        request
            .headers_mut()
            .set(ContentType(mimetypes::requests::RELEASE_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    201 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::EntityEdit>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ReleasePostResponse::Created(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ReleasePostResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| ReleasePostResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn work_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = WorkIdGetResponse, Error = ApiError>> {
        let uri = format!(
            "{}/v0/work/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Get, uri);

        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::WorkEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| WorkIdGetResponse::FetchASingleWorkById(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| WorkIdGetResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| WorkIdGetResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }

    fn work_post(
        &self,
        param_body: Option<models::WorkEntity>,
        context: &Context,
    ) -> Box<Future<Item = WorkPostResponse, Error = ApiError>> {
        let uri = format!("{}/v0/work", self.base_path);

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(futures::done(Err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                )))))
            }
        };

        let mut request = hyper::Request::new(hyper::Method::Post, uri);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));

        if let Some(body) = body {
            request.set_body(body.into_bytes());
        }

        request
            .headers_mut()
            .set(ContentType(mimetypes::requests::WORK_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| request.headers_mut().set(XSpanId(header.clone())));

        let hyper_client = (self.hyper_client)(&*self.handle);
        Box::new(
            hyper_client
                .call(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    201 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::EntityEdit>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| WorkPostResponse::Created(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    400 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| WorkPostResponse::BadRequest(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    0 => {
                        let body = response.body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::Error>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| WorkPostResponse::GenericErrorResponse(body)),
                        ) as Box<Future<Item = _, Error = _>>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) => {
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e))
                                        }
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<Future<Item = _, Error = _>>
                    }
                }),
        )
    }
}

#[derive(Debug)]
pub enum ClientInitError {
    InvalidScheme,
    InvalidUri(hyper::error::UriError),
    MissingHost,
    SslError(openssl::error::ErrorStack),
}

impl From<hyper::error::UriError> for ClientInitError {
    fn from(err: hyper::error::UriError) -> ClientInitError {
        ClientInitError::InvalidUri(err)
    }
}

impl From<openssl::error::ErrorStack> for ClientInitError {
    fn from(err: openssl::error::ErrorStack) -> ClientInitError {
        ClientInitError::SslError(err)
    }
}

impl fmt::Display for ClientInitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self as &fmt::Debug).fmt(f)
    }
}

impl error::Error for ClientInitError {
    fn description(&self) -> &str {
        "Failed to produce a hyper client."
    }
}
