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

use fatcat_api_spec::{ApiNoContext, ContextWrapperExt, Future};

mod helpers;

// Disabled due to hang
//#[test]
#[allow(dead_code)]
fn test_basic() {
    let (client, context, mut server) = helpers::setup_client();
    let client = client.with_context(context);

    client.get_changelog_entry(1).wait().unwrap();
    server.close().unwrap()
}
