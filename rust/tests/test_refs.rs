use diesel::prelude::*;
use fatcat::database_models::*;
use fatcat::database_schema::*;
use fatcat::editing::{accept_editgroup, make_edit_context};
use fatcat::entity_crud::{EntityCrud, HideFlags};
use fatcat::identifiers::FatcatId;
use fatcat::server;
use fatcat_api_spec::models::*;
use std::str::FromStr;
use uuid::Uuid;

mod helpers;

#[test]
fn test_refs_blob() {
    let server = server::create_test_server().unwrap();
    let conn = server.db_pool.get().expect("db_pool error");
    let editor_id = FatcatId::from_str(helpers::TEST_ADMIN_EDITOR_ID).unwrap();
    let editgroup_id = helpers::quick_editgroup(&conn);
    let edit_context = make_edit_context(&conn, editor_id, Some(editgroup_id), false).unwrap();

    // this release entity should be unchanged after being inserted/fetched
    let mut r1 = ReleaseEntity::new();
    r1.title = Some("release-test hashes".to_string());
    r1.refs = Some(vec![
        ReleaseRef {
            index: Some(0),
            target_release_id: None,
            extra: None,
            key: Some("one".to_string()),
            year: Some(1932),
            container_name: Some("bogus container".to_string()),
            title: Some("first bogus paper".to_string()),
            locator: Some("p100".to_string()),
        },
        ReleaseRef {
            index: Some(1),
            target_release_id: Some("aaaaaaaaaaaaarceaaaaaaaaai".to_string()),
            extra: None,
            key: Some("one".to_string()),
            year: Some(2032),
            container_name: Some("bogus other container".to_string()),
            title: Some("second bogus paper".to_string()),
            locator: Some("p200".to_string()),
        },
    ]);

    // this release entity should have the same hash as r1. the indexes will change after fetching,
    // but otherwise the fetched refs should be the same as the r1 fetched results.
    let mut r2 = r1.clone();
    r2.refs = Some(vec![
        ReleaseRef {
            index: None,
            target_release_id: None,
            extra: None,
            key: Some("one".to_string()),
            year: Some(1932),
            container_name: Some("bogus container".to_string()),
            title: Some("first bogus paper".to_string()),
            locator: Some("p100".to_string()),
        },
        ReleaseRef {
            index: Some(99),
            target_release_id: Some("aaaaaaaaaaaaarceaaaaaaaaai".to_string()),
            extra: None,
            key: Some("one".to_string()),
            year: Some(2032),
            container_name: Some("bogus other container".to_string()),
            title: Some("second bogus paper".to_string()),
            locator: Some("p200".to_string()),
        },
    ]);

    // this release entity has different ref *targets* and indexes, but should still have the same
    // refs_blob hashes as r1/r2.
    let mut r3 = r1.clone();
    r3.refs = Some(vec![
        ReleaseRef {
            index: Some(1),
            target_release_id: Some("aaaaaaaaaaaaarceaaaaaaaaae".to_string()),
            extra: None,
            key: Some("one".to_string()),
            year: Some(1932),
            container_name: Some("bogus container".to_string()),
            title: Some("first bogus paper".to_string()),
            locator: Some("p100".to_string()),
        },
        ReleaseRef {
            index: Some(1),
            target_release_id: Some("aaaaaaaaaaaaarceaaaaaaaaam".to_string()),
            extra: None,
            key: Some("one".to_string()),
            year: Some(2032),
            container_name: Some("bogus other container".to_string()),
            title: Some("second bogus paper".to_string()),
            locator: Some("p200".to_string()),
        },
    ]);

    // this one is obviously just plain different (hashes shouldn't match)
    let mut r4 = r1.clone();
    r4.refs = Some(vec![ReleaseRef {
        index: Some(1),
        target_release_id: Some("aaaaaaaaaaaaarceaaaaaaaaae".to_string()),
        extra: None,
        key: Some("one".to_string()),
        year: Some(1932),
        container_name: Some("bogus container".to_string()),
        title: Some("first bogus paper".to_string()),
        locator: Some("p100".to_string()),
    }]);

    let edit1 = r1.db_create(&conn, &edit_context).unwrap();
    let edit2 = r2.db_create(&conn, &edit_context).unwrap();
    let edit3 = r3.db_create(&conn, &edit_context).unwrap();
    let edit4 = r4.db_create(&conn, &edit_context).unwrap();

    let r1b = ReleaseEntity::db_get(&conn, edit1.ident_id.into(), HideFlags::none()).unwrap();
    let r2b = ReleaseEntity::db_get(&conn, edit2.ident_id.into(), HideFlags::none()).unwrap();
    let r3b = ReleaseEntity::db_get(&conn, edit3.ident_id.into(), HideFlags::none()).unwrap();
    let r4b = ReleaseEntity::db_get(&conn, edit4.ident_id.into(), HideFlags::none()).unwrap();
    assert_eq!(r1b.refs, r1.refs);
    assert_eq!(r1b.refs, r2b.refs);
    assert_ne!(r1b.refs, r3b.refs);
    assert_ne!(r1b.refs, r4b.refs);

    let r1_row: ReleaseRevRow = release_rev::table
        .find(Uuid::from_str(&r1b.revision.clone().unwrap()).unwrap())
        .get_result(&conn)
        .unwrap();
    let r2_row: ReleaseRevRow = release_rev::table
        .find(Uuid::from_str(&r2b.revision.unwrap()).unwrap())
        .get_result(&conn)
        .unwrap();
    let r3_row: ReleaseRevRow = release_rev::table
        .find(Uuid::from_str(&r3b.revision.clone().unwrap()).unwrap())
        .get_result(&conn)
        .unwrap();
    let r4_row: ReleaseRevRow = release_rev::table
        .find(Uuid::from_str(&r4b.revision.unwrap()).unwrap())
        .get_result(&conn)
        .unwrap();
    assert_eq!(r1_row.refs_blob_sha1, r2_row.refs_blob_sha1);
    assert_eq!(r1_row.refs_blob_sha1, r3_row.refs_blob_sha1);
    assert_ne!(r1_row.refs_blob_sha1, r4_row.refs_blob_sha1);

    // ensure that SHA1 hashing is stable over time (as much as possible!)
    assert_eq!(
        r1_row.refs_blob_sha1,
        Some("4e38812fbf99e00e0cb648896e9f7a9d58c5ab23".to_string())
    );

    // update r1 with new target_idents (r3); SHA1 row still shouldn't change
    accept_editgroup(&conn, editgroup_id).unwrap();
    let editgroup_id = helpers::quick_editgroup(&conn);
    let edit_context = make_edit_context(&conn, editor_id, Some(editgroup_id), false).unwrap();

    let _edit4 = r3b
        .db_update(&conn, &edit_context, edit1.ident_id.into())
        .unwrap();
    let r1c = ReleaseEntity::db_get(&conn, edit1.ident_id.into(), HideFlags::none()).unwrap();
    let r1c_row: ReleaseRevRow = release_rev::table
        .find(Uuid::from_str(&r1c.revision.unwrap()).unwrap())
        .get_result(&conn)
        .unwrap();
    assert_eq!(r1_row.refs_blob_sha1, r1c_row.refs_blob_sha1);
}
