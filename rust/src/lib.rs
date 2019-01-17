#![allow(proc_macro_derive_resolution_fallback)]
#![recursion_limit = "128"]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

pub mod auth;
pub mod database_models;
pub mod database_schema; // only public for tests
pub mod editing;
pub mod editing_crud;
mod endpoint_handlers;
mod endpoints;
pub mod entity_crud;
pub mod errors;
pub mod identifiers;
pub mod server;
