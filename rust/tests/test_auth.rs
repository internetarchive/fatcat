
extern crate fatcat;
extern crate uuid;
extern crate chrono;

use std::str::FromStr;
use chrono::prelude::*;
use fatcat::auth::*;
use fatcat::api_helpers::*;

#[test]
fn test_macaroons() {
    // Test everything we can without connecting to database
 
    let c = fatcat::auth::AuthConfectionary::new_dummy();
    let editor_id = FatCatId::from_str("q3nouwy3nnbsvo3h5klxsx4a7y").unwrap();

    // create token w/o expiration
    c.create_token(editor_id, None).unwrap();

    // create token w/ expiration
    let tomorrow = Utc::now() + chrono::Duration::days(1);
    c.create_token(editor_id, Some(tomorrow)).unwrap();
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
    let editor_row = c.parse_macaroon_token(&conn, &token).unwrap();
    assert_eq!(editor_row.id, editor_id.to_uuid());
    
    // revoke token
    revoke_tokens(&conn, editor_id);

    // verification should fail
    assert!(c.parse_macaroon_token(&conn, &token).is_err());
}
