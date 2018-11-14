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

use fatcat_api_spec::{Context, Api, ApiNoContext, Future, ContextWrapperExt};

mod helpers;
use helpers::{setup_client};

#[test]
fn test_basic() {

    let (client, mut server) = setup_client();
    let client = client.with_context(Context::new());

    client.get_changelog_entry(1).wait().unwrap();
    server.close().unwrap()
}
