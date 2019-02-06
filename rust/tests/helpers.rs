use fatcat::auth::MacaroonAuthMiddleware;
use fatcat::editing_crud::EditgroupCrud;
use fatcat::identifiers::FatcatId;
use fatcat::server;
use fatcat_api_spec::models::Editgroup;
use iron::headers::{Authorization, Bearer, ContentType};
use iron::mime::Mime;
use iron::{status, Chain, Headers};
use iron_test::response;
use std::str::FromStr;

pub static TEST_ADMIN_EDITOR_ID: &str = "aaaaaaaaaaaabkvkaaaaaaaaae";
//static TEST_ADMIN_EDITOR_ID: FatcatId = FatcatId::from_str("aaaaaaaaaaaabkvkaaaaaaaaae").unwrap();

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

#[allow(dead_code)]
pub fn quick_editgroup(conn: &server::DbConn) -> FatcatId {
    let editor_id = FatcatId::from_str(TEST_ADMIN_EDITOR_ID).unwrap();
    let eg = Editgroup {
        editgroup_id: None,
        editor_id: Some(editor_id.to_string()),
        editor: None,
        changelog_index: None,
        submitted: None,
        description: Some("quick test editgroup".to_string()),
        extra: None,
        annotations: None,
        edits: None,
    };
    let row = eg.db_create(conn, false).unwrap();
    FatcatId::from_uuid(&row.id)
}
