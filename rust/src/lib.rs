#![allow(proc_macro_derive_resolution_fallback)]
#![recursion_limit = "128"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod auth;
pub mod database_models;
pub mod database_schema; // only public for tests
pub mod editing;
mod endpoint_handlers;
mod endpoints;
pub mod entity_crud;
pub mod errors;
pub mod identifiers;
pub mod server;

// TODO: will probably remove these as a public export?
pub use crate::server::{create_server, create_test_server};
