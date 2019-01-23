//! Helpers and types for dealing with the edit lifecycle.
//!
//! Does not contain the core code for creating/updating/reading/deleting the Editor, Annotation,
//! and Changelog objects, which lives under `editing_crud`.

use crate::database_models::*;
use crate::database_schema::*;
use crate::editing_crud::EditgroupCrud;
use crate::entity_crud::EntityCrud;
use crate::errors::{FatcatError, Result};
use crate::identifiers::FatcatId;
use crate::server::DbConn;
use diesel;
use diesel::prelude::*;
use fatcat_api_spec::models::*;

pub struct EditContext {
    pub editor_id: FatcatId,
    pub editgroup_id: FatcatId,
    pub extra_json: Option<serde_json::Value>,
    pub autoaccept: bool,
}

impl EditContext {
    /// This function should always be run within a transaction
    pub fn check(&self, conn: &DbConn) -> Result<()> {
        let count: i64 = changelog::table
            .filter(changelog::editgroup_id.eq(&self.editgroup_id.to_uuid()))
            .count()
            .get_result(conn)?;
        if count > 0 {
            Err(FatcatError::EditgroupAlreadyAccepted(
                self.editgroup_id.to_string(),
            ))?;
        }
        Ok(())
    }
}

pub fn make_edit_context(
    conn: &DbConn,
    editor_id: FatcatId,
    editgroup_id: Option<FatcatId>,
    autoaccept: bool,
    description: Option<String>,
    extra: Option<serde_json::Value>,
) -> Result<EditContext> {
    // *either* autoaccept is false and editgroup_id is Some, *or* autoaccept is true and
    // editgroup_id is None
    let editgroup_id: FatcatId = match (editgroup_id, autoaccept) {
        (Some(eg), false) => eg,
        (None, true) => {
            let eg = Editgroup {
                editgroup_id: None,
                editor_id: Some(editor_id.to_string()),
                editor: None,
                changelog_index: None,
                submitted: None,
                description: description,
                extra: extra,
                annotations: None,
                edits: None,
            };
            let row = eg.db_create(conn, autoaccept)?;
            FatcatId::from_uuid(&row.id)
        }
        _ => Err(FatcatError::BadRequest(
            "unsupported batch editgroup/accept combination".to_string(),
        ))?,
    };
    Ok(EditContext {
        editor_id,
        editgroup_id,
        extra_json: None,
        autoaccept,
    })
}

/// This function should always be run within a transaction
pub fn accept_editgroup(conn: &DbConn, editgroup_id: FatcatId) -> Result<ChangelogRow> {
    // check that we haven't accepted already (in changelog)
    // NB: could leave this to a UNIQUE constraint
    // TODO: redundant with check_edit_context
    let count: i64 = changelog::table
        .filter(changelog::editgroup_id.eq(editgroup_id.to_uuid()))
        .count()
        .get_result(conn)?;
    if count > 0 {
        return Err(FatcatError::EditgroupAlreadyAccepted(editgroup_id.to_string()).into());
    }

    // copy edit columns to ident table
    ContainerEntity::db_accept_edits(conn, editgroup_id)?;
    CreatorEntity::db_accept_edits(conn, editgroup_id)?;
    FileEntity::db_accept_edits(conn, editgroup_id)?;
    FilesetEntity::db_accept_edits(conn, editgroup_id)?;
    WebcaptureEntity::db_accept_edits(conn, editgroup_id)?;
    ReleaseEntity::db_accept_edits(conn, editgroup_id)?;
    WorkEntity::db_accept_edits(conn, editgroup_id)?;

    // append log/changelog row
    let entry: ChangelogRow = diesel::insert_into(changelog::table)
        .values((changelog::editgroup_id.eq(editgroup_id.to_uuid()),))
        .get_result(conn)?;

    // update editgroup row with is_accepted
    diesel::update(editgroup::table.find(editgroup_id.to_uuid()))
        .set(editgroup::is_accepted.eq(true))
        .execute(conn)?;

    Ok(entry)
}
