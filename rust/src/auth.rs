//! Editor bearer token authentication

use swagger::auth::{AuthData, Authorization, Scopes};
//use macaroon::{Macaroon, Verifier};

use std::collections::BTreeSet;
//use database_models::*;
//use database_schema::*;
use api_helpers::*;
use chrono;
//use diesel;
use iron;
//use diesel::prelude::*;
use errors::*;
//use serde_json;
//use std::str::FromStr;
//use uuid::Uuid;

#[derive(Debug)]
pub struct OpenAuthMiddleware;

impl OpenAuthMiddleware {
    /// Create a middleware that authorizes with the configured subject.
    pub fn new() -> OpenAuthMiddleware {
        OpenAuthMiddleware
    }
}
impl iron::middleware::BeforeMiddleware for OpenAuthMiddleware {
    fn before(&self, req: &mut iron::Request) -> iron::IronResult<()> {
        req.extensions.insert::<Authorization>(Authorization {
            subject: "undefined".to_string(),
            scopes: Scopes::All,
            issuer: None,
        });
        Ok(())
    }
}

#[derive(Debug)]
pub struct MacaroonAuthMiddleware;

impl MacaroonAuthMiddleware {

    pub fn new() -> MacaroonAuthMiddleware {
        MacaroonAuthMiddleware
    }
}

impl iron::middleware::BeforeMiddleware for MacaroonAuthMiddleware {
    fn before(&self, req: &mut iron::Request) -> iron::IronResult<()> {

        let res: Option<(String, Vec<String>)> = match req.extensions.get::<AuthData>() {
            Some(AuthData::ApiKey(header)) => {
                let header: Vec<String> = header.split_whitespace().map(|s| s.to_string()).collect();
                // TODO: error types
                assert!(header.len() == 2);
                assert!(header[0] == "Bearer");
                parse_macaroon_token(&header[1]).expect("valid macaroon")
            },
            None => None,
            _ => panic!("valid auth header, or none")
        };
        if let Some((editor_id, scopes)) = res {
            let mut scope_set = BTreeSet::new();
            for s in scopes {
                scope_set.insert(s);
            }
            req.extensions.insert::<Authorization>(Authorization {
                subject: editor_id,
                scopes: Scopes::Some(scope_set),
                issuer: None,
            });
        };
        Ok(())
    }
}

// DUMMY: parse macaroon
pub fn parse_macaroon_token(s: &str) -> Result<Option<(String,Vec<String>)>> {
    Ok(Some(("some_editor_id".to_string(), vec![])))
}

pub fn print_editors() -> Result<()>{
    unimplemented!();
    // iterate over all editors. format id, print flags, auth_epoch
}

pub fn create_editor(username: String, is_admin: bool, is_bot: bool) -> Result<()> { // TODO: EditorRow or something
    unimplemented!();
}

pub fn create_token(editor_id: FatCatId, expires: Option<chrono::NaiveDateTime>) -> Result<String> {
    unimplemented!();
}

pub fn inspect_token(token: &str) -> Result<()> {
    unimplemented!();
}

pub fn revoke_tokens(editor_id: FatCatId) -> Result<()>{
    unimplemented!();
}

pub fn revoke_tokens_everyone() -> Result<u64> {
    unimplemented!();
}
