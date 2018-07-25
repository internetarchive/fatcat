use data_encoding::BASE32_NOPAD;
use database_models::*;
use database_schema::*;
use diesel;
use diesel::prelude::*;
use errors::*;
use regex::Regex;
use uuid::Uuid;

pub fn get_or_create_editgroup(editor_id: Uuid, conn: &PgConnection) -> Result<Uuid> {
    // check for current active
    let ed_row: EditorRow = editor::table.find(editor_id).first(conn)?;
    if let Some(current) = ed_row.active_editgroup_id {
        return Ok(current);
    }

    // need to insert and update
    conn.build_transaction().run(|| {
        let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
            .values((editgroup::editor_id.eq(ed_row.id),))
            .get_result(conn)?;
        diesel::update(editor::table.find(ed_row.id))
            .set(editor::active_editgroup_id.eq(eg_row.id))
            .execute(conn)?;
        Ok(eg_row.id)
    })
}

pub fn accept_editgroup(editgroup_id: Uuid, conn: &PgConnection) -> Result<ChangelogRow> {
    conn.build_transaction().run(|| {
        // check that we haven't accepted already (in changelog)
        // NB: could leave this to a UNIQUE constraint
        let count: i64 = changelog::table
            .filter(changelog::editgroup_id.eq(editgroup_id))
            .count()
            .get_result(conn)?;
        if count > 0 {
            bail!(
                "editgroup {} has already been accepted",
                editgroup_id.to_string()
            );
        }

        // for each entity type...
        //for entity in (container_edit, creator_edit, file_edit, release_edit, work_edit) {
            /*
            // This would be the clean and efficient way, but see:
            // https://github.com/diesel-rs/diesel/issues/1478
            diesel::update(container_ident::table)
                .inner_join(container_edit::table.on(
                    container_ident::id.eq(container_edit::ident_id)
                ))
                .filter(container_edit::editgroup_id.eq(editgroup_id))
                .values((
                    container_ident::is_live.eq(true),
                    container_ident::rev_id.eq(container_edit::rev_id),
                    container_ident::redirect_id.eq(container_edit::redirect_id),
                ))
                .execute()?;
            */

        // Sketchy... but fast? Only a few queries per accept.
        for entity in &["container", "creator", "file", "work", "release"] {
            diesel::sql_query(format!(
                "
                    UPDATE {entity}_ident
                    SET
                        is_live = true,
                        rev_id = {entity}_edit.rev_id,
                        redirect_id = {entity}_edit.redirect_id
                    FROM {entity}_edit
                    WHERE
                        {entity}_ident.id = {entity}_edit.ident_id
                        AND {entity}_edit.editgroup_id = $1",
                entity = entity
            )).bind::<diesel::sql_types::Uuid, _>(editgroup_id)
                .execute(conn)?;
        }

        // append log/changelog row
        let entry: ChangelogRow = diesel::insert_into(changelog::table)
            .values((changelog::editgroup_id.eq(editgroup_id),))
            .get_result(conn)?;

        // update any editor's active editgroup
        let no_active: Option<Uuid> = None;
        diesel::update(editor::table)
            .filter(editor::active_editgroup_id.eq(editgroup_id))
            .set(editor::active_editgroup_id.eq(no_active))
            .execute(conn)?;
        Ok(entry)
    })
}

/// Convert fatcat IDs (base32 strings) to UUID
pub fn fcid2uuid(fcid: &str) -> Result<Uuid> {
    if fcid.len() != 26 {
        return Err(ErrorKind::InvalidFatcatId(fcid.to_string()).into());
    }
    let mut raw = vec![0; 16];
    BASE32_NOPAD
        .decode_mut(fcid.to_uppercase().as_bytes(), &mut raw)
        .map_err(|_dp| ErrorKind::InvalidFatcatId(fcid.to_string()))?;
    // unwrap() is safe here, because we know raw is always 16 bytes
    Ok(Uuid::from_bytes(&raw).unwrap())
}

/// Convert UUID to fatcat ID string (base32 encoded)
pub fn uuid2fcid(id: &Uuid) -> String {
    let raw = id.as_bytes();
    BASE32_NOPAD.encode(raw).to_lowercase()
}

pub fn check_pmcid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^PMC\d+$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(ErrorKind::MalformedExternalId(format!(
            "not a valid PubMed Central ID (PMCID): '{}' (expected, eg, 'PMC12345')",
            raw
        )).into())
    }
}

pub fn check_pmid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d+$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(ErrorKind::MalformedExternalId(format!(
            "not a valid PubMed ID (PMID): '{}' (expected, eg, '1234')",
            raw
        )).into())
    }
}

pub fn check_wikidata_qid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^Q\d+$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(ErrorKind::MalformedExternalId(format!(
            "not a valid Wikidata QID: '{}' (expected, eg, 'Q1234')",
            raw
        )).into())
    }
}

pub fn check_doi(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^10.\d{3,6}/.+$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(ErrorKind::MalformedExternalId(format!(
            "not a valid DOI: '{}' (expected, eg, '10.1234/aksjdfh')",
            raw
        )).into())
    }
}

pub fn check_issn(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d{4}-\d{3}[0-9X]$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(ErrorKind::MalformedExternalId(format!(
            "not a valid ISSN: '{}' (expected, eg, '1234-5678')",
            raw
        )).into())
    }
}

pub fn check_orcid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d{4}-\d{4}-\d{4}-\d{4}$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(ErrorKind::MalformedExternalId(format!(
            "not a valid ORCID: '{}' (expected, eg, '0123-4567-3456-6789')",
            raw
        )).into())
    }
}

// TODO: make the above checks "more correct"
// TODO: check ISBN-13
// TODO: check hashes (SHA-1, etc)
