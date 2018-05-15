use Api;
use hyper;
use hyper::{Error, Request, Response, StatusCode};
use server::url::form_urlencoded;
use std::io;
use swagger::auth::{AuthData, Authorization, Scopes};

pub struct NewService<T>
where
    T: hyper::server::NewService<
        Request = (Request, Option<AuthData>),
        Response = Response,
        Error = Error,
    >,
{
    inner: T,
}

impl<T> NewService<T>
where
    T: hyper::server::NewService<
            Request = (Request, Option<AuthData>),
            Response = Response,
            Error = Error,
        >
        + 'static,
{
    pub fn new(inner: T) -> NewService<T> {
        NewService { inner }
    }
}

impl<T> hyper::server::NewService for NewService<T>
where
    T: hyper::server::NewService<
            Request = (Request, Option<AuthData>),
            Response = Response,
            Error = Error,
        >
        + 'static,
{
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Instance = Service<T::Instance>;

    fn new_service(&self) -> Result<Self::Instance, io::Error> {
        self.inner.new_service().map(|s| Service::new(s))
    }
}

/// Middleware to extract authentication data from request
pub struct Service<T>
where
    T: hyper::server::Service<
        Request = (Request, Option<AuthData>),
        Response = Response,
        Error = Error,
    >,
{
    inner: T,
}

impl<T> Service<T>
where
    T: hyper::server::Service<
        Request = (Request, Option<AuthData>),
        Response = Response,
        Error = Error,
    >,
{
    pub fn new(inner: T) -> Service<T> {
        Service { inner }
    }
}

impl<T> hyper::server::Service for Service<T>
where
    T: hyper::server::Service<
        Request = (Request, Option<AuthData>),
        Response = Response,
        Error = Error,
    >,
{
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = T::Future;

    fn call(&self, req: Self::Request) -> Self::Future {
        return self.inner.call((req, None));
    }
}
