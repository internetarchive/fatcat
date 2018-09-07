extern crate chrono;
extern crate fatcat_api;
#[macro_use]
extern crate diesel;
extern crate diesel_migrations;
extern crate dotenv;
extern crate futures;
extern crate uuid;
#[macro_use]
extern crate hyper;
//extern crate swagger;
#[macro_use]
extern crate error_chain;
extern crate iron;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate data_encoding;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate sha1;

pub mod api_helpers;
pub mod api_server;
pub mod api_wrappers;
pub mod database_models;
pub mod database_schema;
pub mod database_entity_crud;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links { Fmt(::std::fmt::Error);
                        Diesel(::diesel::result::Error);
                        R2d2(::diesel::r2d2::Error);
                        Uuid(::uuid::ParseError);
                        Io(::std::io::Error) #[cfg(unix)];
                        Serde(::serde_json::Error);
        }
        errors {
            InvalidFatcatId(id: String) {
                description("invalid fatcat identifier syntax")
                display("invalid fatcat identifier (expect 26-char base32 encoded): {}", id)
            }
            MalformedExternalId(id: String) {
                description("external identifier doesn't match required pattern")
                display("external identifier doesn't match required pattern")
            }
            EditgroupAlreadyAccepted(id: String) {
                description("editgroup was already accepted")
                display("attempted to accept an editgroup which was already accepted: {}", id)
            }
        }
    }
}

#[doc(hidden)]
pub use errors::*;

pub use self::errors::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use iron::middleware::AfterMiddleware;
use iron::{Request, Response};
use std::env;

#[cfg(feature = "postgres")]
embed_migrations!("../migrations/");

pub type ConnectionPool = diesel::r2d2::Pool<ConnectionManager<diesel::pg::PgConnection>>;

/// Establish a direct database connection. Not currently used, but could be helpful for
/// single-threaded tests or utilities.
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Instantiate a new API server with a pooled database connection
pub fn server() -> Result<api_server::Server> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.");
    Ok(api_server::Server { db_pool: pool })
}

pub fn test_server() -> Result<api_server::Server> {
    dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    env::set_var("DATABASE_URL", database_url);

    let server = server()?;
    let conn = server.db_pool.get().expect("db_pool error");

    // run migrations; revert latest (dummy data); re-run latest
    diesel_migrations::run_pending_migrations(&conn).unwrap();
    diesel_migrations::revert_latest_migration(&conn).unwrap();
    diesel_migrations::run_pending_migrations(&conn).unwrap();
    Ok(server)
}

/// HTTP header middleware
header! { (XClacksOverhead, "X-Clacks-Overhead") => [String] }

pub struct XClacksOverheadMiddleware;

impl AfterMiddleware for XClacksOverheadMiddleware {
    fn after(&self, _req: &mut Request, mut res: Response) -> iron::IronResult<Response> {
        res.headers
            .set(XClacksOverhead("GNU aaronsw, jpb".to_owned()));
        Ok(res)
    }
}
