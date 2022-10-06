use fatcat::auth::AuthConfectionary;
use fatcat::identifiers::FatcatId;
use fatcat::{auth, server};
use std::str::FromStr;

#[test]
fn test_old_token() {
    let server = server::create_test_server().unwrap();
    let conn = server.db_pool.get().expect("db_pool error");

    let admin_dev_token = "AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wNC0wNFQyMzozMjo0NloAAAYgrN3jjy0mgEqIydTFfsOLYSS55dz6Fh2d1CGMNQFLwcQ=";
    let editor_id = FatcatId::from_str("aaaaaaaaaaaabkvkaaaaaaaaai").unwrap();

    let c = AuthConfectionary::new(
        "dev.fatcat.wiki".to_string(),
        "20190101-dev-dummy-key".to_string(),
        "5555555555555555555555555555555555555555xms=",
    )
    .unwrap();
    let editor_row = c
        .parse_macaroon_token(&conn, &admin_dev_token, None)
        .unwrap();
    assert_eq!(editor_row.id, editor_id.to_uuid());
}

#[test]
fn test_macaroons() {
    // Test everything we can without connecting to database

    let c = AuthConfectionary::new_dummy();
    let editor_id = FatcatId::from_str("q3nouwy3nnbsvo3h5klxsx4a7y").unwrap();

    // create token w/o expiration
    c.create_token(editor_id, None).unwrap();

    // create token w/ expiration
    c.create_token(editor_id, Some(chrono::Duration::days(1)))
        .unwrap();
}

#[test]
fn test_auth_db() {
    // Test things that require database

    let server = server::create_test_server().unwrap();
    let conn = server.db_pool.get().expect("db_pool error");
    let c = AuthConfectionary::new_dummy();
    let editor_id = FatcatId::from_str("aaaaaaaaaaaabkvkaaaaaaaaae").unwrap();

    // create token
    let token = c.create_token(editor_id, None).unwrap();

    // verify token
    let editor_row = c.parse_macaroon_token(&conn, &token, None).unwrap();
    assert_eq!(editor_row.id, editor_id.to_uuid());

    // create token w/ expiration
    let token = c
        .create_token(editor_id, Some(chrono::Duration::days(1)))
        .unwrap();

    // verify token w/ expiration
    assert!(c.parse_macaroon_token(&conn, &token, None).is_ok());
    let editor_row = c.parse_macaroon_token(&conn, &token, None).unwrap();
    assert_eq!(editor_row.id, editor_id.to_uuid());

    // revoke token
    auth::revoke_tokens(&conn, editor_id).unwrap();

    // verification should fail
    // TODO: one-second slop breaks this; sleep for 1-2 seconds?
    //assert!(c.parse_macaroon_token(&conn, &token, None).is_err());

    // bad macaroon (not base64)
    assert!(c
        .parse_macaroon_token(&conn, "some string with spaces", None)
        .is_err());

    // bad macaroon (wrong key used to sign)
    let other_key = fatcat::auth::create_key();
    let c_other_key = AuthConfectionary::new(
        "test.fatcat.wiki".to_string(),
        "other-dummy".to_string(),
        &other_key,
    )
    .expect("creating dummy AuthConfectionary");
    let token_other_key = c_other_key.create_token(editor_id, None).unwrap();
    assert!(c_other_key
        .parse_macaroon_token(&conn, &token_other_key, None)
        .is_ok());
    assert!(c
        .parse_macaroon_token(&conn, &token_other_key, None)
        .is_err());
    assert!(c_other_key
        .parse_macaroon_token(&conn, &token, None)
        .is_err());

    // bad macaroon (wrong signing identifier)
    let c_other_location = AuthConfectionary::new(
        "test.fatcat.wiki".to_string(),
        "other-dummy-wrong".to_string(),
        &other_key,
    )
    .expect("creating dummy AuthConfectionary");
    let token_other_location = c_other_location.create_token(editor_id, None).unwrap();
    assert!(c_other_location
        .parse_macaroon_token(&conn, &token_other_location, None)
        .is_ok());
    assert!(c_other_key
        .parse_macaroon_token(&conn, &token_other_location, None)
        .is_err());
    assert!(c_other_location
        .parse_macaroon_token(&conn, &token_other_key, None)
        .is_err());
}
