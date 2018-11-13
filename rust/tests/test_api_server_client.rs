/*
 * This file contains API server tests that hit the API through the Rust API client library.
 *
 * These tests are relatively complex and mutate database state. Tests should take care not to be
 * racey or overwrite each other; for example, they should randomize external identifiers and
 * minimize reliance on hard-coded example entities.
 *
 * Note that these tests currently do *not* include tests the authentication system, or any other
 * middleware.
 */

extern crate fatcat;
extern crate fatcat_api_spec;
extern crate uuid;
extern crate iron;

use iron::{Iron, Listening};
use fatcat_api_spec::{Context, Api, ApiNoContext, Future, ContextWrapperExt};
use fatcat_api_spec::client::Client;
//use uuid::Uuid;


fn setup() -> (
    Context,
    Client,
    Listening,
) {
    let server = fatcat::test_server().unwrap();
    let router = fatcat_api_spec::router(server);
    let iron_server = Iron::new(router)
        .http("localhost:9144")
        .expect("Failed to start HTTP server");

    let context = Context::new();
    let client = Client::try_new_http("http://localhost:9144").unwrap();
    (context, client, iron_server)
}

#[test]
fn test_basic() {

    let (context, client, mut server) = setup();
    let client = client.with_context(context);

    client.get_changelog_entry(1).wait().unwrap();
    server.close().unwrap()
}

#[test]
fn test_basic2() {

    let (context, client, mut server) = setup();
    let client = client.with_context(context);

    client.get_changelog_entry(1).wait().unwrap();
    server.close().unwrap()
}
