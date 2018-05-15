#![allow(unused_extern_crates)]
extern crate chrono;
extern crate hyper_openssl;
extern crate url;

use self::hyper_openssl::openssl;
use self::url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET, QUERY_ENCODE_SET};
use futures;
use futures::{Future, Stream};
use futures::{future, stream};
use hyper;
use hyper::Url;
use hyper::client::IntoUrl;
use hyper::header::{ContentType, Headers};
use hyper::mime;
use hyper::mime::{Attr, Mime, SubLevel, TopLevel, Value};
use std::borrow::Cow;
use std::error;
use std::fmt;
use std::io::{Error, Read};
use std::path::Path;
use std::str;
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
fn into_base_path<T: IntoUrl>(
    input: T,
    correct_scheme: Option<&'static str>,
) -> Result<String, ClientInitError> {
    // First convert to Url, since a base path is a subset of Url.
    let url = input.into_url()?;

    let scheme = url.scheme();

    // Check the scheme if necessary
    if let Some(correct_scheme) = correct_scheme {
        if scheme != correct_scheme {
            return Err(ClientInitError::InvalidScheme);
        }
    }

    let host = url.host().ok_or_else(|| ClientInitError::MissingHost)?;
    let port = url.port().map(|x| format!(":{}", x)).unwrap_or_default();
    Ok(format!("{}://{}{}", scheme, host, port))
}

/// A client that implements the API by making HTTP calls out to a server.
#[derive(Clone)]
pub struct Client {
    base_path: String,
    hyper_client: Arc<Fn() -> hyper::client::Client + Sync + Send>,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client {{ base_path: {} }}", self.base_path)
    }
}

impl Client {
    pub fn try_new_http<T>(base_path: T) -> Result<Client, ClientInitError>
    where
        T: IntoUrl,
    {
        Ok(Client {
            base_path: into_base_path(base_path, Some("http"))?,
            hyper_client: Arc::new(hyper::client::Client::new),
        })
    }

    pub fn try_new_https<T, CA>(base_path: T, ca_certificate: CA) -> Result<Client, ClientInitError>
    where
        T: IntoUrl,
        CA: AsRef<Path>,
    {
        let ca_certificate = ca_certificate.as_ref().to_owned();

        let https_hyper_client = move || {
            // SSL implementation
            let mut ssl =
                openssl::ssl::SslConnectorBuilder::new(openssl::ssl::SslMethod::tls()).unwrap();

            // Server authentication
            ssl.set_ca_file(ca_certificate.clone()).unwrap();

            let ssl = hyper_openssl::OpensslClient::from(ssl.build());
            let connector = hyper::net::HttpsConnector::new(ssl);
            hyper::client::Client::with_connector(connector)
        };

        Ok(Client {
            base_path: into_base_path(base_path, Some("https"))?,
            hyper_client: Arc::new(https_hyper_client),
        })
    }

    pub fn try_new_https_mutual<T, CA, K, C>(
        base_path: T,
        ca_certificate: CA,
        client_key: K,
        client_certificate: C,
    ) -> Result<Client, ClientInitError>
    where
        T: IntoUrl,
        CA: AsRef<Path>,
        K: AsRef<Path>,
        C: AsRef<Path>,
    {
        let ca_certificate = ca_certificate.as_ref().to_owned();
        let client_key = client_key.as_ref().to_owned();
        let client_certificate = client_certificate.as_ref().to_owned();

        let https_mutual_hyper_client = move || {
            // SSL implementation
            let mut ssl =
                openssl::ssl::SslConnectorBuilder::new(openssl::ssl::SslMethod::tls()).unwrap();

            // Server authentication
            ssl.set_ca_file(ca_certificate.clone()).unwrap();

            // Client authentication
            ssl.set_private_key_file(client_key.clone(), openssl::x509::X509_FILETYPE_PEM)
                .unwrap();
            ssl.set_certificate_chain_file(client_certificate.clone())
                .unwrap();
            ssl.check_private_key().unwrap();

            let ssl = hyper_openssl::OpensslClient::from(ssl.build());
            let connector = hyper::net::HttpsConnector::new(ssl);
            hyper::client::Client::with_connector(connector)
        };

        Ok(Client {
            base_path: into_base_path(base_path, Some("https"))?,
            hyper_client: Arc::new(https_mutual_hyper_client),
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
    pub fn try_new_with_hyper_client<T>(
        base_path: T,
        hyper_client: Arc<Fn() -> hyper::client::Client + Sync + Send>,
    ) -> Result<Client, ClientInitError>
    where
        T: IntoUrl,
    {
        Ok(Client {
            base_path: into_base_path(base_path, None)?,
            hyper_client: hyper_client,
        })
    }
}

impl Api for Client {
    fn container_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = ContainerIdGetResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/container/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<ContainerIdGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ContainerEntity>(&buf)?;

                    Ok(ContainerIdGetResponse::FetchASingleContainerById(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ContainerIdGetResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ContainerIdGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn container_lookup_get(
        &self,
        param_issn: String,
        context: &Context,
    ) -> Box<Future<Item = ContainerLookupGetResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_issn = format!("issn={issn}&", issn = param_issn.to_string());

        let url = format!(
            "{}/v0/container/lookup?{issn}",
            self.base_path,
            issn = utf8_percent_encode(&query_issn, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<ContainerLookupGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ContainerEntity>(&buf)?;

                    Ok(ContainerLookupGetResponse::FindASingleContainerByExternalIdentifer(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ContainerLookupGetResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ContainerLookupGetResponse::NoSuchContainer(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ContainerLookupGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn container_post(
        &self,
        param_body: Option<models::ContainerEntity>,
        context: &Context,
    ) -> Box<Future<Item = ContainerPostResponse, Error = ApiError> + Send> {
        let url = format!("{}/v0/container", self.base_path);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = match body {
            Some(ref body) => request.body(body),
            None => request,
        };

        custom_headers.set(ContentType(mimetypes::requests::CONTAINER_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<ContainerPostResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(ContainerPostResponse::Created(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ContainerPostResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ContainerPostResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn creator_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = CreatorIdGetResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/creator/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<CreatorIdGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::CreatorEntity>(&buf)?;

                    Ok(CreatorIdGetResponse::FetchASingleCreatorById(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(CreatorIdGetResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(CreatorIdGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn creator_lookup_get(
        &self,
        param_orcid: String,
        context: &Context,
    ) -> Box<Future<Item = CreatorLookupGetResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_orcid = format!("orcid={orcid}&", orcid = param_orcid.to_string());

        let url = format!(
            "{}/v0/creator/lookup?{orcid}",
            self.base_path,
            orcid = utf8_percent_encode(&query_orcid, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<CreatorLookupGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::CreatorEntity>(&buf)?;

                    Ok(CreatorLookupGetResponse::FindASingleCreatorByExternalIdentifer(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(CreatorLookupGetResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(CreatorLookupGetResponse::NoSuchCreator(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(CreatorLookupGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn creator_post(
        &self,
        param_body: Option<models::CreatorEntity>,
        context: &Context,
    ) -> Box<Future<Item = CreatorPostResponse, Error = ApiError> + Send> {
        let url = format!("{}/v0/creator", self.base_path);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = match body {
            Some(ref body) => request.body(body),
            None => request,
        };

        custom_headers.set(ContentType(mimetypes::requests::CREATOR_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<CreatorPostResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(CreatorPostResponse::Created(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(CreatorPostResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(CreatorPostResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn editgroup_id_accept_post(
        &self,
        param_id: i32,
        context: &Context,
    ) -> Box<Future<Item = EditgroupIdAcceptPostResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/editgroup/{id}/accept",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<EditgroupIdAcceptPostResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Success>(&buf)?;

                    Ok(EditgroupIdAcceptPostResponse::MergedEditgroupSuccessfully_(
                        body,
                    ))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditgroupIdAcceptPostResponse::EditgroupIsInAnUnmergableState(body))
                }
                404 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditgroupIdAcceptPostResponse::NoSuchEditgroup(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditgroupIdAcceptPostResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn editgroup_id_get(
        &self,
        param_id: i32,
        context: &Context,
    ) -> Box<Future<Item = EditgroupIdGetResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/editgroup/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<EditgroupIdGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Editgroup>(&buf)?;

                    Ok(EditgroupIdGetResponse::FetchEditgroupByIdentifier(body))
                }
                404 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditgroupIdGetResponse::NoSuchEditgroup(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditgroupIdGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn editgroup_post(
        &self,
        context: &Context,
    ) -> Box<Future<Item = EditgroupPostResponse, Error = ApiError> + Send> {
        let url = format!("{}/v0/editgroup", self.base_path);

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<EditgroupPostResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Editgroup>(&buf)?;

                    Ok(EditgroupPostResponse::SuccessfullyCreated(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditgroupPostResponse::InvalidRequestParameters(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditgroupPostResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn editor_username_changelog_get(
        &self,
        param_username: String,
        context: &Context,
    ) -> Box<Future<Item = EditorUsernameChangelogGetResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/editor/{username}/changelog",
            self.base_path,
            username = utf8_percent_encode(&param_username.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<EditorUsernameChangelogGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Changelogentry>(&buf)?;

                    Ok(EditorUsernameChangelogGetResponse::FindChanges_(body))
                }
                404 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditorUsernameChangelogGetResponse::UsernameNotFound(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditorUsernameChangelogGetResponse::GenericErrorResponse(
                        body,
                    ))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn editor_username_get(
        &self,
        param_username: String,
        context: &Context,
    ) -> Box<Future<Item = EditorUsernameGetResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/editor/{username}",
            self.base_path,
            username = utf8_percent_encode(&param_username.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<EditorUsernameGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Editor>(&buf)?;

                    Ok(EditorUsernameGetResponse::FetchGenericInformationAboutAnEditor(body))
                }
                404 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditorUsernameGetResponse::UsernameNotFound(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(EditorUsernameGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn file_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = FileIdGetResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/file/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<FileIdGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::FileEntity>(&buf)?;

                    Ok(FileIdGetResponse::FetchASingleFileById(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(FileIdGetResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(FileIdGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn file_lookup_get(
        &self,
        param_sha1: String,
        context: &Context,
    ) -> Box<Future<Item = FileLookupGetResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_sha1 = format!("sha1={sha1}&", sha1 = param_sha1.to_string());

        let url = format!(
            "{}/v0/file/lookup?{sha1}",
            self.base_path,
            sha1 = utf8_percent_encode(&query_sha1, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<FileLookupGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::FileEntity>(&buf)?;

                    Ok(FileLookupGetResponse::FindASingleFileByExternalIdentifer(
                        body,
                    ))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(FileLookupGetResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(FileLookupGetResponse::NoSuchFile(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(FileLookupGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn file_post(
        &self,
        param_body: Option<models::FileEntity>,
        context: &Context,
    ) -> Box<Future<Item = FilePostResponse, Error = ApiError> + Send> {
        let url = format!("{}/v0/file", self.base_path);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = match body {
            Some(ref body) => request.body(body),
            None => request,
        };

        custom_headers.set(ContentType(mimetypes::requests::FILE_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<FilePostResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(FilePostResponse::Created(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(FilePostResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(FilePostResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn release_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = ReleaseIdGetResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/release/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<ReleaseIdGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ReleaseEntity>(&buf)?;

                    Ok(ReleaseIdGetResponse::FetchASingleReleaseById(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ReleaseIdGetResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ReleaseIdGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn release_lookup_get(
        &self,
        param_doi: String,
        context: &Context,
    ) -> Box<Future<Item = ReleaseLookupGetResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_doi = format!("doi={doi}&", doi = param_doi.to_string());

        let url = format!(
            "{}/v0/release/lookup?{doi}",
            self.base_path,
            doi = utf8_percent_encode(&query_doi, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<ReleaseLookupGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ReleaseEntity>(&buf)?;

                    Ok(ReleaseLookupGetResponse::FindASingleReleaseByExternalIdentifer(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ReleaseLookupGetResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ReleaseLookupGetResponse::NoSuchRelease(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ReleaseLookupGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn release_post(
        &self,
        param_body: Option<models::ReleaseEntity>,
        context: &Context,
    ) -> Box<Future<Item = ReleasePostResponse, Error = ApiError> + Send> {
        let url = format!("{}/v0/release", self.base_path);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = match body {
            Some(ref body) => request.body(body),
            None => request,
        };

        custom_headers.set(ContentType(mimetypes::requests::RELEASE_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<ReleasePostResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(ReleasePostResponse::Created(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ReleasePostResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(ReleasePostResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn work_id_get(
        &self,
        param_id: String,
        context: &Context,
    ) -> Box<Future<Item = WorkIdGetResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/work/{id}",
            self.base_path,
            id = utf8_percent_encode(&param_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<WorkIdGetResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::WorkEntity>(&buf)?;

                    Ok(WorkIdGetResponse::FetchASingleWorkById(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(WorkIdGetResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(WorkIdGetResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn work_post(
        &self,
        param_body: Option<models::WorkEntity>,
        context: &Context,
    ) -> Box<Future<Item = WorkPostResponse, Error = ApiError> + Send> {
        let url = format!("{}/v0/work", self.base_path);

        let body = param_body
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));
        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = match body {
            Some(ref body) => request.body(body),
            None => request,
        };

        custom_headers.set(ContentType(mimetypes::requests::WORK_POST.clone()));
        context
            .x_span_id
            .as_ref()
            .map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(
            mut response: hyper::client::response::Response,
        ) -> Result<WorkPostResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(WorkPostResponse::Created(body))
                }
                400 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(WorkPostResponse::BadRequest(body))
                }
                0 => {
                    let mut buf = String::new();
                    response
                        .read_to_string(&mut buf)
                        .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Error>(&buf)?;

                    Ok(WorkPostResponse::GenericErrorResponse(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!(
                                "<Body was not UTF8: {:?}>",
                                &buf[..len].to_vec()
                            )),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!(
                        "Unexpected response code {}:\n{:?}\n\n{}",
                        code, response.headers, debug_body
                    )))
                }
            }
        }

        let result = request
            .send()
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .and_then(parse_response);
        Box::new(futures::done(result))
    }
}

#[derive(Debug)]
pub enum ClientInitError {
    InvalidScheme,
    InvalidUrl(hyper::error::ParseError),
    MissingHost,
    SslError(openssl::error::ErrorStack),
}

impl From<hyper::error::ParseError> for ClientInitError {
    fn from(err: hyper::error::ParseError) -> ClientInitError {
        ClientInitError::InvalidUrl(err)
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
