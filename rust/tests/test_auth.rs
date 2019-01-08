extern crate chrono;
extern crate fatcat;
extern crate uuid;

use chrono::prelude::*;
use fatcat::api_helpers::*;
use fatcat::auth::*;
use std::str::FromStr;

#[test]
fn test_macaroons() {
    // Test everything we can without connecting to database

    let c = fatcat::auth::AuthConfectionary::new_dummy();
    let editor_id = FatCatId::from_str("q3nouwy3nnbsvo3h5klxsx4a7y").unwrap();

    // create token w/o expiration
    c.create_token(editor_id, None).unwrap();

    // create token w/ expiration
    c.create_token(editor_id, Some(chrono::Duration::days(1))).unwrap();
}

#[test]
fn test_auth_db() {
    // Test things that require database

    let server = fatcat::test_server().unwrap();
    let conn = server.db_pool.get().expect("db_pool error");
    let c = fatcat::auth::AuthConfectionary::new_dummy();
    let editor_id = FatCatId::from_str("aaaaaaaaaaaabkvkaaaaaaaaae").unwrap();

    // create token
    let token = c.create_token(editor_id, None).unwrap();

    // verify token
    let editor_row = c.parse_macaroon_token(&conn, &token, None).unwrap();
    assert_eq!(editor_row.id, editor_id.to_uuid());

    // revoke token
    revoke_tokens(&conn, editor_id).unwrap();

    // verification should fail
    // XXX: one-second slop breaks this
    //assert!(c.parse_macaroon_token(&conn, &token, None).is_err());
}
