#![allow(unused_extern_crates)]
extern crate chrono;
extern crate hyper_openssl;
extern crate url;

use self::hyper_openssl::openssl;
use self::url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET, QUERY_ENCODE_SET};
use futures;
use futures::{future, stream};
use futures::{Future, Stream};
use hyper;
use hyper::client::IntoUrl;
use hyper::header::{ContentType, Headers};
use hyper::mime;
use hyper::mime::{Attr, Mime, SubLevel, TopLevel, Value};
use hyper::Url;
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
use {
    AcceptEditgroupResponse, Api, CreateContainerBatchResponse, CreateContainerResponse, CreateCreatorBatchResponse, CreateCreatorResponse, CreateEditgroupResponse, CreateFileBatchResponse,
    CreateFileResponse, CreateReleaseBatchResponse, CreateReleaseResponse, CreateWorkBatchResponse, CreateWorkResponse, DeleteContainerEditResponse, DeleteContainerResponse,
    DeleteCreatorEditResponse, DeleteCreatorResponse, DeleteFileEditResponse, DeleteFileResponse, DeleteReleaseEditResponse, DeleteReleaseResponse, DeleteWorkEditResponse, DeleteWorkResponse,
    GetChangelogEntryResponse, GetChangelogResponse, GetContainerEditResponse, GetContainerHistoryResponse, GetContainerRedirectsResponse, GetContainerResponse, GetContainerRevisionResponse,
    GetCreatorEditResponse, GetCreatorHistoryResponse, GetCreatorRedirectsResponse, GetCreatorReleasesResponse, GetCreatorResponse, GetCreatorRevisionResponse, GetEditgroupResponse,
    GetEditorChangelogResponse, GetEditorResponse, GetFileEditResponse, GetFileHistoryResponse, GetFileRedirectsResponse, GetFileResponse, GetFileRevisionResponse, GetReleaseEditResponse,
    GetReleaseFilesResponse, GetReleaseHistoryResponse, GetReleaseRedirectsResponse, GetReleaseResponse, GetReleaseRevisionResponse, GetStatsResponse, GetWorkEditResponse, GetWorkHistoryResponse,
    GetWorkRedirectsResponse, GetWorkReleasesResponse, GetWorkResponse, GetWorkRevisionResponse, LookupContainerResponse, LookupCreatorResponse, LookupFileResponse, LookupReleaseResponse,
    UpdateContainerResponse, UpdateCreatorResponse, UpdateFileResponse, UpdateReleaseResponse, UpdateWorkResponse,
};

/// Convert input into a base path, e.g. "http://example:123". Also checks the scheme as it goes.
fn into_base_path<T: IntoUrl>(input: T, correct_scheme: Option<&'static str>) -> Result<String, ClientInitError> {
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
            let mut ssl = openssl::ssl::SslConnectorBuilder::new(openssl::ssl::SslMethod::tls()).unwrap();

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

    pub fn try_new_https_mutual<T, CA, K, C>(base_path: T, ca_certificate: CA, client_key: K, client_certificate: C) -> Result<Client, ClientInitError>
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
            let mut ssl = openssl::ssl::SslConnectorBuilder::new(openssl::ssl::SslMethod::tls()).unwrap();

            // Server authentication
            ssl.set_ca_file(ca_certificate.clone()).unwrap();

            // Client authentication
            ssl.set_private_key_file(client_key.clone(), openssl::x509::X509_FILETYPE_PEM).unwrap();
            ssl.set_certificate_chain_file(client_certificate.clone()).unwrap();
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
    pub fn try_new_with_hyper_client<T>(base_path: T, hyper_client: Arc<Fn() -> hyper::client::Client + Sync + Send>) -> Result<Client, ClientInitError>
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
    fn create_container(&self, param_entity: models::ContainerEntity, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateContainerResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/container?{editgroup_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_CONTAINER.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateContainerResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(CreateContainerResponse::CreatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateContainerResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateContainerResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateContainerResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_container_batch(
        &self,
        param_entity_list: &Vec<models::ContainerEntity>,
        param_autoaccept: Option<bool>,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateContainerBatchResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_autoaccept = param_autoaccept.map_or_else(String::new, |query| format!("autoaccept={autoaccept}&", autoaccept = query.to_string()));
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/container/batch?{autoaccept}{editgroup_id}",
            self.base_path,
            autoaccept = utf8_percent_encode(&query_autoaccept, QUERY_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity_list).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_CONTAINER_BATCH.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateContainerBatchResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityEdit>>(&buf)?;

                    Ok(CreateContainerBatchResponse::CreatedEntities(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateContainerBatchResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateContainerBatchResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateContainerBatchResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_container(&self, param_ident: String, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteContainerResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/container/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteContainerResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(DeleteContainerResponse::DeletedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteContainerResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteContainerResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteContainerResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_container_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = DeleteContainerEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/container/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteContainerEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Success>(&buf)?;

                    Ok(DeleteContainerEditResponse::DeletedEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteContainerEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteContainerEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteContainerEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_container(&self, param_ident: String, param_expand: Option<String>, param_hide: Option<String>, context: &Context) -> Box<Future<Item = GetContainerResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/container/{ident}?{expand}{hide}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetContainerResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ContainerEntity>(&buf)?;

                    Ok(GetContainerResponse::FoundEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_container_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = GetContainerEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/container/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetContainerEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(GetContainerEditResponse::FoundEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_container_history(&self, param_ident: String, param_limit: Option<i64>, context: &Context) -> Box<Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_limit = param_limit.map_or_else(String::new, |query| format!("limit={limit}&", limit = query.to_string()));

        let url = format!(
            "{}/v0/container/{ident}/history?{limit}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            limit = utf8_percent_encode(&query_limit, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetContainerHistoryResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityHistoryEntry>>(&buf)?;

                    Ok(GetContainerHistoryResponse::FoundEntityHistory(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerHistoryResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerHistoryResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerHistoryResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_container_redirects(&self, param_ident: String, context: &Context) -> Box<Future<Item = GetContainerRedirectsResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/container/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetContainerRedirectsResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<String>>(&buf)?;

                    Ok(GetContainerRedirectsResponse::FoundEntityRedirects(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerRedirectsResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerRedirectsResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerRedirectsResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_container_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = GetContainerRevisionResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/container/rev/{rev_id}?{expand}{hide}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetContainerRevisionResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ContainerEntity>(&buf)?;

                    Ok(GetContainerRevisionResponse::FoundEntityRevision(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerRevisionResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerRevisionResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetContainerRevisionResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn lookup_container(
        &self,
        param_issnl: Option<String>,
        param_wikidata_qid: Option<String>,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupContainerResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_issnl = param_issnl.map_or_else(String::new, |query| format!("issnl={issnl}&", issnl = query.to_string()));
        let query_wikidata_qid = param_wikidata_qid.map_or_else(String::new, |query| format!("wikidata_qid={wikidata_qid}&", wikidata_qid = query.to_string()));
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/container/lookup?{issnl}{wikidata_qid}{expand}{hide}",
            self.base_path,
            issnl = utf8_percent_encode(&query_issnl, QUERY_ENCODE_SET),
            wikidata_qid = utf8_percent_encode(&query_wikidata_qid, QUERY_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<LookupContainerResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ContainerEntity>(&buf)?;

                    Ok(LookupContainerResponse::FoundEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupContainerResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupContainerResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupContainerResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn update_container(
        &self,
        param_ident: String,
        param_entity: models::ContainerEntity,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = UpdateContainerResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/container/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Put, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::UPDATE_CONTAINER.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<UpdateContainerResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(UpdateContainerResponse::UpdatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateContainerResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateContainerResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateContainerResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_creator(&self, param_entity: models::CreatorEntity, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateCreatorResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/creator?{editgroup_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_CREATOR.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateCreatorResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(CreateCreatorResponse::CreatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateCreatorResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateCreatorResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateCreatorResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_creator_batch(
        &self,
        param_entity_list: &Vec<models::CreatorEntity>,
        param_autoaccept: Option<bool>,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateCreatorBatchResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_autoaccept = param_autoaccept.map_or_else(String::new, |query| format!("autoaccept={autoaccept}&", autoaccept = query.to_string()));
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/creator/batch?{autoaccept}{editgroup_id}",
            self.base_path,
            autoaccept = utf8_percent_encode(&query_autoaccept, QUERY_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity_list).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_CREATOR_BATCH.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateCreatorBatchResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityEdit>>(&buf)?;

                    Ok(CreateCreatorBatchResponse::CreatedEntities(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateCreatorBatchResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateCreatorBatchResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateCreatorBatchResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_creator(&self, param_ident: String, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteCreatorResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/creator/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteCreatorResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(DeleteCreatorResponse::DeletedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteCreatorResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteCreatorResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteCreatorResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_creator_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = DeleteCreatorEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/creator/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteCreatorEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Success>(&buf)?;

                    Ok(DeleteCreatorEditResponse::DeletedEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteCreatorEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteCreatorEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteCreatorEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_creator(&self, param_ident: String, param_expand: Option<String>, param_hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/creator/{ident}?{expand}{hide}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetCreatorResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::CreatorEntity>(&buf)?;

                    Ok(GetCreatorResponse::FoundEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_creator_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = GetCreatorEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/creator/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetCreatorEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(GetCreatorEditResponse::FoundEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_creator_history(&self, param_ident: String, param_limit: Option<i64>, context: &Context) -> Box<Future<Item = GetCreatorHistoryResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_limit = param_limit.map_or_else(String::new, |query| format!("limit={limit}&", limit = query.to_string()));

        let url = format!(
            "{}/v0/creator/{ident}/history?{limit}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            limit = utf8_percent_encode(&query_limit, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetCreatorHistoryResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityHistoryEntry>>(&buf)?;

                    Ok(GetCreatorHistoryResponse::FoundEntityHistory(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorHistoryResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorHistoryResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorHistoryResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_creator_redirects(&self, param_ident: String, context: &Context) -> Box<Future<Item = GetCreatorRedirectsResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/creator/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetCreatorRedirectsResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<String>>(&buf)?;

                    Ok(GetCreatorRedirectsResponse::FoundEntityRedirects(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorRedirectsResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorRedirectsResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorRedirectsResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_creator_releases(&self, param_ident: String, param_hide: Option<String>, context: &Context) -> Box<Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/creator/{ident}/releases?{hide}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetCreatorReleasesResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::ReleaseEntity>>(&buf)?;

                    Ok(GetCreatorReleasesResponse::Found(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorReleasesResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorReleasesResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorReleasesResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_creator_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = GetCreatorRevisionResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/creator/rev/{rev_id}?{expand}{hide}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetCreatorRevisionResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::CreatorEntity>(&buf)?;

                    Ok(GetCreatorRevisionResponse::FoundEntityRevision(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorRevisionResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorRevisionResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetCreatorRevisionResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn lookup_creator(
        &self,
        param_orcid: Option<String>,
        param_wikidata_qid: Option<String>,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupCreatorResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_orcid = param_orcid.map_or_else(String::new, |query| format!("orcid={orcid}&", orcid = query.to_string()));
        let query_wikidata_qid = param_wikidata_qid.map_or_else(String::new, |query| format!("wikidata_qid={wikidata_qid}&", wikidata_qid = query.to_string()));
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/creator/lookup?{orcid}{wikidata_qid}{expand}{hide}",
            self.base_path,
            orcid = utf8_percent_encode(&query_orcid, QUERY_ENCODE_SET),
            wikidata_qid = utf8_percent_encode(&query_wikidata_qid, QUERY_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<LookupCreatorResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::CreatorEntity>(&buf)?;

                    Ok(LookupCreatorResponse::FoundEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupCreatorResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupCreatorResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupCreatorResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn update_creator(
        &self,
        param_ident: String,
        param_entity: models::CreatorEntity,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = UpdateCreatorResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/creator/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Put, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::UPDATE_CREATOR.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<UpdateCreatorResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(UpdateCreatorResponse::UpdatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateCreatorResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateCreatorResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateCreatorResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_editor(&self, param_editor_id: String, context: &Context) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/editor/{editor_id}",
            self.base_path,
            editor_id = utf8_percent_encode(&param_editor_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetEditorResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Editor>(&buf)?;

                    Ok(GetEditorResponse::Found(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetEditorResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetEditorResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetEditorResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_editor_changelog(&self, param_editor_id: String, context: &Context) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/editor/{editor_id}/changelog",
            self.base_path,
            editor_id = utf8_percent_encode(&param_editor_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetEditorChangelogResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::ChangelogEntry>>(&buf)?;

                    Ok(GetEditorChangelogResponse::Found(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetEditorChangelogResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetEditorChangelogResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetEditorChangelogResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_stats(&self, param_more: Option<String>, context: &Context) -> Box<Future<Item = GetStatsResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_more = param_more.map_or_else(String::new, |query| format!("more={more}&", more = query.to_string()));

        let url = format!("{}/v0/stats?{more}", self.base_path, more = utf8_percent_encode(&query_more, QUERY_ENCODE_SET));

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetStatsResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::StatsResponse>(&buf)?;

                    Ok(GetStatsResponse::Success(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetStatsResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn accept_editgroup(&self, param_editgroup_id: String, context: &Context) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/editgroup/{editgroup_id}/accept",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<AcceptEditgroupResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Success>(&buf)?;

                    Ok(AcceptEditgroupResponse::MergedSuccessfully(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(AcceptEditgroupResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(AcceptEditgroupResponse::NotFound(body))
                }
                409 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(AcceptEditgroupResponse::EditConflict(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(AcceptEditgroupResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_editgroup(&self, param_editgroup: models::Editgroup, context: &Context) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send> {
        let url = format!("{}/v0/editgroup", self.base_path);

        let body = serde_json::to_string(&param_editgroup).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_EDITGROUP.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateEditgroupResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Editgroup>(&buf)?;

                    Ok(CreateEditgroupResponse::SuccessfullyCreated(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateEditgroupResponse::BadRequest(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateEditgroupResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_changelog(&self, param_limit: Option<i64>, context: &Context) -> Box<Future<Item = GetChangelogResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_limit = param_limit.map_or_else(String::new, |query| format!("limit={limit}&", limit = query.to_string()));

        let url = format!("{}/v0/changelog?{limit}", self.base_path, limit = utf8_percent_encode(&query_limit, QUERY_ENCODE_SET));

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetChangelogResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::ChangelogEntry>>(&buf)?;

                    Ok(GetChangelogResponse::Success(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetChangelogResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_changelog_entry(&self, param_index: i64, context: &Context) -> Box<Future<Item = GetChangelogEntryResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/changelog/{index}",
            self.base_path,
            index = utf8_percent_encode(&param_index.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetChangelogEntryResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ChangelogEntry>(&buf)?;

                    Ok(GetChangelogEntryResponse::FoundChangelogEntry(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetChangelogEntryResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetChangelogEntryResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_editgroup(&self, param_editgroup_id: String, context: &Context) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/editgroup/{editgroup_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetEditgroupResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Editgroup>(&buf)?;

                    Ok(GetEditgroupResponse::Found(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetEditgroupResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetEditgroupResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetEditgroupResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_file(&self, param_entity: models::FileEntity, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateFileResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!("{}/v0/file?{editgroup_id}", self.base_path, editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET));

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_FILE.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateFileResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(CreateFileResponse::CreatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateFileResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateFileResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateFileResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_file_batch(
        &self,
        param_entity_list: &Vec<models::FileEntity>,
        param_autoaccept: Option<bool>,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateFileBatchResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_autoaccept = param_autoaccept.map_or_else(String::new, |query| format!("autoaccept={autoaccept}&", autoaccept = query.to_string()));
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/file/batch?{autoaccept}{editgroup_id}",
            self.base_path,
            autoaccept = utf8_percent_encode(&query_autoaccept, QUERY_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity_list).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_FILE_BATCH.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateFileBatchResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityEdit>>(&buf)?;

                    Ok(CreateFileBatchResponse::CreatedEntities(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateFileBatchResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateFileBatchResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateFileBatchResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_file(&self, param_ident: String, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteFileResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/file/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteFileResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(DeleteFileResponse::DeletedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteFileResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteFileResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteFileResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_file_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = DeleteFileEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/file/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteFileEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Success>(&buf)?;

                    Ok(DeleteFileEditResponse::DeletedEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteFileEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteFileEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteFileEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_file(&self, param_ident: String, param_expand: Option<String>, param_hide: Option<String>, context: &Context) -> Box<Future<Item = GetFileResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/file/{ident}?{expand}{hide}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetFileResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::FileEntity>(&buf)?;

                    Ok(GetFileResponse::FoundEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_file_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = GetFileEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/file/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetFileEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(GetFileEditResponse::FoundEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_file_history(&self, param_ident: String, param_limit: Option<i64>, context: &Context) -> Box<Future<Item = GetFileHistoryResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_limit = param_limit.map_or_else(String::new, |query| format!("limit={limit}&", limit = query.to_string()));

        let url = format!(
            "{}/v0/file/{ident}/history?{limit}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            limit = utf8_percent_encode(&query_limit, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetFileHistoryResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityHistoryEntry>>(&buf)?;

                    Ok(GetFileHistoryResponse::FoundEntityHistory(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileHistoryResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileHistoryResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileHistoryResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_file_redirects(&self, param_ident: String, context: &Context) -> Box<Future<Item = GetFileRedirectsResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/file/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetFileRedirectsResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<String>>(&buf)?;

                    Ok(GetFileRedirectsResponse::FoundEntityRedirects(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileRedirectsResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileRedirectsResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileRedirectsResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_file_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = GetFileRevisionResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/file/rev/{rev_id}?{expand}{hide}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetFileRevisionResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::FileEntity>(&buf)?;

                    Ok(GetFileRevisionResponse::FoundEntityRevision(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileRevisionResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileRevisionResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetFileRevisionResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn lookup_file(
        &self,
        param_md5: Option<String>,
        param_sha1: Option<String>,
        param_sha256: Option<String>,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupFileResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_md5 = param_md5.map_or_else(String::new, |query| format!("md5={md5}&", md5 = query.to_string()));
        let query_sha1 = param_sha1.map_or_else(String::new, |query| format!("sha1={sha1}&", sha1 = query.to_string()));
        let query_sha256 = param_sha256.map_or_else(String::new, |query| format!("sha256={sha256}&", sha256 = query.to_string()));
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/file/lookup?{md5}{sha1}{sha256}{expand}{hide}",
            self.base_path,
            md5 = utf8_percent_encode(&query_md5, QUERY_ENCODE_SET),
            sha1 = utf8_percent_encode(&query_sha1, QUERY_ENCODE_SET),
            sha256 = utf8_percent_encode(&query_sha256, QUERY_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<LookupFileResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::FileEntity>(&buf)?;

                    Ok(LookupFileResponse::FoundEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupFileResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupFileResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupFileResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn update_file(
        &self,
        param_ident: String,
        param_entity: models::FileEntity,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = UpdateFileResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/file/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Put, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::UPDATE_FILE.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<UpdateFileResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(UpdateFileResponse::UpdatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateFileResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateFileResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateFileResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_release(&self, param_entity: models::ReleaseEntity, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateReleaseResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/release?{editgroup_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_RELEASE.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateReleaseResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(CreateReleaseResponse::CreatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateReleaseResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateReleaseResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateReleaseResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_release_batch(
        &self,
        param_entity_list: &Vec<models::ReleaseEntity>,
        param_autoaccept: Option<bool>,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateReleaseBatchResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_autoaccept = param_autoaccept.map_or_else(String::new, |query| format!("autoaccept={autoaccept}&", autoaccept = query.to_string()));
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/release/batch?{autoaccept}{editgroup_id}",
            self.base_path,
            autoaccept = utf8_percent_encode(&query_autoaccept, QUERY_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity_list).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_RELEASE_BATCH.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateReleaseBatchResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityEdit>>(&buf)?;

                    Ok(CreateReleaseBatchResponse::CreatedEntities(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateReleaseBatchResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateReleaseBatchResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateReleaseBatchResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_work(&self, param_entity: models::WorkEntity, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = CreateWorkResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!("{}/v0/work?{editgroup_id}", self.base_path, editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET));

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_WORK.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateWorkResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(CreateWorkResponse::CreatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateWorkResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateWorkResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateWorkResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_release(&self, param_ident: String, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteReleaseResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/release/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteReleaseResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(DeleteReleaseResponse::DeletedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteReleaseResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteReleaseResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteReleaseResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_release_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = DeleteReleaseEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/release/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteReleaseEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Success>(&buf)?;

                    Ok(DeleteReleaseEditResponse::DeletedEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteReleaseEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteReleaseEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteReleaseEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_release(&self, param_ident: String, param_expand: Option<String>, param_hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/release/{ident}?{expand}{hide}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetReleaseResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ReleaseEntity>(&buf)?;

                    Ok(GetReleaseResponse::FoundEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_release_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = GetReleaseEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/release/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetReleaseEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(GetReleaseEditResponse::FoundEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_release_files(&self, param_ident: String, param_hide: Option<String>, context: &Context) -> Box<Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/release/{ident}/files?{hide}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetReleaseFilesResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::FileEntity>>(&buf)?;

                    Ok(GetReleaseFilesResponse::Found(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseFilesResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseFilesResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseFilesResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_release_history(&self, param_ident: String, param_limit: Option<i64>, context: &Context) -> Box<Future<Item = GetReleaseHistoryResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_limit = param_limit.map_or_else(String::new, |query| format!("limit={limit}&", limit = query.to_string()));

        let url = format!(
            "{}/v0/release/{ident}/history?{limit}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            limit = utf8_percent_encode(&query_limit, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetReleaseHistoryResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityHistoryEntry>>(&buf)?;

                    Ok(GetReleaseHistoryResponse::FoundEntityHistory(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseHistoryResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseHistoryResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseHistoryResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_release_redirects(&self, param_ident: String, context: &Context) -> Box<Future<Item = GetReleaseRedirectsResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/release/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetReleaseRedirectsResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<String>>(&buf)?;

                    Ok(GetReleaseRedirectsResponse::FoundEntityRedirects(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseRedirectsResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseRedirectsResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseRedirectsResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_release_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = GetReleaseRevisionResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/release/rev/{rev_id}?{expand}{hide}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetReleaseRevisionResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ReleaseEntity>(&buf)?;

                    Ok(GetReleaseRevisionResponse::FoundEntityRevision(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseRevisionResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseRevisionResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetReleaseRevisionResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn lookup_release(
        &self,
        param_doi: Option<String>,
        param_wikidata_qid: Option<String>,
        param_isbn13: Option<String>,
        param_pmid: Option<String>,
        param_pmcid: Option<String>,
        param_core_id: Option<String>,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = LookupReleaseResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_doi = param_doi.map_or_else(String::new, |query| format!("doi={doi}&", doi = query.to_string()));
        let query_wikidata_qid = param_wikidata_qid.map_or_else(String::new, |query| format!("wikidata_qid={wikidata_qid}&", wikidata_qid = query.to_string()));
        let query_isbn13 = param_isbn13.map_or_else(String::new, |query| format!("isbn13={isbn13}&", isbn13 = query.to_string()));
        let query_pmid = param_pmid.map_or_else(String::new, |query| format!("pmid={pmid}&", pmid = query.to_string()));
        let query_pmcid = param_pmcid.map_or_else(String::new, |query| format!("pmcid={pmcid}&", pmcid = query.to_string()));
        let query_core_id = param_core_id.map_or_else(String::new, |query| format!("core_id={core_id}&", core_id = query.to_string()));
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/release/lookup?{doi}{wikidata_qid}{isbn13}{pmid}{pmcid}{core_id}{expand}{hide}",
            self.base_path,
            doi = utf8_percent_encode(&query_doi, QUERY_ENCODE_SET),
            wikidata_qid = utf8_percent_encode(&query_wikidata_qid, QUERY_ENCODE_SET),
            isbn13 = utf8_percent_encode(&query_isbn13, QUERY_ENCODE_SET),
            pmid = utf8_percent_encode(&query_pmid, QUERY_ENCODE_SET),
            pmcid = utf8_percent_encode(&query_pmcid, QUERY_ENCODE_SET),
            core_id = utf8_percent_encode(&query_core_id, QUERY_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<LookupReleaseResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ReleaseEntity>(&buf)?;

                    Ok(LookupReleaseResponse::FoundEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupReleaseResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupReleaseResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(LookupReleaseResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn update_release(
        &self,
        param_ident: String,
        param_entity: models::ReleaseEntity,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = UpdateReleaseResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/release/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Put, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::UPDATE_RELEASE.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<UpdateReleaseResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(UpdateReleaseResponse::UpdatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateReleaseResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateReleaseResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateReleaseResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn create_work_batch(
        &self,
        param_entity_list: &Vec<models::WorkEntity>,
        param_autoaccept: Option<bool>,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = CreateWorkBatchResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_autoaccept = param_autoaccept.map_or_else(String::new, |query| format!("autoaccept={autoaccept}&", autoaccept = query.to_string()));
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/work/batch?{autoaccept}{editgroup_id}",
            self.base_path,
            autoaccept = utf8_percent_encode(&query_autoaccept, QUERY_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity_list).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Post, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::CREATE_WORK_BATCH.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<CreateWorkBatchResponse, ApiError> {
            match response.status.to_u16() {
                201 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityEdit>>(&buf)?;

                    Ok(CreateWorkBatchResponse::CreatedEntities(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateWorkBatchResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateWorkBatchResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(CreateWorkBatchResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_work(&self, param_ident: String, param_editgroup_id: Option<String>, context: &Context) -> Box<Future<Item = DeleteWorkResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/work/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteWorkResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(DeleteWorkResponse::DeletedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteWorkResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteWorkResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteWorkResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn delete_work_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = DeleteWorkEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/work/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Delete, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<DeleteWorkEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::Success>(&buf)?;

                    Ok(DeleteWorkEditResponse::DeletedEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteWorkEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteWorkEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(DeleteWorkEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_work(&self, param_ident: String, param_expand: Option<String>, param_hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/work/{ident}?{expand}{hide}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetWorkResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::WorkEntity>(&buf)?;

                    Ok(GetWorkResponse::FoundEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_work_edit(&self, param_edit_id: i64, context: &Context) -> Box<Future<Item = GetWorkEditResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/work/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetWorkEditResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(GetWorkEditResponse::FoundEdit(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkEditResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkEditResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkEditResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_work_history(&self, param_ident: String, param_limit: Option<i64>, context: &Context) -> Box<Future<Item = GetWorkHistoryResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_limit = param_limit.map_or_else(String::new, |query| format!("limit={limit}&", limit = query.to_string()));

        let url = format!(
            "{}/v0/work/{ident}/history?{limit}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            limit = utf8_percent_encode(&query_limit, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetWorkHistoryResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::EntityHistoryEntry>>(&buf)?;

                    Ok(GetWorkHistoryResponse::FoundEntityHistory(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkHistoryResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkHistoryResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkHistoryResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_work_redirects(&self, param_ident: String, context: &Context) -> Box<Future<Item = GetWorkRedirectsResponse, Error = ApiError> + Send> {
        let url = format!(
            "{}/v0/work/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetWorkRedirectsResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<String>>(&buf)?;

                    Ok(GetWorkRedirectsResponse::FoundEntityRedirects(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkRedirectsResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkRedirectsResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkRedirectsResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_work_releases(&self, param_ident: String, param_hide: Option<String>, context: &Context) -> Box<Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/work/{ident}/releases?{hide}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetWorkReleasesResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<Vec<models::ReleaseEntity>>(&buf)?;

                    Ok(GetWorkReleasesResponse::Found(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkReleasesResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkReleasesResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkReleasesResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn get_work_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = GetWorkRevisionResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_expand = param_expand.map_or_else(String::new, |query| format!("expand={expand}&", expand = query.to_string()));
        let query_hide = param_hide.map_or_else(String::new, |query| format!("hide={hide}&", hide = query.to_string()));

        let url = format!(
            "{}/v0/work/rev/{rev_id}?{expand}{hide}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), PATH_SEGMENT_ENCODE_SET),
            expand = utf8_percent_encode(&query_expand, QUERY_ENCODE_SET),
            hide = utf8_percent_encode(&query_hide, QUERY_ENCODE_SET)
        );

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Get, &url);
        let mut custom_headers = hyper::header::Headers::new();

        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<GetWorkRevisionResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::WorkEntity>(&buf)?;

                    Ok(GetWorkRevisionResponse::FoundEntityRevision(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkRevisionResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkRevisionResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(GetWorkRevisionResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
        Box::new(futures::done(result))
    }

    fn update_work(
        &self,
        param_ident: String,
        param_entity: models::WorkEntity,
        param_editgroup_id: Option<String>,
        context: &Context,
    ) -> Box<Future<Item = UpdateWorkResponse, Error = ApiError> + Send> {
        // Query parameters
        let query_editgroup_id = param_editgroup_id.map_or_else(String::new, |query| format!("editgroup_id={editgroup_id}&", editgroup_id = query.to_string()));

        let url = format!(
            "{}/v0/work/{ident}?{editgroup_id}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), PATH_SEGMENT_ENCODE_SET),
            editgroup_id = utf8_percent_encode(&query_editgroup_id, QUERY_ENCODE_SET)
        );

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        let hyper_client = (self.hyper_client)();
        let request = hyper_client.request(hyper::method::Method::Put, &url);
        let mut custom_headers = hyper::header::Headers::new();

        let request = request.body(&body);

        custom_headers.set(ContentType(mimetypes::requests::UPDATE_WORK.clone()));
        context.x_span_id.as_ref().map(|header| custom_headers.set(XSpanId(header.clone())));

        let request = request.headers(custom_headers);

        // Helper function to provide a code block to use `?` in (to be replaced by the `catch` block when it exists).
        fn parse_response(mut response: hyper::client::response::Response) -> Result<UpdateWorkResponse, ApiError> {
            match response.status.to_u16() {
                200 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::EntityEdit>(&buf)?;

                    Ok(UpdateWorkResponse::UpdatedEntity(body))
                }
                400 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateWorkResponse::BadRequest(body))
                }
                404 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateWorkResponse::NotFound(body))
                }
                500 => {
                    let mut buf = String::new();
                    response.read_to_string(&mut buf).map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                    let body = serde_json::from_str::<models::ErrorResponse>(&buf)?;

                    Ok(UpdateWorkResponse::GenericError(body))
                }
                code => {
                    let mut buf = [0; 100];
                    let debug_body = match response.read(&mut buf) {
                        Ok(len) => match str::from_utf8(&buf[..len]) {
                            Ok(body) => Cow::from(body),
                            Err(_) => Cow::from(format!("<Body was not UTF8: {:?}>", &buf[..len].to_vec())),
                        },
                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                    };
                    Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}", code, response.headers, debug_body)))
                }
            }
        }

        let result = request.send().map_err(|e| ApiError(format!("No response received: {}", e))).and_then(parse_response);
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
