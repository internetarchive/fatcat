//! Editor bearer token authentication

use swagger::auth::{AuthData, Authorization, Scopes};
use macaroon::{Format, Macaroon, Verifier};
use data_encoding::BASE64;

use std::collections::BTreeSet;
use std::fmt;
use database_models::*;
use database_schema::*;
use api_helpers::*;
use chrono::prelude::*;
use diesel;
use iron;
use diesel::prelude::*;
use errors::*;
//use serde_json;
use std::str::FromStr;
//use uuid::Uuid;

// 32 bytes max (!)
static DUMMY_KEY: &[u8] = b"dummy-key-a-one-two-three-a-la";

#[derive(Debug)]
pub struct OpenAuthMiddleware;

#[derive(Debug)]
pub struct AuthError {
    msg: String,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AuthError: {}", &self.msg)
    }
}

impl iron::Error for AuthError {
    fn description(&self) -> &str {
        &self.msg
    }
    fn cause(&self) -> Option<&iron::Error> {
        None
    }
}

/*
#[derive(Debug)]
pub struct FatcatAuthBakery {
    root_key_store: bool, // hashmap
    signing_key: bool, // string name
}

// impl:
//  - new()
//  - verify(&str) -> Result<AuthContext>
*/

fn new_auth_ironerror(m: &str) -> iron::error::IronError {
    iron::error::IronError::new(
        AuthError { msg: m.to_string() },
        (iron::status::BadRequest, m.to_string())
    )
}

impl OpenAuthMiddleware {
    /// Create a middleware that authorizes with the configured subject.
    pub fn new() -> OpenAuthMiddleware {
        OpenAuthMiddleware
    }
}
impl iron::middleware::BeforeMiddleware for OpenAuthMiddleware {
    fn before(&self, req: &mut iron::Request) -> iron::IronResult<()> {
        req.extensions.insert::<Authorization>(Authorization {
            subject: "undefined".to_string(),
            scopes: Scopes::All,
            issuer: None,
        });
        Ok(())
    }
}

#[derive(Debug)]
pub struct MacaroonAuthMiddleware;

impl MacaroonAuthMiddleware {

    pub fn new() -> MacaroonAuthMiddleware {
        macaroon::initialize().unwrap();
        MacaroonAuthMiddleware
    }
}

impl iron::middleware::BeforeMiddleware for MacaroonAuthMiddleware {
    fn before(&self, req: &mut iron::Request) -> iron::IronResult<()> {

        let res: Option<(String, Vec<String>)> = match req.extensions.get::<AuthData>() {
            Some(AuthData::ApiKey(header)) => {
                let header: Vec<String> = header.split_whitespace().map(|s| s.to_string()).collect();
                if !(header.len() == 2 && header[0] == "Bearer") {
                    return Err(new_auth_ironerror("invalid bearer auth HTTP Header"));
                }
                sniff_macaroon_token(&header[1]).expect("valid macaroon")
            },
            None => None,
            _ => {
                return Err(new_auth_ironerror("auth HTTP Header should be empty or API key"));
            }
        };
        if let Some((editor_id, scopes)) = res {
            let mut scope_set = BTreeSet::new();
            for s in scopes {
                scope_set.insert(s);
            }
            req.extensions.insert::<Authorization>(Authorization {
                subject: editor_id,
                scopes: Scopes::Some(scope_set),
                issuer: None,
            });
        };
        Ok(())
    }
}

/// Just checks signature and expired time; can't hit database, so nothing else
pub fn sniff_macaroon_token(s: &str) -> Result<Option<(String,Vec<String>)>> {
    let raw = BASE64.decode(s.as_bytes())?;
    let mac = match Macaroon::deserialize(&raw) {
        Ok(m) => m,
        Err(e) => bail!("macaroon deserialize error: {:?}", e),
    };
    let mac = match mac.validate() {
        Ok(m) => m,
        Err(e) => bail!("macaroon validate error: {:?}", e),
    };
    let mut editor_id: Option<FatCatId> = None;
    for caveat in mac.first_party_caveats() {
        if caveat.predicate().starts_with("editor_id = ") {
            editor_id = Some(FatCatId::from_str(caveat.predicate().get(12..).unwrap())?);
            break
        }
    }
    let editor_id = editor_id.expect("expected an editor_id caveat");
    Ok(Some((editor_id.to_string(), vec![])))
}

/// On success, returns Some((editor_id, scopes)), where `scopes` is a vector of strings.
pub fn parse_macaroon_token(conn: &DbConn, s: &str) -> Result<Option<(String,Vec<String>)>> {
    let raw = BASE64.decode(s.as_bytes())?;
    let mac = match Macaroon::deserialize(&raw) {
        Ok(m) => m,
        Err(e) => bail!("macaroon deserialize error: {:?}", e),
    };
    let mac = match mac.validate() {
        Ok(m) => m,
        Err(e) => bail!("macaroon validate error: {:?}", e),
    };
    let mut verifier = Verifier::new();
    let mut editor_id: Option<FatCatId> = None;
    for caveat in mac.first_party_caveats() {
        if caveat.predicate().starts_with("editor_id = ") {
            editor_id = Some(FatCatId::from_str(caveat.predicate().get(12..).unwrap())?);
            break
        }
    }
    let editor_id = editor_id.expect("expected an editor_id caveat");
    verifier.satisfy_exact(&format!("editor_id = {}", editor_id.to_string()));
    let mut created: Option<DateTime<Utc>> = None;
    for caveat in mac.first_party_caveats() {
        if caveat.predicate().starts_with("created = ") {
            created = Some(DateTime::parse_from_rfc3339(caveat.predicate().get(10..).unwrap())
                .unwrap()
                .with_timezone(&Utc));
            break
        }
    }
    let created = created.expect("expected a 'created' caveat");
    verifier.satisfy_exact(&format!("created = {}", created.to_rfc3339_opts(SecondsFormat::Secs, true)));
    let editor: EditorRow = editor::table
        .find(&editor_id.to_uuid())
        .get_result(conn)?;
    let auth_epoch = DateTime::<Utc>::from_utc(editor.auth_epoch, Utc);
    if created < auth_epoch {
        bail!("token created before current auth_epoch (was probably revoked by editor)")
    }
    verifier.satisfy_general(|p: &str| -> bool {
        // not expired (based on expires)
        if p.starts_with("expires = ") {
            let expires: DateTime<Utc> = DateTime::parse_from_rfc3339(p.get(12..).unwrap())
                .unwrap()
                .with_timezone(&Utc);
            expires < Utc::now()
        } else {
            false
        }
    });
    if !mac.verify_signature(DUMMY_KEY) {
        bail!("token signature verification failed");
    };
    match mac.verify(DUMMY_KEY, &mut verifier) {
        Ok(true) => (),
        Ok(false) => bail!("token overall verification failed"),
        Err(e) => bail!("token parsing failed: {:?}", e),
    }
    Ok(Some((editor_id.to_string(), vec![])))
}

pub fn print_editors(conn: &DbConn) -> Result<()>{
    // iterate over all editors. format id, print flags, auth_epoch
    let all_editors: Vec<EditorRow> = editor::table
        .load(conn)?;
    println!("editor_id\t\t\tis_admin/is_bot\tauth_epoch\t\t\tusername\twrangler_id");
    for e in all_editors {
        println!("{}\t{}\t{}\t{}\t{}\t{:?}",
            FatCatId::from_uuid(&e.id).to_string(),
            e.is_admin,
            e.is_bot,
            e.auth_epoch,
            e.username,
            e.wrangler_id,
        );
    }
    Ok(())
}

// TODO: move to api_helpers or some such
// TODO: verify username
pub fn create_editor(conn: &DbConn, username: String, is_admin: bool, is_bot: bool) -> Result<EditorRow> {
    let ed: EditorRow = diesel::insert_into(editor::table)
        .values((
            editor::username.eq(username),
            editor::is_admin.eq(is_admin),
            editor::is_bot.eq(is_bot),
        ))
        .get_result(conn)?;
    Ok(ed) 
}

pub fn create_token(conn: &DbConn, editor_id: FatCatId, expires: Option<DateTime<Utc>>) -> Result<String> {
    let _ed: EditorRow = editor::table
        .find(&editor_id.to_uuid())
        .get_result(conn)?;
    let mut mac = Macaroon::create("fatcat.wiki", DUMMY_KEY, "dummy-key").expect("Macaroon creation");
    mac.add_first_party_caveat(&format!("editor_id = {}", editor_id.to_string()));
    // TODO: put created one second in the past to prevent timing synchronization glitches?
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    mac.add_first_party_caveat(&format!("created = {}", now));
    if let Some(expires) = expires {
        mac.add_first_party_caveat(&format!("expires = {:?}",
            &expires.to_rfc3339_opts(SecondsFormat::Secs, true)));
    };
    let raw = mac.serialize(Format::V2).expect("macaroon serialization");
    Ok(BASE64.encode(&raw))
}

pub fn inspect_token(conn: &DbConn, token: &str) -> Result<()> {
    let raw = BASE64.decode(token.as_bytes())?;
    let mac = match Macaroon::deserialize(&raw) {
        Ok(m) => m,
        Err(e) => bail!("macaroon deserialize error: {:?}", e),
    };
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    println!("current time: {}", now);
    println!("domain (location): {:?}", mac.location());
    println!("signing key name (identifier): {}", mac.identifier());
    for caveat in mac.first_party_caveats() {
        println!("caveat: {}", caveat.predicate());
    }
    // TODO: don't display full stacktrace on failure
    println!("verify: {:?}", parse_macaroon_token(conn, token));
    Ok(())
}

pub fn revoke_tokens(conn: &DbConn, editor_id: FatCatId) -> Result<()>{
    diesel::update(editor::table.filter(editor::id.eq(&editor_id.to_uuid())))
        .set(editor::auth_epoch.eq(Utc::now()))
        .execute(conn)?;
    Ok(())
}

pub fn revoke_tokens_everyone(conn: &DbConn) -> Result<()> {
    diesel::update(editor::table)
        .set(editor::auth_epoch.eq(Utc::now()))
        .execute(conn)?;
    Ok(())
}
