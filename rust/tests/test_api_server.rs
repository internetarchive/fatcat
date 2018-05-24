extern crate diesel;
extern crate fatcat;
extern crate fatcat_api;
extern crate iron;
extern crate iron_test;

use diesel::prelude::*;
use fatcat::api_helpers::*;
use fatcat::database_schema::*;
use iron::headers::ContentType;
use iron::mime::Mime;
use iron::{status, Headers};
use iron_test::{request, response};

#[test]
fn test_basics() {
    let server = fatcat::test_server().unwrap();
    let router = fatcat_api::router(server);

    let response = request::get(
        "http://localhost:9411/v0/creator/f1f046a3-45c9-4b99-adce-000000000001",
        Headers::new(),
        &router,
    ).unwrap();
    assert_eq!(response.status, Some(status::Ok));
    let body = response::extract_body_to_string(response);
    assert!(body.contains("Grace Hopper"));

    let response = request::get(
        "http://localhost:9411/v0/creator/f1f046a3-45c9-4b99-adce-999999999999",
        Headers::new(),
        &router,
    ).unwrap();
    assert_eq!(response.status, Some(status::NotFound));
}

#[test]
fn test_lookups() {
    let server = fatcat::test_server().unwrap();
    let router = fatcat_api::router(server);

    let response = request::get(
        "http://localhost:9411/v0/container/lookup?issn=1234-5678",
        Headers::new(),
        &router,
    ).unwrap();
    assert_eq!(response.status, Some(status::Ok));
    let body = response::extract_body_to_string(response);
    assert!(body.contains("Journal of Trivial Results"));

    let response = request::get(
        "http://localhost:9411/v0/creator/lookup?orcid=0000-0003-2088-7465",
        Headers::new(),
        &router,
    ).unwrap();
    assert_eq!(response.status, Some(status::Ok));
    let body = response::extract_body_to_string(response);
    assert!(body.contains("Christine Moran"));
}

#[test]
fn test_post_container() {
    let server = fatcat::test_server().unwrap();
    let router = fatcat_api::router(server);
    let mut headers = Headers::new();
    let mime: Mime = "application/json".parse().unwrap();
    headers.set(ContentType(mime));

    let response = request::post(
        "http://localhost:9411/v0/container",
        headers,
        r#"{"name": "test journal"}"#,
        &router,
    ).unwrap();
    assert_eq!(response.status, Some(status::Created));
    let body = response::extract_body_to_string(response);
    println!("{}", body);
    //assert!(body.contains("test journal"));
}

#[test]
fn test_accept_editgroup() {
    let server = fatcat::test_server().unwrap();
    let conn = server.db_pool.get().expect("db_pool error");
    let router = fatcat_api::router(server);
    let mut headers = Headers::new();
    let mime: Mime = "application/json".parse().unwrap();
    headers.set(ContentType(mime));

    let editgroup_id = get_or_create_editgroup(1, &conn).unwrap();

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

    let response = request::post(
        "http://localhost:9411/v0/container",
        headers.clone(),
        &format!(
            "{{\"name\": \"test journal 1\", \"editgroup_id\": {}}}",
            editgroup_id
        ),
        &router,
    ).unwrap();
    assert_eq!(response.status, Some(status::Created));
    let response = request::post(
        "http://localhost:9411/v0/container",
        headers.clone(),
        &format!(
            "{{\"name\": \"test journal 2\", \"editgroup_id\": {}}}",
            editgroup_id
        ),
        &router,
    ).unwrap();
    assert_eq!(response.status, Some(status::Created));

    let c: i64 = container_ident::table
        .filter(container_ident::is_live.eq(false))
        .count()
        .get_result(&conn)
        .unwrap();
    assert_eq!(c, 2);

    let response = request::post(
        &format!("http://localhost:9411/v0/editgroup/{}/accept", editgroup_id),
        headers.clone(),
        "",
        &router,
    ).unwrap();
    assert_eq!(response.status, Some(status::Ok));

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
