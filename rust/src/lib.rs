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

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

/// Instantiate a new server.
pub fn server() -> Result<api_server::Server> {
    Ok(api_server::Server {})
}
