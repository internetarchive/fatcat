extern crate fatcat;
extern crate fatcat_api;
extern crate iron;
extern crate iron_test;

use iron::{status, Headers};
use iron_test::{request, response};

#[test]
fn test_basics() {
    let server = fatcat::server().unwrap();
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
