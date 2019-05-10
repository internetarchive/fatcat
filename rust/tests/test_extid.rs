
use diesel::prelude::*;
use fatcat::database_models::*;
use fatcat::database_schema::*;
use fatcat::server;
use diesel::insert_into;
use std::str::FromStr;
use uuid::Uuid;

mod helpers;

#[test]
fn test_extid_uniq() {
    // This test ensures that the SQL UNIQ constraint is working
    let server = server::create_test_server().unwrap();
    let conn = server.db_pool.get().expect("db_pool error");

    let resp = insert_into(release_rev_extid::table)
        .values(ReleaseExtidRow {
            release_rev: Uuid::from_str("00000000-0000-0000-4444-FFF000000002").unwrap(),
            extid_type: "arxiv".to_string(),
            value: "1905.03769v1".to_string(),
        })
        .execute(&conn);

    assert!(resp.is_err());

    let resp = insert_into(release_rev_extid::table)
        .values(ReleaseExtidRow {
            release_rev: Uuid::from_str("00000000-0000-0000-4444-FFF000000002").unwrap(),
            extid_type: "arxiv_blah".to_string(),
            value: "1905.03769v1".to_string(),
        })
        .execute(&conn);

    assert!(resp.is_ok());
}
