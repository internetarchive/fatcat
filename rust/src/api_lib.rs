//! Main library entry point for fatcat implementation.

// Imports required by server library.
// extern crate fatcat;
// extern crate swagger;
extern crate futures;
extern crate chrono;
#[macro_use]
extern crate error_chain;

mod server;

mod errors {
    error_chain!{}
}

pub use self::errors::*;
use std::io;
use hyper;
use fatcat;

pub struct NewService;

impl hyper::server::NewService for NewService {
    type Request = (hyper::Request, fatcat::Context);
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Instance = fatcat::server::Service<server::Server>;

    /// Instantiate a new server.
    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(fatcat::server::Service::new(server::Server))
    }
}
