#[macro_use]
extern crate fatcat_api;
extern crate chrono;
extern crate diesel;
extern crate dotenv;
extern crate futures;
extern crate hyper;
extern crate swagger;
#[macro_use]
extern crate error_chain;

pub mod api_server;

mod errors {
    error_chain!{}
}

pub use self::errors::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::io;
//use hyper;
//use fatcat_api;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub struct NewService;

impl hyper::server::NewService for NewService {
    type Request = (hyper::Request, fatcat_api::Context);
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Instance = fatcat_api::server::Service<api_server::Server>;

    /// Instantiate a new server.
    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(fatcat_api::server::Service::new(api_server::Server))
    }
}
