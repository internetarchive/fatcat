//! Editor bearer token authentication

use swagger::auth::AuthData;
use macaroon::{Format, Macaroon, Verifier};
use data_encoding::BASE64;

use std::collections::HashMap;
use database_models::*;
use database_schema::*;
use api_helpers::*;
use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use errors::*;
use std::str::FromStr;

// 32 bytes max (!)
static DUMMY_KEY: &[u8] = b"dummy-key-a-one-two-three-a-la";

#[derive(Clone)]
pub struct AuthContext {
    pub editor_id: FatCatId,
    editor_row: EditorRow,
    roles: Vec<String>, // TODO: BTreeSet
}

impl AuthContext {

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string()) || self.roles.contains(&"admin".to_string())
    }
}

#[derive(Clone)]
pub struct AuthConfectionary {
    pub location: String,
    pub identifier: String,
    pub key: Vec<u8>,
    pub root_keys: HashMap<String, Vec<u8>>,
}

impl AuthConfectionary {
    pub fn new(location: String, identifier: String, key: Vec<u8>) -> AuthConfectionary {
        let mut root_keys = HashMap::new();
        root_keys.insert(identifier.clone(), key.clone());
        AuthConfectionary {
            location: location,
            identifier: identifier,
            key: key,
            root_keys: root_keys,
        }
    }

    pub fn new_dummy() -> AuthConfectionary {
        AuthConfectionary::new(
            "test.fatcat.wiki".to_string(),
            "dummy".to_string(),
            DUMMY_KEY.to_vec())
    }

    pub fn create_token(&self, conn: &DbConn, editor_id: FatCatId, expires: Option<DateTime<Utc>>) -> Result<String> {
        let _ed: EditorRow = editor::table
            .find(&editor_id.to_uuid())
            .get_result(conn)?;
        let mut mac = Macaroon::create(&self.location, &self.key, &self.identifier).expect("Macaroon creation");
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

    /// On success, returns Some((editor_id, scopes)), where `scopes` is a vector of strings.
    pub fn parse_macaroon_token(&self, conn: &DbConn, s: &str) -> Result<EditorRow> {
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
        let verify_key = match self.root_keys.get(mac.identifier()) {
            Some(key) => key,
            None => bail!("key not found for identifier: {}", mac.identifier()),
        };
        match mac.verify(verify_key, &mut verifier) {
            Ok(true) => (),
            Ok(false) => bail!("token overall verification failed"),
            Err(e) => bail!("token parsing failed: {:?}", e),
        }
        Ok(editor)
    }

    pub fn parse_swagger(&self, conn: &DbConn, auth_data: &Option<AuthData>) -> Result<Option<AuthContext>> {

        let token: Option<String> = match auth_data {
            Some(AuthData::ApiKey(header)) => {
                let header: Vec<String> = header.split_whitespace().map(|s| s.to_string()).collect();
                if !(header.len() == 2 && header[0] == "Bearer") {
                    bail!("invalid Bearer Auth HTTP header");
                }
                Some(header[1].clone())
            },
            None => None,
            _ => bail!("Authentication HTTP Header should either be empty or a Beaerer API key"),
        };
        let token = match token {
            Some(t) => t,
            None => return Ok(None),
        };
        let editor_row = self.parse_macaroon_token(conn, &token)?;
        let roles = if editor_row.is_admin { vec!["admin".to_string()] } else { vec![] };
        Ok(Some(AuthContext {
            editor_id: FatCatId::from_uuid(&editor_row.id),
            editor_row: editor_row,
            roles: roles,
        }))
    }

    // TODO: refactor out of this file?
    /// Only used from CLI tool
    pub fn inspect_token(&self, conn: &DbConn, token: &str) -> Result<()> {
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
        println!("verify: {:?}", self.parse_macaroon_token(conn, token));
        Ok(())
    }
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

// TODO: refactor out of this file?
/// Only used from CLI tool
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
