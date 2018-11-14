
extern crate diesel;
extern crate fatcat;
extern crate fatcat_api_spec;
extern crate uuid;
extern crate iron;
extern crate iron_test;

use std::{thread, time};
use self::iron_test::response;
use iron::{status, Iron, Listening, Headers};
use iron::headers::ContentType;
use iron::mime::Mime;
use fatcat_api_spec::client::Client;

// A current problem with this method is that if the test fails (eg, panics, assert fails), the
// server never gets closed, and the server thread hangs forever.
// One workaround might be to invert the function, take a closure, capture the panic/failure, and
// cleanup.
pub fn setup_client() -> (
    Client,
    Listening,
) {
    let server = fatcat::test_server().unwrap();
    let router = fatcat_api_spec::router(server);
    // this is an unfortunate hack for testings seeming to fail in CI
    thread::sleep(time::Duration::from_millis(100));
    let iron_server = Iron::new(router)
        .http("localhost:9144")
        .expect("Failed to start HTTP server");

    let client = Client::try_new_http("http://localhost:9144").unwrap();
    (client, iron_server)
}

pub fn setup_http() -> (
    Headers,
    fatcat_api_spec::router::Router,
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) {
    let server = fatcat::test_server().unwrap();
    let conn = server.db_pool.get().expect("db_pool error");
    let router = fatcat_api_spec::router(server);
    let mut headers = Headers::new();
    let mime: Mime = "application/json".parse().unwrap();
    headers.set(ContentType(mime));
    (headers, router, conn)
}

pub fn check_http_response(
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
