use crate::database_models::*;
use crate::database_schema::*;
use crate::entity_crud::EntityCrud;
use crate::errors::{Result, FatcatError};
use crate::identifiers::{FatcatId, check_username};
use crate::server::DbConn;
use diesel;
use diesel::prelude::*;
use fatcat_api_spec::models::*;
use uuid::Uuid;

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
            return Err(FatcatError::EditgroupAlreadyAccepted(self.editgroup_id.to_string()).into());
        }
        Ok(())
    }
}

pub fn make_edit_context(
    conn: &DbConn,
    editor_id: FatcatId,
    editgroup_id: Option<FatcatId>,
    autoaccept: bool,
) -> Result<EditContext> {
    let editgroup_id: FatcatId = match (editgroup_id, autoaccept) {
        (Some(eg), _) => eg,
        // If autoaccept and no editgroup_id passed, always create a new one for this transaction
        (None, true) => {
            let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
                .values((editgroup::editor_id.eq(editor_id.to_uuid()),))
                .get_result(conn)?;
            FatcatId::from_uuid(&eg_row.id)
        }
        (None, false) => FatcatId::from_uuid(&create_editgroup(conn, editor_id.to_uuid())?),
    };
    Ok(EditContext {
        editor_id,
        editgroup_id,
        extra_json: None,
        autoaccept,
    })
}

pub fn create_editor(
    conn: &DbConn,
    username: String,
    is_admin: bool,
    is_bot: bool,
) -> Result<EditorRow> {
    check_username(&username)?;
    let ed: EditorRow = diesel::insert_into(editor::table)
        .values((
            editor::username.eq(username),
            editor::is_admin.eq(is_admin),
            editor::is_bot.eq(is_bot),
        ))
        .get_result(conn)?;
    Ok(ed)
}

pub fn update_editor_username(
    conn: &DbConn,
    editor_id: FatcatId,
    username: String,
) -> Result<EditorRow> {
    check_username(&username)?;
    diesel::update(editor::table.find(editor_id.to_uuid()))
        .set(editor::username.eq(username))
        .execute(conn)?;
    let editor: EditorRow = editor::table.find(editor_id.to_uuid()).get_result(conn)?;
    Ok(editor)
}

/// This function should always be run within a transaction
pub fn create_editgroup(conn: &DbConn, editor_id: Uuid) -> Result<Uuid> {
    // need to insert and update
    let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
        .values((editgroup::editor_id.eq(editor_id),))
        .get_result(conn)?;
    Ok(eg_row.id)
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

    Ok(entry)
}
