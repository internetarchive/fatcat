extern crate diesel;
extern crate fatcat;
extern crate fatcat_api;
extern crate iron;
extern crate iron_test;
extern crate uuid;

use diesel::prelude::*;
use fatcat::api_helpers::*;
use fatcat::database_schema::*;
use iron::headers::ContentType;
use iron::mime::Mime;
use iron::{status, Headers};
use iron_test::{request, response};
use uuid::Uuid;

fn setup() -> (
    Headers,
    fatcat_api::router::Router,
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) {
    let server = fatcat::test_server().unwrap();
    let conn = server.db_pool.get().expect("db_pool error");
    let router = fatcat_api::router(server);
    let mut headers = Headers::new();
    let mime: Mime = "application/json".parse().unwrap();
    headers.set(ContentType(mime));
    (headers, router, conn)
}

fn check_response(
    resp: iron::IronResult<iron::response::Response>,
    want_status: status::Status,
    in_body: Option<&str>,
) {
    let resp = resp.unwrap();
    let status = resp.status;
    let body = response::extract_body_to_string(resp);
    println!("{}", body);
    assert_eq!(status, Some(want_status));
    if let Some(thing) = in_body {
        assert!(body.contains(thing));
    }
}

#[test]
fn test_entity_gets() {
    let (headers, router, _conn) = setup();

    check_response(
        request::get(
            "http://localhost:9411/v0/container/aaaaaaaaaaaaaeiraaaaaaaaai",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("Trivial Results"),
    );

    // Check revision encoding
    check_response(
        request::get(
            "http://localhost:9411/v0/container/aaaaaaaaaaaaaeiraaaaaaaaai",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("00000000-0000-0000-1111-fff000000002"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/creator/aaaaaaaaaaaaaircaaaaaaaaae",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("Grace Hopper"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/file/aaaaaaaaaaaaamztaaaaaaaaai",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("archive.org"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/release/aaaaaaaaaaaaarceaaaaaaaaai",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("bigger example"),
    );

    // expand keyword
    check_response(
        request::get(
            "http://localhost:9411/v0/release/aaaaaaaaaaaaarceaaaaaaaaai?expand=all",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("MySpace Blog"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/work/aaaaaaaaaaaaavkvaaaaaaaaai",
            headers.clone(),
            &router,
        ),
        status::Ok,
        None,
    );
}

#[test]
fn test_entity_404() {
    let (headers, router, _conn) = setup();

    check_response(
        request::get(
            "http://localhost:9411/v0/creator/aaaaaaaaaaaaairceeeeeeeeee",
            headers.clone(),
            &router,
        ),
        status::NotFound,
        None,
    );
}

#[test]
fn test_entity_history() {
    let (headers, router, _conn) = setup();

    check_response(
        request::get(
            "http://localhost:9411/v0/container/aaaaaaaaaaaaaeiraaaaaaaaai/history",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("changelog"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/creator/aaaaaaaaaaaaaircaaaaaaaaae/history",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("changelog"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/file/aaaaaaaaaaaaamztaaaaaaaaai/history",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("changelog"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/release/aaaaaaaaaaaaarceaaaaaaaaai/history",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("changelog"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/work/aaaaaaaaaaaaavkvaaaaaaaaai/history",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("changelog"),
    );
}

#[test]
fn test_lookups() {
    let (headers, router, _conn) = setup();

    check_response(
        request::get(
            "http://localhost:9411/v0/container/lookup?issnl=1234-5678",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("Journal of Trivial Results"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/creator/lookup?orcid=0000-0003-2088-7465",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("Christine Moran"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/file/lookup?sha1=7d97e98f8af710c7e7fe703abc8f639e0ee507c4",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("robots.txt"),
    );
}

#[test]
fn test_reverse_lookups() {
    let (headers, router, _conn) = setup();

    check_response(
        request::get(
            "http://localhost:9411/v0/creator/aaaaaaaaaaaaaircaaaaaaaaai/releases",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("bigger example"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/release/aaaaaaaaaaaaarceaaaaaaaaai/files",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("7d97e98f8af710c7e7fe703abc8f639e0ee507c4"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/work/aaaaaaaaaaaaavkvaaaaaaaaai/releases",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("bigger example"),
    );
}

#[test]
fn test_post_container() {
    let (headers, router, _conn) = setup();

    check_response(
        request::post(
            "http://localhost:9411/v0/container",
            headers,
            r#"{"name": "test journal"}"#,
            &router,
        ),
        status::Created,
        None,
    ); // TODO: "test journal"
}

#[test]
fn test_post_batch_container() {
    let (headers, router, _conn) = setup();

    check_response(
        request::post(
            "http://localhost:9411/v0/container/batch",
            headers,
            r#"[{"name": "test journal"}, {"name": "another test journal"}]"#,
            &router,
        ),
        status::Created,
        None,
    ); // TODO: "test journal"
}

#[test]
fn test_post_creator() {
    let (headers, router, _conn) = setup();

    check_response(
        request::post(
            "http://localhost:9411/v0/creator",
            headers,
            r#"{"display_name": "some person"}"#,
            &router,
        ),
        status::Created,
        None,
    );
}

#[test]
fn test_post_file() {
    let (headers, router, conn) = setup();

    check_response(
        request::post(
            "http://localhost:9411/v0/file",
            headers.clone(),
            r#"{ }"#,
            &router,
        ),
        status::Created,
        None,
    );

    check_response(
        request::post(
            "http://localhost:9411/v0/file",
            headers.clone(),
            r#"{"size": 76543,
                "sha1": "f0000000000000008b7eb2a93e6d0440c1f3e7f8",
                "md5": "0b6d347b01d437a092be84c2edfce72c",
                "sha256": "a77e4c11a57f1d757fca5754a8f83b5d4ece49a2d28596889127c1a2f3f28832",
                "urls": [
                    {"url": "http://archive.org/asdf.txt", "rel": "web" },
                    {"url": "http://web.archive.org/2/http://archive.org/asdf.txt", "rel": "webarchive" }
                ],
                "mimetype": "application/pdf",
                "releases": [
                    "aaaaaaaaaaaaarceaaaaaaaaae",
                    "aaaaaaaaaaaaarceaaaaaaaaai"
                ],
                "extra": { "source": "speculation" }
                }"#,
            &router,
        ),
        status::Created,
        None,
    );

    let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap();
    let editgroup_id = get_or_create_editgroup(editor_id, &conn).unwrap();
    check_response(
        request::post(
            &format!(
                "http://localhost:9411/v0/editgroup/{}/accept",
                uuid2fcid(&editgroup_id)
            ),
            headers.clone(),
            "",
            &router,
        ),
        status::Ok,
        None,
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/file/lookup?sha1=f0000000000000008b7eb2a93e6d0440c1f3e7f8",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("web.archive.org/2/http"),
    );
}

#[test]
fn test_post_release() {
    let (headers, router, _conn) = setup();

    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            // TODO: target_release_id
            r#"{"title": "secret minimal paper",
                "release_type": "journal-article",
                "work_id": "aaaaaaaaaaaaavkvaaaaaaaaae"
                }"#,
            &router,
        ),
        status::Created,
        None,
    ); // TODO: "secret paper"

    // No work_id supplied (auto-created)
    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            // TODO: target_release_id
            r#"{"title": "secret minimal paper the second",
                "release_type": "journal-article"
                }"#,
            &router,
        ),
        status::Created,
        None,
    );

    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers,
            // TODO: target_release_id
            r#"{"title": "secret paper",
                "release_type": "journal-article",
                "doi": "10.1234/abcde.781231231239",
                "pmid": "54321",
                "pmcid": "PMC12345",
                "wikidata_qid": "Q12345",
                "core_id": "7890",
                "volume": "439",
                "issue": "IV",
                "pages": "1-399",
                "work_id": "aaaaaaaaaaaaavkvaaaaaaaaai",
                "container_id": "aaaaaaaaaaaaaeiraaaaaaaaae",
                "refs": [{
                        "index": 3,
                        "raw": "just a string"
                    },{
                        "raw": "just a string"
                    }],
                "contribs": [{
                        "index": 1,
                        "raw_name": "textual description of contributor (aka, name)",
                        "creator_id": "aaaaaaaaaaaaaircaaaaaaaaae",
                        "contrib_type": "author"
                    },{
                        "raw_name": "shorter"
                    }],
                "extra": { "source": "speculation" }
                }"#,
            &router,
        ),
        status::Created,
        None,
    ); // TODO: "secret paper"
}

#[test]
fn test_post_work() {
    let (headers, router, _conn) = setup();

    check_response(
        request::post(
            "http://localhost:9411/v0/work",
            headers.clone(),
            // TODO: target_work_id
            r#"{
                "extra": { "source": "speculation" }
            }"#,
            &router,
        ),
        status::Created,
        None,
    );
}

#[test]
fn test_accept_editgroup() {
    let (headers, router, conn) = setup();

    let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap();
    let editgroup_id = get_or_create_editgroup(editor_id, &conn).unwrap();

    let c: i64 = container_ident::table
        .filter(container_ident::is_live.eq(false))
        .count()
        .get_result(&conn)
        .unwrap();
    assert_eq!(c, 0);
    let c: i64 = changelog::table
        .filter(changelog::editgroup_id.eq(editgroup_id))
        .count()
        .get_result(&conn)
        .unwrap();
    assert_eq!(c, 0);

    check_response(
        request::post(
            "http://localhost:9411/v0/container",
            headers.clone(),
            &format!(
                "{{\"name\": \"test journal 1\", \"editgroup_id\": \"{}\"}}",
                uuid2fcid(&editgroup_id)
            ),
            &router,
        ),
        status::Created,
        None,
    );
    check_response(
        request::post(
            "http://localhost:9411/v0/container",
            headers.clone(),
            &format!(
                "{{\"name\": \"test journal 2\", \"editgroup_id\": \"{}\"}}",
                uuid2fcid(&editgroup_id)
            ),
            &router,
        ),
        status::Created,
        None,
    );

    let c: i64 = container_ident::table
        .filter(container_ident::is_live.eq(false))
        .count()
        .get_result(&conn)
        .unwrap();
    assert_eq!(c, 2);

    check_response(
        request::get(
            &format!(
                "http://localhost:9411/v0/editgroup/{}",
                uuid2fcid(&editgroup_id)
            ),
            headers.clone(),
            &router,
        ),
        status::Ok,
        None,
    );

    check_response(
        request::post(
            &format!(
                "http://localhost:9411/v0/editgroup/{}/accept",
                uuid2fcid(&editgroup_id)
            ),
            headers.clone(),
            "",
            &router,
        ),
        status::Ok,
        None,
    );

    let c: i64 = container_ident::table
        .filter(container_ident::is_live.eq(false))
        .count()
        .get_result(&conn)
        .unwrap();
    assert_eq!(c, 0);
    let c: i64 = changelog::table
        .filter(changelog::editgroup_id.eq(editgroup_id))
        .count()
        .get_result(&conn)
        .unwrap();
    assert_eq!(c, 1);
}

#[test]
fn test_changelog() {
    let (headers, router, _conn) = setup();

    check_response(
        request::get(
            "http://localhost:9411/v0/changelog",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("editgroup_id"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/changelog/1",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("files"),
    );
}

#[test]
fn test_stats() {
    let (headers, router, _conn) = setup();

    check_response(
        request::get("http://localhost:9411/v0/stats", headers.clone(), &router),
        status::Ok,
        Some("merged_editgroups"),
    );
    check_response(
        request::get(
            "http://localhost:9411/v0/stats?more=yes",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("merged_editgroups"),
    );
}

#[test]
fn test_400() {
    let (headers, router, _conn) = setup();

    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers,
            r#"{"title": "secret paper",
                "release_type": "journal-article",
                "doi": "10.1234/abcde.781231231239",
                "volume": "439",
                "issue": "IV",
                "pages": "1-399",
                "work_id": "aaaaaaaaaaaaavkvaaaaaaaaae",
                "container_id": "aaaaaaaaaaaaaeiraaaaaae",
                "refs": [{
                        "index": 3,
                        "raw": "just a string"
                    },{
                        "raw": "just a string"
                    }],
                "contribs": [{
                        "index": 1,
                        "raw_name": "textual description of contributor (aka, name)",
                        "creator_id": "aaaaaaaaaaaaaircaaaaaaaaae",
                        "contrib_type": "author"
                    },{
                        "raw_name": "shorter"
                    }],
                "extra": { "source": "speculation" }
                }"#,
            &router,
        ),
        status::BadRequest,
        None,
    );
}

#[test]
fn test_edit_gets() {
    let (headers, router, _conn) = setup();

    check_response(
        request::get(
            "http://localhost:9411/v0/editor/aaaaaaaaaaaabkvkaaaaaaaaae",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("admin"),
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/editor/aaaaaaaaaaaabkvkaaaaaaaaae/changelog",
            headers.clone(),
            &router,
        ),
        status::Ok,
        None,
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/editgroup/aaaaaaaaaaaabo53aaaaaaaaae",
            headers.clone(),
            &router,
        ),
        status::Ok,
        None,
    );
}

#[test]
fn test_bad_external_idents() {
    let (headers, router, _conn) = setup();

    // Bad wikidata QID
    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            r#"{"title": "secret paper",
                "wikidata_qid": "P12345"
                }"#,
            &router,
        ),
        status::BadRequest,
        Some("Wikidata QID"),
    );
    check_response(
        request::post(
            "http://localhost:9411/v0/container",
            headers.clone(),
            r#"{"name": "my journal",
                "wikidata_qid": "P12345"
                }"#,
            &router,
        ),
        status::BadRequest,
        Some("Wikidata QID"),
    );
    check_response(
        request::post(
            "http://localhost:9411/v0/creator",
            headers.clone(),
            r#"{"display_name": "some body",
                "wikidata_qid": "P12345"
                }"#,
            &router,
        ),
        status::BadRequest,
        Some("Wikidata QID"),
    );

    // Bad PMCID
    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            r#"{"title": "secret paper",
                "pmcid": "12345"
                }"#,
            &router,
        ),
        status::BadRequest,
        Some("PMCID"),
    );

    // Bad PMID
    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            r#"{"title": "secret paper",
                "pmid": "not-a-number"
                }"#,
            &router,
        ),
        status::BadRequest,
        Some("PMID"),
    );

    // Bad DOI
    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            r#"{"title": "secret paper",
                "doi": "asdf"
                }"#,
            &router,
        ),
        status::BadRequest,
        Some("DOI"),
    );

    // Good identifiers
    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            r#"{"title": "secret paper",
                "doi": "10.1234/abcde.781231231239",
                "pmid": "54321",
                "pmcid": "PMC12345",
                "wikidata_qid": "Q12345"
                }"#,
            &router,
        ),
        status::Created,
        None,
    );
}

#[test]
fn test_abstracts() {
    let (headers, router, conn) = setup();

    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            r#"{"title": "some paper",
                "doi": "10.1234/iiiiiii",
                "abstracts": [
                  {"lang": "zh",
                   "mimetype": "text/plain",
                   "content": "some rando abstract 24iu3i25u2" },
                  {"lang": "en",
                   "mimetype": "application/xml+jats",
                   "content": "some other abstract 99139405" }
                ]
                }"#,
            &router,
        ),
        status::Created,
        None,
    );

    // Same abstracts; checking that re-inserting works
    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            r#"{"title": "some paper again",
                "abstracts": [
                  {"lang": "zh",
                   "mimetype": "text/plain",
                   "content": "some rando abstract 24iu3i25u2" },
                  {"lang": "en",
                   "mimetype": "application/xml+jats",
                   "content": "some other abstract 99139405" }
                ]
                }"#,
            &router,
        ),
        status::Created,
        None,
    );

    let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap();
    let editgroup_id = get_or_create_editgroup(editor_id, &conn).unwrap();
    check_response(
        request::post(
            &format!(
                "http://localhost:9411/v0/editgroup/{}/accept",
                uuid2fcid(&editgroup_id)
            ),
            headers.clone(),
            "",
            &router,
        ),
        status::Ok,
        None,
    );

    check_response(
        request::get(
            "http://localhost:9411/v0/release/lookup?doi=10.1234/iiiiiii",
            headers.clone(),
            &router,
        ),
        status::Ok,
        // SHA-1 of first abstract string (with no trailing newline)
        Some("65c171bd8c968e12ede25ad95f02cd4b2ce9db52"),
    );
    check_response(
        request::get(
            "http://localhost:9411/v0/release/lookup?doi=10.1234/iiiiiii",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("99139405"),
    );
    check_response(
        request::get(
            "http://localhost:9411/v0/release/lookup?doi=10.1234/iiiiiii",
            headers.clone(),
            &router,
        ),
        status::Ok,
        Some("24iu3i25u2"),
    );
}

#[test]
fn test_contribs() {
    let (headers, router, conn) = setup();

    check_response(
        request::post(
            "http://localhost:9411/v0/release",
            headers.clone(),
            r#"{"title": "some paper",
                "doi": "10.1234/iiiiiii",
                "contribs": [{
                        "index": 1,
                        "raw_name": "textual description of contributor (aka, name)",
                        "creator_id": "aaaaaaaaaaaaaircaaaaaaaaae",
                        "contrib_type": "author",
                        "extra": {"key": "value 28328424942"}
                    },{
                        "raw_name": "shorter"
                    }]
                }"#,
            &router,
        ),
        status::Created,
        None,
    );

    let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001").unwrap();
    let editgroup_id = get_or_create_editgroup(editor_id, &conn).unwrap();
    check_response(
        request::post(
            &format!(
                "http://localhost:9411/v0/editgroup/{}/accept",
                uuid2fcid(&editgroup_id)
            ),
            headers.clone(),
            "",
            &router,
        ),
        status::Ok,
        None,
    );
}

#[test]
fn test_post_batch_autoaccept() {
    let (headers, router, _conn) = setup();

    // "true"
    check_response(
        request::post(
            "http://localhost:9411/v0/container/batch?autoaccept=true",
            headers.clone(),
            r#"[{"name": "test journal"}, {"name": "another test journal"}]"#,
            &router,
        ),
        status::Created,
        None,
    );

    // "n"
    check_response(
        request::post(
            "http://localhost:9411/v0/container/batch?autoaccept=n",
            headers.clone(),
            r#"[{"name": "test journal"}, {"name": "another test journal"}]"#,
            &router,
        ),
        status::Created,
        None,
    );

    // editgroup
    check_response(
        request::post(
            "http://localhost:9411/v0/container/batch?autoaccept=yes&editgroup=asdf",
            headers.clone(),
            r#"[{"name": "test journal"}, {"name": "another test journal"}]"#,
            &router,
        ),
        status::BadRequest,
        None,
    );
}
