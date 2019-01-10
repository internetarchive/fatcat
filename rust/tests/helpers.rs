use fatcat::auth::MacaroonAuthMiddleware;
use fatcat::identifiers::FatcatId;
use fatcat::server;
use fatcat_api_spec::client::Client;
use fatcat_api_spec::Context;
use iron::headers::{Authorization, Bearer, ContentType};
use iron::mime::Mime;
use iron::{status, Chain, Headers, Iron, Listening};
use iron_test::response;
use std::str::FromStr;

pub static TEST_ADMIN_EDITOR_ID: &str = "aaaaaaaaaaaabkvkaaaaaaaaae";
//static TEST_ADMIN_EDITOR_ID: FatcatId = FatcatId::from_str("aaaaaaaaaaaabkvkaaaaaaaaae").unwrap();

// A current problem with this method is that if the test fails (eg, panics, assert fails), the
// server never gets closed, and the server thread hangs forever.
// One workaround might be to invert the function, take a closure, capture the panic/failure, and
// cleanup.
#[allow(dead_code)]
pub fn setup_client() -> (Client, Context, Listening) {
    let server = server::create_test_server().unwrap();

    // setup auth as admin user
    let admin_id = FatcatId::from_str(TEST_ADMIN_EDITOR_ID).unwrap();
    let token = server
        .auth_confectionary
        .create_token(admin_id, None)
        .unwrap();
    let client_context = Context {
        x_span_id: None,
        authorization: None,
        auth_data: Some(swagger::auth::AuthData::ApiKey(token)),
    };

    let router = fatcat_api_spec::router(server);
    let mut chain = Chain::new(router);
    chain.link_before(fatcat_api_spec::server::ExtractAuthData);
    chain.link_before(MacaroonAuthMiddleware::new());

    let mut iron_server = Iron::new(chain);
    iron_server.threads = 1;
    // XXX: this isn't supposed to block, but it is. Disabling these tests for now.
    let iron_server = iron_server
        .http("localhost:9300")
        .expect("Failed to start HTTP server");

    let client = Client::try_new_http("http://localhost:9144").unwrap();
    (client, client_context, iron_server)
}

#[allow(dead_code)]
pub fn setup_http() -> (
    Headers,
    iron::middleware::Chain,
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) {
    let server = server::create_test_server().unwrap();
    let conn = server.db_pool.get().expect("db_pool error");

    // setup auth as admin user
    let admin_id = FatcatId::from_str(TEST_ADMIN_EDITOR_ID).unwrap();
    let token = server
        .auth_confectionary
        .create_token(admin_id, None)
        .unwrap();

    let router = fatcat_api_spec::router(server);
    let mut chain = Chain::new(router);
    chain.link_before(fatcat_api_spec::server::ExtractAuthData);
    chain.link_before(MacaroonAuthMiddleware::new());
    let mut headers = Headers::new();
    let mime: Mime = "application/json".parse().unwrap();
    headers.set(ContentType(mime));
    headers.set(Authorization(Bearer { token: token }));

    (headers, chain, conn)
}

#[allow(dead_code)]
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
