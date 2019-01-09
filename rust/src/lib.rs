#![allow(proc_macro_derive_resolution_fallback)]
#![recursion_limit = "128"]

extern crate chrono;
extern crate fatcat_api_spec;
#[macro_use]
extern crate diesel;
extern crate diesel_migrations;
extern crate dotenv;
extern crate futures;
extern crate uuid;
#[macro_use]
extern crate hyper;
extern crate swagger;
#[macro_use]
extern crate error_chain;
extern crate iron;
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate data_encoding;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate macaroon;
extern crate sha1;
extern crate rand;

pub mod api_entity_crud;
pub mod api_helpers;
pub mod api_server;
pub mod api_wrappers;
pub mod auth;
pub mod database_models;
pub mod database_schema;

pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links { Fmt(::std::fmt::Error);
                        Diesel(::diesel::result::Error);
                        R2d2(::diesel::r2d2::Error);
                        Uuid(::uuid::ParseError);
                        Io(::std::io::Error) #[cfg(unix)];
                        Serde(::serde_json::Error);
                        Utf8Decode(::std::string::FromUtf8Error);
                        StringDecode(::data_encoding::DecodeError);
        }
        errors {
            InvalidFatcatId(id: String) {
                description("invalid fatcat identifier syntax")
                display("invalid fatcat identifier (expect 26-char base32 encoded): {}", id)
            }
            MalformedExternalId(id: String) {
                description("external identifier doesn't match required pattern")
                display("external identifier doesn't match required pattern: {}", id)
            }
            MalformedChecksum(hash: String) {
                description("checksum doesn't match required pattern (hex encoding)")
                display("checksum doesn't match required pattern (hex encoding): {}", hash)
            }
            NotInControlledVocabulary(word: String) {
                description("word or type not correct for controlled vocabulary")
                display("word or type not correct for controlled vocabulary")
            }
            EditgroupAlreadyAccepted(id: String) {
                description("editgroup was already accepted")
                display("attempted to accept or mutate an editgroup which was already accepted: {}", id)
            }
            MissingOrMultipleExternalId(message: String) {
                description("external identifiers missing or multiple specified")
                display("external identifiers missing or multiple specified; please supply exactly one")
            }
            InvalidEntityStateTransform(message: String) {
                description("Invalid Entity State Transform")
                display("tried to mutate an entity which was not in an appropriate state: {}", message)
            }
            InvalidCredentials(message: String) {
                description("auth token was missing, expired, revoked, or corrupt")
                display("auth token was missing, expired, revoked, or corrupt: {}", message)
            }
            InsufficientPrivileges(message: String) {
                description("editor account doesn't have authorization")
                display("editor account doesn't have authorization: {}", message)
            }
            OtherBadRequest(message: String) {
                description("catch-all error for bad or unallowed requests")
                display("broke a constraint or made an otherwise invalid request: {}", message)
            }
        }
    }
}

#[doc(hidden)]
pub use crate::errors::*;

pub use self::errors::*;
use crate::auth::AuthConfectionary;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use iron::middleware::AfterMiddleware;
use iron::{Request, Response};
use std::{env, thread, time};
use std::process::Command;
use rand::Rng;

#[cfg(feature = "postgres")]
embed_migrations!("../migrations/");

pub type ConnectionPool = diesel::r2d2::Pool<ConnectionManager<diesel::pg::PgConnection>>;

/// Instantiate a new API server with a pooled database connection
pub fn database_worker_pool() -> Result<ConnectionPool> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.");
    Ok(pool)
}

pub fn env_confectionary() -> Result<AuthConfectionary> {
    let auth_location = env::var("AUTH_LOCATION").expect("AUTH_LOCATION must be set");
    let auth_key = env::var("AUTH_SECRET_KEY").expect("AUTH_SECRET_KEY must be set");
    let auth_key_ident = env::var("AUTH_KEY_IDENT").expect("AUTH_KEY_IDENT must be set");
    info!("Loaded primary auth key: {}", auth_key_ident);
    let mut confectionary = AuthConfectionary::new(auth_location, auth_key_ident, auth_key)?;
    match env::var("AUTH_ALT_KEYS") {
        Ok(var) => {
            for pair in var.split(",") {
                let pair: Vec<&str> = pair.split(":").collect();
                if pair.len() != 2 {
                    println!("{:#?}", pair);
                    bail!("couldn't parse keypair from AUTH_ALT_KEYS (expected 'ident:key' pairs separated by commas)");
                }
                info!("Loading alt auth key: {}", pair[0]);
                confectionary.add_keypair(pair[0].to_string(), pair[1].to_string())?;
            }
        }
        Err(_) => (),
    }
    Ok(confectionary)
}

/// Instantiate a new API server with a pooled database connection
pub fn server() -> Result<api_server::Server> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.");
    let confectionary = env_confectionary()?;
    Ok(api_server::Server {
        db_pool: pool,
        auth_confectionary: confectionary,
    })
}

/// Generates a server for testing. Calls an external bash script to generate a random postgres
/// database, which will be unique to this process but common across threads and connections. The
/// database will automagically get cleaned up (deleted) after 60 seconds.
/// Currently, start times are staggered by up to 200ms to prevent internal postgres concurrency
/// errors; if this fails run the tests serially (one at a time), which is slower but more robust.
/// CI should run tests serially.
pub fn test_server() -> Result<api_server::Server> {
    dotenv().ok();
    // sleep a bit so we don't have thundering herd collisions, resuliting in
    // "pg_extension_name_index" or "pg_proc_proname_args_nsp_index" or "pg_type_typname_nsp_index"
    // duplicate key violations.
    thread::sleep(time::Duration::from_millis(rand::thread_rng().gen_range(0, 200)));
    let pg_tmp = Command::new("./tests/pg_tmp.sh")
        .output()
        .expect("run ./tests/pg_tmp.sh to get temporary postgres DB");
    let database_url = String::from_utf8_lossy(&pg_tmp.stdout).to_string();
    env::set_var("DATABASE_URL", database_url);

    let mut server = server()?;
    server.auth_confectionary = AuthConfectionary::new_dummy();
    let conn = server.db_pool.get().expect("db_pool error");

    // run migrations; this is a fresh/bare database
    diesel_migrations::run_pending_migrations(&conn).unwrap();
    Ok(server)
}

// TODO: move this to bin/fatcatd

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
