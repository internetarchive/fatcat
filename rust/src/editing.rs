use crate::database_models::*;
use crate::database_schema::*;
use crate::entity_crud::EntityCrud;
use crate::errors::*;
use crate::identifiers::*;
use crate::server::*;
use diesel;
use diesel::prelude::*;
use fatcat_api_spec::models::*;
use uuid::Uuid;

pub struct EditContext {
    pub editor_id: FatCatId,
    pub editgroup_id: FatCatId,
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
            return Err(ErrorKind::EditgroupAlreadyAccepted(self.editgroup_id.to_string()).into());
        }
        Ok(())
    }
}

pub fn make_edit_context(
    conn: &DbConn,
    editor_id: FatCatId,
    editgroup_id: Option<FatCatId>,
    autoaccept: bool,
) -> Result<EditContext> {
    let editgroup_id: FatCatId = match (editgroup_id, autoaccept) {
        (Some(eg), _) => eg,
        // If autoaccept and no editgroup_id passed, always create a new one for this transaction
        (None, true) => {
            let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
                .values((editgroup::editor_id.eq(editor_id.to_uuid()),))
                .get_result(conn)?;
            FatCatId::from_uuid(&eg_row.id)
        }
        (None, false) => FatCatId::from_uuid(&get_or_create_editgroup(editor_id.to_uuid(), conn)?),
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
    editor_id: FatCatId,
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
pub fn get_or_create_editgroup(editor_id: Uuid, conn: &DbConn) -> Result<Uuid> {
    // check for current active
    let ed_row: EditorRow = editor::table.find(editor_id).first(conn)?;
    if let Some(current) = ed_row.active_editgroup_id {
        return Ok(current);
    }

    // need to insert and update
    let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
        .values((editgroup::editor_id.eq(ed_row.id),))
        .get_result(conn)?;
    diesel::update(editor::table.find(ed_row.id))
        .set(editor::active_editgroup_id.eq(eg_row.id))
        .execute(conn)?;
    Ok(eg_row.id)
}

/// This function should always be run within a transaction
pub fn accept_editgroup(editgroup_id: FatCatId, conn: &DbConn) -> Result<ChangelogRow> {
    // check that we haven't accepted already (in changelog)
    // NB: could leave this to a UNIQUE constraint
    // TODO: redundant with check_edit_context
    let count: i64 = changelog::table
        .filter(changelog::editgroup_id.eq(editgroup_id.to_uuid()))
        .count()
        .get_result(conn)?;
    if count > 0 {
        return Err(ErrorKind::EditgroupAlreadyAccepted(editgroup_id.to_string()).into());
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

    // update any editor's active editgroup
    let no_active: Option<Uuid> = None;
    diesel::update(editor::table)
        .filter(editor::active_editgroup_id.eq(editgroup_id.to_uuid()))
        .set(editor::active_editgroup_id.eq(no_active))
        .execute(conn)?;
    Ok(entry)
}
