use futures;
use futures::{future, stream, Future, Stream};
use hyper;
use hyper::client::HttpConnector;
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use hyper::{Body, Response, Uri};
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use hyper_openssl::HttpsConnector;
use serde_json;
use std::borrow::Cow;
use std::convert::TryInto;
use std::error;
use std::fmt;
use std::io::{Error, ErrorKind, Read};
use std::path::Path;
use std::str;
use std::str::FromStr;
use std::string::ToString;
use std::sync::Arc;
use swagger;
use swagger::{client::Service, ApiError, AuthData, Connector, Has, XSpanIdString};
use url::form_urlencoded;
use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET, QUERY_ENCODE_SET};

use crate::header;
use crate::models;

url::define_encode_set! {
    /// This encode set is used for object IDs
    ///
    /// Aside from the special characters defined in the `PATH_SEGMENT_ENCODE_SET`,
    /// the vertical bar (|) is encoded.
    pub ID_ENCODE_SET = [PATH_SEGMENT_ENCODE_SET] | {'|'}
}

use crate::{
    AcceptEditgroupResponse, Api, AuthCheckResponse, AuthOidcResponse, CreateAuthTokenResponse,
    CreateContainerAutoBatchResponse, CreateContainerResponse, CreateCreatorAutoBatchResponse,
    CreateCreatorResponse, CreateEditgroupAnnotationResponse, CreateEditgroupResponse,
    CreateFileAutoBatchResponse, CreateFileResponse, CreateFilesetAutoBatchResponse,
    CreateFilesetResponse, CreateReleaseAutoBatchResponse, CreateReleaseResponse,
    CreateWebcaptureAutoBatchResponse, CreateWebcaptureResponse, CreateWorkAutoBatchResponse,
    CreateWorkResponse, DeleteContainerEditResponse, DeleteContainerResponse,
    DeleteCreatorEditResponse, DeleteCreatorResponse, DeleteFileEditResponse, DeleteFileResponse,
    DeleteFilesetEditResponse, DeleteFilesetResponse, DeleteReleaseEditResponse,
    DeleteReleaseResponse, DeleteWebcaptureEditResponse, DeleteWebcaptureResponse,
    DeleteWorkEditResponse, DeleteWorkResponse, GetChangelogEntryResponse, GetChangelogResponse,
    GetContainerEditResponse, GetContainerHistoryResponse, GetContainerRedirectsResponse,
    GetContainerResponse, GetContainerRevisionResponse, GetCreatorEditResponse,
    GetCreatorHistoryResponse, GetCreatorRedirectsResponse, GetCreatorReleasesResponse,
    GetCreatorResponse, GetCreatorRevisionResponse, GetEditgroupAnnotationsResponse,
    GetEditgroupResponse, GetEditgroupsReviewableResponse, GetEditorAnnotationsResponse,
    GetEditorEditgroupsResponse, GetEditorResponse, GetFileEditResponse, GetFileHistoryResponse,
    GetFileRedirectsResponse, GetFileResponse, GetFileRevisionResponse, GetFilesetEditResponse,
    GetFilesetHistoryResponse, GetFilesetRedirectsResponse, GetFilesetResponse,
    GetFilesetRevisionResponse, GetReleaseEditResponse, GetReleaseFilesResponse,
    GetReleaseFilesetsResponse, GetReleaseHistoryResponse, GetReleaseRedirectsResponse,
    GetReleaseResponse, GetReleaseRevisionResponse, GetReleaseWebcapturesResponse,
    GetWebcaptureEditResponse, GetWebcaptureHistoryResponse, GetWebcaptureRedirectsResponse,
    GetWebcaptureResponse, GetWebcaptureRevisionResponse, GetWorkEditResponse,
    GetWorkHistoryResponse, GetWorkRedirectsResponse, GetWorkReleasesResponse, GetWorkResponse,
    GetWorkRevisionResponse, LookupContainerResponse, LookupCreatorResponse, LookupFileResponse,
    LookupReleaseResponse, UpdateContainerResponse, UpdateCreatorResponse, UpdateEditgroupResponse,
    UpdateEditorResponse, UpdateFileResponse, UpdateFilesetResponse, UpdateReleaseResponse,
    UpdateWebcaptureResponse, UpdateWorkResponse,
};

/// Convert input into a base path, e.g. "http://example:123". Also checks the scheme as it goes.
fn into_base_path(
    input: &str,
    correct_scheme: Option<&'static str>,
) -> Result<String, ClientInitError> {
    // First convert to Uri, since a base path is a subset of Uri.
    let uri = Uri::from_str(input)?;

    let scheme = uri.scheme_part().ok_or(ClientInitError::InvalidScheme)?;

    // Check the scheme if necessary
    if let Some(correct_scheme) = correct_scheme {
        if scheme != correct_scheme {
            return Err(ClientInitError::InvalidScheme);
        }
    }

    let host = uri.host().ok_or_else(|| ClientInitError::MissingHost)?;
    let port = uri
        .port_part()
        .map(|x| format!(":{}", x))
        .unwrap_or_default();
    Ok(format!(
        "{}://{}{}{}",
        scheme,
        host,
        port,
        uri.path().trim_end_matches('/')
    ))
}

/// A client that implements the API by making HTTP calls out to a server.
pub struct Client<F> {
    /// Inner service
    client_service: Arc<Box<dyn Service<ReqBody = Body, Future = F> + Send + Sync>>,

    /// Base path of the API
    base_path: String,
}

impl<F> fmt::Debug for Client<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ base_path: {} }}", self.base_path)
    }
}

impl<F> Clone for Client<F> {
    fn clone(&self) -> Self {
        Client {
            client_service: self.client_service.clone(),
            base_path: self.base_path.clone(),
        }
    }
}

impl Client<hyper::client::ResponseFuture> {
    /// Create a client with a custom implementation of hyper::client::Connect.
    ///
    /// Intended for use with custom implementations of connect for e.g. protocol logging
    /// or similar functionality which requires wrapping the transport layer. When wrapping a TCP connection,
    /// this function should be used in conjunction with `swagger::Connector::builder()`.
    ///
    /// For ordinary tcp connections, prefer the use of `try_new_http`, `try_new_https`
    /// and `try_new_https_mutual`, to avoid introducing a dependency on the underlying transport layer.
    ///
    /// # Arguments
    ///
    /// * `base_path` - base path of the client API, i.e. "www.my-api-implementation.com"
    /// * `protocol` - Which protocol to use when constructing the request url, e.g. `Some("http")`
    /// * `connector` - Implementation of `hyper::client::Connect` to use for the client
    pub fn try_new_with_connector<C>(
        base_path: &str,
        protocol: Option<&'static str>,
        connector: C,
    ) -> Result<Self, ClientInitError>
    where
        C: hyper::client::connect::Connect + 'static,
        C::Transport: 'static,
        C::Future: 'static,
    {
        let client_service = Box::new(hyper::client::Client::builder().build(connector));

        Ok(Client {
            client_service: Arc::new(client_service),
            base_path: into_base_path(base_path, protocol)?,
        })
    }

    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "www.my-api-implementation.com"
    pub fn try_new_http(base_path: &str) -> Result<Self, ClientInitError> {
        let http_connector = Connector::builder().build();

        Self::try_new_with_connector(base_path, Some("http"), http_connector)
    }

    /// Create a client with a TLS connection to the server
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "www.my-api-implementation.com"
    pub fn try_new_https(base_path: &str) -> Result<Self, ClientInitError> {
        let https_connector = Connector::builder()
            .https()
            .build()
            .map_err(|e| ClientInitError::SslError(e))?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }

    /// Create a client with a TLS connection to the server using a pinned certificate
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "www.my-api-implementation.com"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn try_new_https_pinned<CA>(
        base_path: &str,
        ca_certificate: CA,
    ) -> Result<Self, ClientInitError>
    where
        CA: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .build()
            .map_err(|e| ClientInitError::SslError(e))?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }

    /// Create a client with a mutually authenticated TLS connection to the server.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "www.my-api-implementation.com"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    /// * `client_key` - Path to the client private key
    /// * `client_certificate` - Path to the client's public certificate associated with the private key
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn try_new_https_mutual<CA, K, D>(
        base_path: &str,
        ca_certificate: CA,
        client_key: K,
        client_certificate: D,
    ) -> Result<Self, ClientInitError>
    where
        CA: AsRef<Path>,
        K: AsRef<Path>,
        D: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .client_authentication(client_key, client_certificate)
            .build()
            .map_err(|e| ClientInitError::SslError(e))?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }
}

impl<F> Client<F> {
    /// Constructor for creating a `Client` by passing in a pre-made `swagger::Service`
    ///
    /// This allows adding custom wrappers around the underlying transport, for example for logging.
    pub fn try_new_with_client_service(
        client_service: Arc<Box<dyn Service<ReqBody = Body, Future = F> + Send + Sync>>,
        base_path: &str,
    ) -> Result<Self, ClientInitError> {
        Ok(Client {
            client_service: client_service,
            base_path: into_base_path(base_path, None)?,
        })
    }
}

/// Error type failing to create a Client
#[derive(Debug)]
pub enum ClientInitError {
    /// Invalid URL Scheme
    InvalidScheme,

    /// Invalid URI
    InvalidUri(hyper::http::uri::InvalidUri),

    /// Missing Hostname
    MissingHost,

    /// SSL Connection Error
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
    SslError(native_tls::Error),

    /// SSL Connection Error
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    SslError(openssl::error::ErrorStack),
}

impl From<hyper::http::uri::InvalidUri> for ClientInitError {
    fn from(err: hyper::http::uri::InvalidUri) -> ClientInitError {
        ClientInitError::InvalidUri(err)
    }
}

impl fmt::Display for ClientInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &dyn fmt::Debug = self;
        s.fmt(f)
    }
}

impl error::Error for ClientInitError {
    fn description(&self) -> &str {
        "Failed to produce a hyper client."
    }
}

impl<C, F> Api<C> for Client<F>
where
    C: Has<XSpanIdString> + Has<Option<AuthData>>,
    F: Future<Item = Response<Body>, Error = hyper::Error> + Send + 'static,
{
    fn accept_editgroup(
        &self,
        param_editgroup_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/accept",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Success>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AcceptEditgroupResponse::MergedSuccessfully
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AcceptEditgroupResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AcceptEditgroupResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AcceptEditgroupResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AcceptEditgroupResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                409 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AcceptEditgroupResponse::EditConflict
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AcceptEditgroupResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn auth_check(
        &self,
        param_role: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = AuthCheckResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/auth/check", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_role) = param_role {
            query_string.append_pair("role", &param_role.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Success>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthCheckResponse::Success
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthCheckResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthCheckResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthCheckResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthCheckResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn auth_oidc(
        &self,
        param_oidc_params: models::AuthOidc,
        context: &C,
    ) -> Box<dyn Future<Item = AuthOidcResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/auth/oidc", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_oidc_params).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::AuthOidcResult>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthOidcResponse::Found
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::AuthOidcResult>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthOidcResponse::Created
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthOidcResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthOidcResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthOidcResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                409 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthOidcResponse::Conflict
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            AuthOidcResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_auth_token(
        &self,
        param_editor_id: String,
        param_duration_seconds: Option<i32>,
        context: &C,
    ) -> Box<dyn Future<Item = CreateAuthTokenResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/auth/token/{editor_id}",
            self.base_path,
            editor_id = utf8_percent_encode(&param_editor_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_duration_seconds) = param_duration_seconds {
            query_string.append_pair("duration_seconds", &param_duration_seconds.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::AuthTokenResult>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateAuthTokenResponse::Success
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateAuthTokenResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateAuthTokenResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateAuthTokenResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateAuthTokenResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_container(
        &self,
        param_editgroup_id: String,
        param_entity: models::ContainerEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateContainerResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/container",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerResponse::CreatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_container_auto_batch(
        &self,
        param_auto_batch: models::ContainerAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateContainerAutoBatchResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/editgroup/auto/container/batch", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_auto_batch).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editgroup>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerAutoBatchResponse::CreatedEditgroup
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerAutoBatchResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerAutoBatchResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerAutoBatchResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerAutoBatchResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateContainerAutoBatchResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_creator(
        &self,
        param_editgroup_id: String,
        param_entity: models::CreatorEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateCreatorResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/creator",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorResponse::CreatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_creator_auto_batch(
        &self,
        param_auto_batch: models::CreatorAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateCreatorAutoBatchResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/editgroup/auto/creator/batch", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_auto_batch).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editgroup>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorAutoBatchResponse::CreatedEditgroup
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorAutoBatchResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorAutoBatchResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorAutoBatchResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorAutoBatchResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateCreatorAutoBatchResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_editgroup(
        &self,
        param_editgroup: models::Editgroup,
        context: &C,
    ) -> Box<dyn Future<Item = CreateEditgroupResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/editgroup", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_editgroup).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editgroup>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupResponse::SuccessfullyCreated
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_editgroup_annotation(
        &self,
        param_editgroup_id: String,
        param_annotation: models::EditgroupAnnotation,
        context: &C,
    ) -> Box<dyn Future<Item = CreateEditgroupAnnotationResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/annotation",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_annotation).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EditgroupAnnotation>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupAnnotationResponse::Created
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupAnnotationResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupAnnotationResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupAnnotationResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupAnnotationResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateEditgroupAnnotationResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_file(
        &self,
        param_editgroup_id: String,
        param_entity: models::FileEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateFileResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/file",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileResponse::CreatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_file_auto_batch(
        &self,
        param_auto_batch: models::FileAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateFileAutoBatchResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/editgroup/auto/file/batch", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_auto_batch).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editgroup>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileAutoBatchResponse::CreatedEditgroup
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileAutoBatchResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileAutoBatchResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileAutoBatchResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileAutoBatchResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFileAutoBatchResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_fileset(
        &self,
        param_editgroup_id: String,
        param_entity: models::FilesetEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateFilesetResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/fileset",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetResponse::CreatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_fileset_auto_batch(
        &self,
        param_auto_batch: models::FilesetAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateFilesetAutoBatchResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/editgroup/auto/fileset/batch", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_auto_batch).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editgroup>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetAutoBatchResponse::CreatedEditgroup
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetAutoBatchResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetAutoBatchResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetAutoBatchResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetAutoBatchResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateFilesetAutoBatchResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_release(
        &self,
        param_editgroup_id: String,
        param_entity: models::ReleaseEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateReleaseResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/release",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseResponse::CreatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_release_auto_batch(
        &self,
        param_auto_batch: models::ReleaseAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateReleaseAutoBatchResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/editgroup/auto/release/batch", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_auto_batch).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editgroup>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseAutoBatchResponse::CreatedEditgroup
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseAutoBatchResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseAutoBatchResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseAutoBatchResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseAutoBatchResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateReleaseAutoBatchResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_webcapture(
        &self,
        param_editgroup_id: String,
        param_entity: models::WebcaptureEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateWebcaptureResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/webcapture",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureResponse::CreatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_webcapture_auto_batch(
        &self,
        param_auto_batch: models::WebcaptureAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateWebcaptureAutoBatchResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/editgroup/auto/webcapture/batch", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_auto_batch).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editgroup>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureAutoBatchResponse::CreatedEditgroup
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureAutoBatchResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureAutoBatchResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureAutoBatchResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureAutoBatchResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWebcaptureAutoBatchResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_work(
        &self,
        param_editgroup_id: String,
        param_entity: models::WorkEntity,
        context: &C,
    ) -> Box<dyn Future<Item = CreateWorkResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/work",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkResponse::CreatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn create_work_auto_batch(
        &self,
        param_auto_batch: models::WorkAutoBatch,
        context: &C,
    ) -> Box<dyn Future<Item = CreateWorkAutoBatchResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/editgroup/auto/work/batch", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_auto_batch).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                201 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editgroup>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkAutoBatchResponse::CreatedEditgroup
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkAutoBatchResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkAutoBatchResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkAutoBatchResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkAutoBatchResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            CreateWorkAutoBatchResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_container(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteContainerResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/container/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerResponse::DeletedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_container_edit(
        &self,
        param_editgroup_id: String,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteContainerEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/container/edit/{edit_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Success>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerEditResponse::DeletedEdit
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerEditResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerEditResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerEditResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerEditResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteContainerEditResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_creator(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteCreatorResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/creator/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorResponse::DeletedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_creator_edit(
        &self,
        param_editgroup_id: String,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteCreatorEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/creator/edit/{edit_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Success>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorEditResponse::DeletedEdit
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorEditResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorEditResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorEditResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorEditResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteCreatorEditResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_file(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteFileResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/file/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileResponse::DeletedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_file_edit(
        &self,
        param_editgroup_id: String,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteFileEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/file/edit/{edit_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Success>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileEditResponse::DeletedEdit
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileEditResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileEditResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileEditResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileEditResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFileEditResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_fileset(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteFilesetResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/fileset/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetResponse::DeletedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_fileset_edit(
        &self,
        param_editgroup_id: String,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteFilesetEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/fileset/edit/{edit_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Success>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetEditResponse::DeletedEdit
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetEditResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetEditResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetEditResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetEditResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteFilesetEditResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_release(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteReleaseResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/release/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseResponse::DeletedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_release_edit(
        &self,
        param_editgroup_id: String,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteReleaseEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/release/edit/{edit_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Success>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseEditResponse::DeletedEdit
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseEditResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseEditResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseEditResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseEditResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteReleaseEditResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_webcapture(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteWebcaptureResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/webcapture/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureResponse::DeletedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_webcapture_edit(
        &self,
        param_editgroup_id: String,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteWebcaptureEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/webcapture/edit/{edit_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Success>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureEditResponse::DeletedEdit
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureEditResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureEditResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureEditResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureEditResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWebcaptureEditResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_work(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteWorkResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/work/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkResponse::DeletedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn delete_work_edit(
        &self,
        param_editgroup_id: String,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = DeleteWorkEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/work/edit/{edit_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Success>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkEditResponse::DeletedEdit
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkEditResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkEditResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkEditResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkEditResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            DeleteWorkEditResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn get_changelog(
        &self,
        param_limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetChangelogResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/changelog", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::ChangelogEntry>>(
                                                body,
                                            )
                                            .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetChangelogResponse::Success(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetChangelogResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetChangelogResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_changelog_entry(
        &self,
        param_index: i64,
        context: &C,
    ) -> Box<dyn Future<Item = GetChangelogEntryResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/changelog/{index}",
            self.base_path,
            index = utf8_percent_encode(&param_index.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ChangelogEntry>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetChangelogEntryResponse::FoundChangelogEntry(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetChangelogEntryResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetChangelogEntryResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetChangelogEntryResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_container(
        &self,
        param_ident: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/container/{ident}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetContainerResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_container_edit(
        &self,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/container/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetContainerEditResponse::FoundEdit(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerEditResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerEditResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerEditResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_container_history(
        &self,
        param_ident: String,
        param_limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerHistoryResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/container/{ident}/history",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::EntityHistoryEntry>>(
                                                body,
                                            )
                                            .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetContainerHistoryResponse::FoundEntityHistory(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerHistoryResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerHistoryResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerHistoryResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_container_redirects(
        &self,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerRedirectsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/container/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<String>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetContainerRedirectsResponse::FoundEntityRedirects(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerRedirectsResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerRedirectsResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerRedirectsResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_container_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetContainerRevisionResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/container/rev/{rev_id}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                    GetContainerRevisionResponse::FoundEntityRevision(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerRevisionResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerRevisionResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetContainerRevisionResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_creator(
        &self,
        param_ident: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/creator/{ident}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetCreatorResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_creator_edit(
        &self,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/creator/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetCreatorEditResponse::FoundEdit(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorEditResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorEditResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorEditResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_creator_history(
        &self,
        param_ident: String,
        param_limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorHistoryResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/creator/{ident}/history",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::EntityHistoryEntry>>(
                                                body,
                                            )
                                            .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetCreatorHistoryResponse::FoundEntityHistory(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorHistoryResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorHistoryResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorHistoryResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_creator_redirects(
        &self,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorRedirectsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/creator/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<String>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetCreatorRedirectsResponse::FoundEntityRedirects(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorRedirectsResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorRedirectsResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorRedirectsResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_creator_releases(
        &self,
        param_ident: String,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorReleasesResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/creator/{ident}/releases",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::ReleaseEntity>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorReleasesResponse::Found(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorReleasesResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorReleasesResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorReleasesResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_creator_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetCreatorRevisionResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/creator/rev/{rev_id}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                    GetCreatorRevisionResponse::FoundEntityRevision(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorRevisionResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorRevisionResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetCreatorRevisionResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_editgroup(
        &self,
        param_editgroup_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetEditgroupResponse::Found(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditgroupResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditgroupResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditgroupResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_editgroup_annotations(
        &self,
        param_editgroup_id: String,
        param_expand: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditgroupAnnotationsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/annotations",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<Vec<models::EditgroupAnnotation>>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditgroupAnnotationsResponse::Success
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditgroupAnnotationsResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditgroupAnnotationsResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditgroupAnnotationsResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditgroupAnnotationsResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditgroupAnnotationsResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn get_editgroups_reviewable(
        &self,
        param_expand: Option<String>,
        param_limit: Option<i64>,
        param_before: Option<chrono::DateTime<chrono::Utc>>,
        param_since: Option<chrono::DateTime<chrono::Utc>>,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditgroupsReviewableResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/editgroup/reviewable", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        if let Some(param_before) = param_before {
            query_string.append_pair("before", &param_before.to_string());
        }
        if let Some(param_since) = param_since {
            query_string.append_pair("since", &param_since.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::Editgroup>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditgroupsReviewableResponse::Found(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditgroupsReviewableResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditgroupsReviewableResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetEditgroupsReviewableResponse::GenericError(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_editor(
        &self,
        param_editor_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditorResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editor/{editor_id}",
            self.base_path,
            editor_id = utf8_percent_encode(&param_editor_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetEditorResponse::Found(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditorResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditorResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditorResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_editor_annotations(
        &self,
        param_editor_id: String,
        param_limit: Option<i64>,
        param_before: Option<chrono::DateTime<chrono::Utc>>,
        param_since: Option<chrono::DateTime<chrono::Utc>>,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditorAnnotationsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editor/{editor_id}/annotations",
            self.base_path,
            editor_id = utf8_percent_encode(&param_editor_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        if let Some(param_before) = param_before {
            query_string.append_pair("before", &param_before.to_string());
        }
        if let Some(param_since) = param_since {
            query_string.append_pair("since", &param_since.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<Vec<models::EditgroupAnnotation>>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditorAnnotationsResponse::Success
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditorAnnotationsResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditorAnnotationsResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditorAnnotationsResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditorAnnotationsResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            GetEditorAnnotationsResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn get_editor_editgroups(
        &self,
        param_editor_id: String,
        param_limit: Option<i64>,
        param_before: Option<chrono::DateTime<chrono::Utc>>,
        param_since: Option<chrono::DateTime<chrono::Utc>>,
        context: &C,
    ) -> Box<dyn Future<Item = GetEditorEditgroupsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editor/{editor_id}/editgroups",
            self.base_path,
            editor_id = utf8_percent_encode(&param_editor_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        if let Some(param_before) = param_before {
            query_string.append_pair("before", &param_before.to_string());
        }
        if let Some(param_since) = param_since {
            query_string.append_pair("since", &param_since.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::Editgroup>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditorEditgroupsResponse::Found(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditorEditgroupsResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditorEditgroupsResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetEditorEditgroupsResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_file(
        &self,
        param_ident: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/file/{ident}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetFileResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_file_edit(
        &self,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/file/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetFileEditResponse::FoundEdit(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileEditResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileEditResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileEditResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_file_history(
        &self,
        param_ident: String,
        param_limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileHistoryResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/file/{ident}/history",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::EntityHistoryEntry>>(
                                                body,
                                            )
                                            .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileHistoryResponse::FoundEntityHistory(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileHistoryResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileHistoryResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileHistoryResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_file_redirects(
        &self,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileRedirectsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/file/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<String>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetFileRedirectsResponse::FoundEntityRedirects(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileRedirectsResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileRedirectsResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileRedirectsResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_file_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFileRevisionResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/file/rev/{rev_id}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                    GetFileRevisionResponse::FoundEntityRevision(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileRevisionResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileRevisionResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFileRevisionResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_fileset(
        &self,
        param_ident: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/fileset/{ident}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::FilesetEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_fileset_edit(
        &self,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/fileset/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetFilesetEditResponse::FoundEdit(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetEditResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetEditResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetEditResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_fileset_history(
        &self,
        param_ident: String,
        param_limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetHistoryResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/fileset/{ident}/history",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::EntityHistoryEntry>>(
                                                body,
                                            )
                                            .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetFilesetHistoryResponse::FoundEntityHistory(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetHistoryResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetHistoryResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetHistoryResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_fileset_redirects(
        &self,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetRedirectsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/fileset/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<String>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetFilesetRedirectsResponse::FoundEntityRedirects(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetRedirectsResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetRedirectsResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetRedirectsResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_fileset_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetFilesetRevisionResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/fileset/rev/{rev_id}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::FilesetEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetFilesetRevisionResponse::FoundEntityRevision(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetRevisionResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetRevisionResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetFilesetRevisionResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_release(
        &self,
        param_ident: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/release/{ident}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetReleaseResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_release_edit(
        &self,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/release/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetReleaseEditResponse::FoundEdit(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseEditResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseEditResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseEditResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_release_files(
        &self,
        param_ident: String,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseFilesResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/release/{ident}/files",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::FileEntity>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseFilesResponse::Found(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseFilesResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseFilesResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseFilesResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_release_filesets(
        &self,
        param_ident: String,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseFilesetsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/release/{ident}/filesets",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::FilesetEntity>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseFilesetsResponse::Found(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseFilesetsResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseFilesetsResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseFilesetsResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_release_history(
        &self,
        param_ident: String,
        param_limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseHistoryResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/release/{ident}/history",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::EntityHistoryEntry>>(
                                                body,
                                            )
                                            .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetReleaseHistoryResponse::FoundEntityHistory(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseHistoryResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseHistoryResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseHistoryResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_release_redirects(
        &self,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseRedirectsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/release/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<String>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetReleaseRedirectsResponse::FoundEntityRedirects(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseRedirectsResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseRedirectsResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseRedirectsResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_release_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseRevisionResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/release/rev/{rev_id}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                    GetReleaseRevisionResponse::FoundEntityRevision(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseRevisionResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseRevisionResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseRevisionResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_release_webcaptures(
        &self,
        param_ident: String,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetReleaseWebcapturesResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/release/{ident}/webcaptures",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::WebcaptureEntity>>(
                                                body,
                                            )
                                            .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseWebcapturesResponse::Found(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseWebcapturesResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseWebcapturesResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetReleaseWebcapturesResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_webcapture(
        &self,
        param_ident: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/webcapture/{ident}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::WebcaptureEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_webcapture_edit(
        &self,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/webcapture/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetWebcaptureEditResponse::FoundEdit(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureEditResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureEditResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureEditResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_webcapture_history(
        &self,
        param_ident: String,
        param_limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureHistoryResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/webcapture/{ident}/history",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::EntityHistoryEntry>>(
                                                body,
                                            )
                                            .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetWebcaptureHistoryResponse::FoundEntityHistory(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureHistoryResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureHistoryResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureHistoryResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_webcapture_redirects(
        &self,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureRedirectsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/webcapture/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<String>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetWebcaptureRedirectsResponse::FoundEntityRedirects(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureRedirectsResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureRedirectsResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetWebcaptureRedirectsResponse::GenericError(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_webcapture_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWebcaptureRevisionResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/webcapture/rev/{rev_id}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::WebcaptureEntity>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetWebcaptureRevisionResponse::FoundEntityRevision(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureRevisionResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureRevisionResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWebcaptureRevisionResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_work(
        &self,
        param_ident: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/work/{ident}",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetWorkResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_work_edit(
        &self,
        param_edit_id: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkEditResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/work/edit/{edit_id}",
            self.base_path,
            edit_id = utf8_percent_encode(&param_edit_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| GetWorkEditResponse::FoundEdit(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkEditResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkEditResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkEditResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_work_history(
        &self,
        param_ident: String,
        param_limit: Option<i64>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkHistoryResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/work/{ident}/history",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_limit) = param_limit {
            query_string.append_pair("limit", &param_limit.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::EntityHistoryEntry>>(
                                                body,
                                            )
                                            .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkHistoryResponse::FoundEntityHistory(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkHistoryResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkHistoryResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkHistoryResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_work_redirects(
        &self,
        param_ident: String,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkRedirectsResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/work/{ident}/redirects",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<String>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| {
                                    GetWorkRedirectsResponse::FoundEntityRedirects(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkRedirectsResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkRedirectsResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkRedirectsResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_work_releases(
        &self,
        param_ident: String,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkReleasesResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/work/{ident}/releases",
            self.base_path,
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<Vec<models::ReleaseEntity>>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkReleasesResponse::Found(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkReleasesResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkReleasesResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkReleasesResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn get_work_revision(
        &self,
        param_rev_id: String,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = GetWorkRevisionResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/work/rev/{rev_id}",
            self.base_path,
            rev_id = utf8_percent_encode(&param_rev_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| {
                                    GetWorkRevisionResponse::FoundEntityRevision(body)
                                }),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkRevisionResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkRevisionResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| GetWorkRevisionResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn lookup_container(
        &self,
        param_issnl: Option<String>,
        param_wikidata_qid: Option<String>,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = LookupContainerResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/container/lookup", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_issnl) = param_issnl {
            query_string.append_pair("issnl", &param_issnl.to_string());
        }
        if let Some(param_wikidata_qid) = param_wikidata_qid {
            query_string.append_pair("wikidata_qid", &param_wikidata_qid.to_string());
        }
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| LookupContainerResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupContainerResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupContainerResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupContainerResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn lookup_creator(
        &self,
        param_orcid: Option<String>,
        param_wikidata_qid: Option<String>,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = LookupCreatorResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/creator/lookup", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_orcid) = param_orcid {
            query_string.append_pair("orcid", &param_orcid.to_string());
        }
        if let Some(param_wikidata_qid) = param_wikidata_qid {
            query_string.append_pair("wikidata_qid", &param_wikidata_qid.to_string());
        }
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| LookupCreatorResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupCreatorResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupCreatorResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupCreatorResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn lookup_file(
        &self,
        param_md5: Option<String>,
        param_sha1: Option<String>,
        param_sha256: Option<String>,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = LookupFileResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/file/lookup", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_md5) = param_md5 {
            query_string.append_pair("md5", &param_md5.to_string());
        }
        if let Some(param_sha1) = param_sha1 {
            query_string.append_pair("sha1", &param_sha1.to_string());
        }
        if let Some(param_sha256) = param_sha256 {
            query_string.append_pair("sha256", &param_sha256.to_string());
        }
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| LookupFileResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupFileResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupFileResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupFileResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn lookup_release(
        &self,
        param_doi: Option<String>,
        param_wikidata_qid: Option<String>,
        param_isbn13: Option<String>,
        param_pmid: Option<String>,
        param_pmcid: Option<String>,
        param_core: Option<String>,
        param_arxiv: Option<String>,
        param_jstor: Option<String>,
        param_ark: Option<String>,
        param_mag: Option<String>,
        param_expand: Option<String>,
        param_hide: Option<String>,
        context: &C,
    ) -> Box<dyn Future<Item = LookupReleaseResponse, Error = ApiError> + Send> {
        let mut uri = format!("{}/v0/release/lookup", self.base_path);

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_doi) = param_doi {
            query_string.append_pair("doi", &param_doi.to_string());
        }
        if let Some(param_wikidata_qid) = param_wikidata_qid {
            query_string.append_pair("wikidata_qid", &param_wikidata_qid.to_string());
        }
        if let Some(param_isbn13) = param_isbn13 {
            query_string.append_pair("isbn13", &param_isbn13.to_string());
        }
        if let Some(param_pmid) = param_pmid {
            query_string.append_pair("pmid", &param_pmid.to_string());
        }
        if let Some(param_pmcid) = param_pmcid {
            query_string.append_pair("pmcid", &param_pmcid.to_string());
        }
        if let Some(param_core) = param_core {
            query_string.append_pair("core", &param_core.to_string());
        }
        if let Some(param_arxiv) = param_arxiv {
            query_string.append_pair("arxiv", &param_arxiv.to_string());
        }
        if let Some(param_jstor) = param_jstor {
            query_string.append_pair("jstor", &param_jstor.to_string());
        }
        if let Some(param_ark) = param_ark {
            query_string.append_pair("ark", &param_ark.to_string());
        }
        if let Some(param_mag) = param_mag {
            query_string.append_pair("mag", &param_mag.to_string());
        }
        if let Some(param_expand) = param_expand {
            query_string.append_pair("expand", &param_expand.to_string());
        }
        if let Some(param_hide) = param_hide {
            query_string.append_pair("hide", &param_hide.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        Box::new(
            self.client_service
                .request(request)
                .map_err(|e| ApiError(format!("No response received: {}", e)))
                .and_then(|mut response| match response.status().as_u16() {
                    200 => {
                        let body = response.into_body();
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
                                .map(move |body| LookupReleaseResponse::FoundEntity(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    400 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupReleaseResponse::BadRequest(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    404 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupReleaseResponse::NotFound(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    500 => {
                        let body = response.into_body();
                        Box::new(
                            body.concat2()
                                .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                                .and_then(|body| {
                                    str::from_utf8(&body)
                                        .map_err(|e| {
                                            ApiError(format!("Response was not valid UTF8: {}", e))
                                        })
                                        .and_then(|body| {
                                            serde_json::from_str::<models::ErrorResponse>(body)
                                                .map_err(|e| e.into())
                                        })
                                })
                                .map(move |body| LookupReleaseResponse::GenericError(body)),
                        ) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                    code => {
                        let headers = response.headers().clone();
                        Box::new(response.into_body().take(100).concat2().then(move |body| {
                            future::err(ApiError(format!(
                                "Unexpected response code {}:\n{:?}\n\n{}",
                                code,
                                headers,
                                match body {
                                    Ok(ref body) => match str::from_utf8(body) {
                                        Ok(body) => Cow::from(body),
                                        Err(e) =>
                                            Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                    },
                                    Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                }
                            )))
                        })) as Box<dyn Future<Item = _, Error = _> + Send>
                    }
                }),
        )
    }

    fn update_container(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        param_entity: models::ContainerEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateContainerResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/container/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateContainerResponse::UpdatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateContainerResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateContainerResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateContainerResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateContainerResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateContainerResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn update_creator(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        param_entity: models::CreatorEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateCreatorResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/creator/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateCreatorResponse::UpdatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateCreatorResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateCreatorResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateCreatorResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateCreatorResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateCreatorResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn update_editgroup(
        &self,
        param_editgroup_id: String,
        param_editgroup: models::Editgroup,
        param_submit: Option<bool>,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateEditgroupResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        if let Some(param_submit) = param_submit {
            query_string.append_pair("submit", &param_submit.to_string());
        }
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body =
            serde_json::to_string(&param_editgroup).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editgroup>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditgroupResponse::UpdatedEditgroup
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditgroupResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditgroupResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditgroupResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditgroupResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditgroupResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn update_editor(
        &self,
        param_editor_id: String,
        param_editor: models::Editor,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateEditorResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editor/{editor_id}",
            self.base_path,
            editor_id = utf8_percent_encode(&param_editor_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_editor).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::Editor>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditorResponse::UpdatedEditor
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditorResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditorResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditorResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditorResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateEditorResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn update_file(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        param_entity: models::FileEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateFileResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/file/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFileResponse::UpdatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFileResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFileResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFileResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFileResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFileResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn update_fileset(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        param_entity: models::FilesetEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateFilesetResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/fileset/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFilesetResponse::UpdatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFilesetResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFilesetResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFilesetResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFilesetResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateFilesetResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn update_release(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        param_entity: models::ReleaseEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateReleaseResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/release/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateReleaseResponse::UpdatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateReleaseResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateReleaseResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateReleaseResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateReleaseResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateReleaseResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn update_webcapture(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        param_entity: models::WebcaptureEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateWebcaptureResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/webcapture/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );
        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWebcaptureResponse::UpdatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWebcaptureResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWebcaptureResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWebcaptureResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWebcaptureResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWebcaptureResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }

    fn update_work(
        &self,
        param_editgroup_id: String,
        param_ident: String,
        param_entity: models::WorkEntity,
        context: &C,
    ) -> Box<dyn Future<Item = UpdateWorkResponse, Error = ApiError> + Send> {
        let mut uri = format!(
            "{}/v0/editgroup/{editgroup_id}/work/{ident}",
            self.base_path,
            editgroup_id = utf8_percent_encode(&param_editgroup_id.to_string(), ID_ENCODE_SET),
            ident = utf8_percent_encode(&param_ident.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let mut query_string = url::form_urlencoded::Serializer::new("".to_owned());
        let query_string_str = query_string.finish();
        if !query_string_str.is_empty() {
            uri += "?";
            uri += &query_string_str;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to build URI: {}",
                    err
                ))))
            }
        };

        let mut request = match hyper::Request::builder()
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => {
                return Box::new(future::err(ApiError(format!(
                    "Unable to create request: {}",
                    e
                ))))
            }
        };

        let body = serde_json::to_string(&param_entity).expect("impossible to fail to serialize");

        *request.body_mut() = Body::from(body);

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    ))))
                }
            },
        );

        let header = HeaderValue::from_str(
            (context as &dyn Has<XSpanIdString>)
                .get()
                .0
                .clone()
                .to_string()
                .as_str(),
        );
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Box::new(future::err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    ))))
                }
            },
        );

        if let Some(auth_data) = (context as &dyn Has<Option<AuthData>>).get().as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            match auth_data {
                _ => {}
            }
        }

        Box::new(self.client_service.request(request)
                             .map_err(|e| ApiError(format!("No response received: {}", e)))
                             .and_then(|mut response| {
            match response.status().as_u16() {
                200 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::EntityEdit>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWorkResponse::UpdatedEntity
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                400 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWorkResponse::BadRequest
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                401 => {
                    let response_www_authenticate = match response.headers().get(HeaderName::from_static("www_authenticate")) {
                        Some(response_www_authenticate) => response_www_authenticate.clone(),
                        None => return Box::new(future::err(ApiError(String::from("Required response header WWW_Authenticate for response 401 was not found.")))) as Box<dyn Future<Item=_, Error=_> + Send>,
                    };
                    let response_www_authenticate = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_www_authenticate) {
                        Ok(value) => value,
                        Err(e) => {
                            return Box::new(future::err(ApiError(format!("Invalid response header WWW_Authenticate for response 401 - {}", e)))) as Box<dyn Future<Item=_, Error=_> + Send>;
                        },
                    };
                    let response_www_authenticate = response_www_authenticate.0;

                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWorkResponse::NotAuthorized
                            {
                                body: body,
                                www_authenticate: response_www_authenticate,
                            }
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                403 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWorkResponse::Forbidden
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                404 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWorkResponse::NotFound
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                500 => {
                    let body = response.into_body();
                    Box::new(
                        body
                        .concat2()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                        .and_then(|body|
                        str::from_utf8(&body)
                                             .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))
                                             .and_then(|body|
                                                 serde_json::from_str::<models::ErrorResponse>(body)
                                                     .map_err(|e| e.into())
                                             )
                                 )
                        .map(move |body| {
                            UpdateWorkResponse::GenericError
                            (body)
                        })
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                },
                code => {
                    let headers = response.headers().clone();
                    Box::new(response.into_body()
                            .take(100)
                            .concat2()
                            .then(move |body|
                                future::err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
                                    code,
                                    headers,
                                    match body {
                                        Ok(ref body) => match str::from_utf8(body) {
                                            Ok(body) => Cow::from(body),
                                            Err(e) => Cow::from(format!("<Body was not UTF8: {:?}>", e)),
                                        },
                                        Err(e) => Cow::from(format!("<Failed to read body: {}>", e)),
                                    })))
                            )
                    ) as Box<dyn Future<Item=_, Error=_> + Send>
                }
            }
        }))
    }
}
