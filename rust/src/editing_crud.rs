use crate::database_models::*;
use crate::database_schema::*;
use crate::entity_crud::ExpandFlags;
use crate::errors::*;
use crate::identifiers::{self, FatcatId};
use crate::server::DbConn;
use diesel::prelude::*;
use fatcat_api_spec::models::*;
use std::str::FromStr;
use uuid::Uuid;

/*
 * The object types with accessors defined here:
 *
 * - editor
 * - editgroup
 * - editgroup_annotation
 *
 * Generic verbs/actions look like:
 *
 * - db_get (single)
 * - db_get_range (group; by timestamp, with limits)
 * - db_create (single)
 * - db_update (single)
 * - db_expand (single)
 *
 */

pub trait EditorCrud {
    fn db_get(conn: &DbConn, editor_id: FatcatId) -> Result<EditorRow>;
    fn db_create(&self, conn: &DbConn) -> Result<EditorRow>;
    fn db_update_username(&self, conn: &DbConn, editor_id: FatcatId) -> Result<EditorRow>;
}
impl EditorCrud for Editor {
    fn db_get(conn: &DbConn, editor_id: FatcatId) -> Result<EditorRow> {
        let editor: EditorRow = match editor::table.find(editor_id.to_uuid()).get_result(conn) {
            Ok(ed) => ed,
            Err(diesel::result::Error::NotFound) => {
                return Err(
                    FatcatError::NotFound("editor".to_string(), editor_id.to_string()).into(),
                );
            }
            other => other?,
        };
        Ok(editor)
    }

    fn db_create(&self, conn: &DbConn) -> Result<EditorRow> {
        identifiers::check_username(&self.username)?;
        let is_admin = self.is_admin.unwrap_or(false);
        let is_bot = self.is_bot.unwrap_or(false);
        let row: EditorRow = diesel::insert_into(editor::table)
            .values((
                editor::username.eq(&self.username),
                editor::is_admin.eq(is_admin),
                editor::is_bot.eq(is_bot),
            ))
            .get_result(conn)?;
        Ok(row)
    }

    fn db_update_username(&self, conn: &DbConn, editor_id: FatcatId) -> Result<EditorRow> {
        identifiers::check_username(&self.username)?;
        diesel::update(editor::table.find(editor_id.to_uuid()))
            .set(editor::username.eq(&self.username))
            .execute(conn)?;
        let editor: EditorRow = editor::table.find(editor_id.to_uuid()).get_result(conn)?;
        Ok(editor)
    }
}

pub trait EditgroupCrud {
    fn db_get(conn: &DbConn, editgroup_id: FatcatId) -> Result<EditgroupRow>;
    fn db_get_with_changelog(
        conn: &DbConn,
        editgroup_id: FatcatId,
    ) -> Result<(EditgroupRow, Option<ChangelogRow>)>;
    fn db_expand(&mut self, conn: &DbConn, expand: ExpandFlags) -> Result<()>;
    fn db_get_range_for_editor(
        conn: &DbConn,
        editor_id: FatcatId,
        limit: u64,
        since: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<(EditgroupRow, Option<ChangelogRow>)>>;
    fn db_get_range_reviewable(
        conn: &DbConn,
        limit: u64,
        since: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<EditgroupRow>>;
    fn db_create(&self, conn: &DbConn, autoaccept: bool) -> Result<EditgroupRow>;
    fn db_update(
        &self,
        conn: &DbConn,
        editgroup_id: FatcatId,
        submit: Option<bool>,
    ) -> Result<EditgroupRow>;
}

impl EditgroupCrud for Editgroup {
    /// This method does *not* expand the 'edits'; currently that's still done in the endpoint
    /// handler, but it probably should be done in this trait with a db_expand()
    fn db_get(conn: &DbConn, editgroup_id: FatcatId) -> Result<EditgroupRow> {
        // Note: at least for now, continue to fetch along with changelog to ensure is_accepted is
        // consistent.
        let (row, _): (EditgroupRow, Option<ChangelogRow>) =
            Self::db_get_with_changelog(conn, editgroup_id)?;
        Ok(row)
    }

    fn db_get_with_changelog(
        conn: &DbConn,
        editgroup_id: FatcatId,
    ) -> Result<(EditgroupRow, Option<ChangelogRow>)> {
        let (eg_row, cl_row): (EditgroupRow, Option<ChangelogRow>) = match editgroup::table
            .left_outer_join(changelog::table)
            .filter(editgroup::id.eq(editgroup_id.to_uuid()))
            .first(conn)
        {
            Ok(eg) => eg,
            Err(diesel::result::Error::NotFound) => {
                return Err(FatcatError::NotFound(
                    "editgroup".to_string(),
                    editgroup_id.to_string(),
                )
                .into());
            }
            other => other?,
        };

        ensure!(
            cl_row.is_some() == eg_row.is_accepted,
            "internal database consistency error on editgroup: {}",
            editgroup_id
        );
        Ok((eg_row, cl_row))
    }

    /// Note: this *still* doesn't expand the 'edits', at least yet. *Only* the direct editgroup
    /// 'GET' handler does that.
    fn db_expand(&mut self, conn: &DbConn, expand: ExpandFlags) -> Result<()> {
        if expand.editors {
            let editor_id = FatcatId::from_str(
                self.editor_id
                    .as_ref()
                    .expect("tried to expand bare Editor model"),
            )?;
            self.editor = Some(Editor::db_get(conn, editor_id)?.into_model());
        }
        Ok(())
    }

    fn db_get_range_for_editor(
        conn: &DbConn,
        editor_id: FatcatId,
        limit: u64,
        since: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<(EditgroupRow, Option<ChangelogRow>)>> {
        let rows: Vec<(EditgroupRow, Option<ChangelogRow>)> = match (since, before) {
            (Some(since), None) => editgroup::table
                .left_outer_join(changelog::table)
                .filter(editgroup::editor_id.eq(editor_id.to_uuid()))
                .filter(editgroup::created.gt(since))
                .order_by(editgroup::created.asc())
                .limit(limit as i64)
                .get_results(conn)?,
            (_, Some(before)) => editgroup::table
                .left_outer_join(changelog::table)
                .filter(editgroup::editor_id.eq(editor_id.to_uuid()))
                .filter(editgroup::created.lt(before))
                .order_by(editgroup::created.desc())
                .limit(limit as i64)
                .get_results(conn)?,
            (None, None) => editgroup::table
                .left_outer_join(changelog::table)
                .filter(editgroup::editor_id.eq(editor_id.to_uuid()))
                .order_by(editgroup::created.desc())
                .limit(limit as i64)
                .get_results(conn)?,
        };
        Ok(rows)
    }

    fn db_get_range_reviewable(
        conn: &DbConn,
        limit: u64,
        since: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<EditgroupRow>> {
        let rows: Vec<EditgroupRow> = match (since, before) {
            (Some(since), None) => editgroup::table
                .filter(editgroup::is_accepted.eq(false))
                .filter(editgroup::submitted.is_not_null())
                .filter(editgroup::submitted.gt(since))
                .order_by(editgroup::submitted.asc())
                .limit(limit as i64)
                .get_results(conn)?,
            (_, Some(before)) => editgroup::table
                .filter(editgroup::is_accepted.eq(false))
                .filter(editgroup::submitted.is_not_null())
                .filter(editgroup::submitted.lt(before))
                .order_by(editgroup::submitted.desc())
                .limit(limit as i64)
                .get_results(conn)?,
            (None, None) => editgroup::table
                .filter(editgroup::is_accepted.eq(false))
                .filter(editgroup::submitted.is_not_null())
                .order_by(editgroup::created.desc())
                .limit(limit as i64)
                .get_results(conn)?,
        };
        Ok(rows)
    }

    fn db_create(&self, conn: &DbConn, autoaccept: bool) -> Result<EditgroupRow> {
        let editor_id = self
            .editor_id
            .clone()
            .ok_or_else(|| FatcatError::BadRequest("missing editor_id".to_string()))?;
        let editor_id = FatcatId::from_str(&editor_id)?;
        let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
            .values((
                editgroup::editor_id.eq(editor_id.to_uuid()),
                editgroup::is_accepted.eq(autoaccept),
                editgroup::description.eq(&self.description),
                editgroup::extra_json.eq(&self.extra),
            ))
            .get_result(conn)?;
        Ok(eg_row)
    }

    fn db_update(
        &self,
        conn: &DbConn,
        editgroup_id: FatcatId,
        submit: Option<bool>,
    ) -> Result<EditgroupRow> {
        let row = Self::db_get(conn, editgroup_id)?;
        if row.is_accepted {
            // "can't update an accepted editgroup"
            Err(FatcatError::EditgroupAlreadyAccepted(
                editgroup_id.to_string(),
            ))?;
        }
        match submit {
            Some(true) => {
                // just a submit
                let row = diesel::update(editgroup::table.find(editgroup_id.to_uuid()))
                    .set(editgroup::submitted.eq(diesel::dsl::now))
                    .get_result(conn)?;
                Ok(row)
            }
            Some(false) => {
                // just a retraction
                let submitted: Option<chrono::NaiveDateTime> = None;
                let row = diesel::update(editgroup::table.find(editgroup_id.to_uuid()))
                    .set(editgroup::submitted.eq(submitted))
                    .get_result(conn)?;
                Ok(row)
            }
            None => {
                // full-on row update... though we only do extra and description
                let row = diesel::update(editgroup::table.find(editgroup_id.to_uuid()))
                    .set((
                        editgroup::description.eq(&self.description),
                        editgroup::extra_json.eq(&self.extra),
                    ))
                    .get_result(conn)?;
                Ok(row)
            }
        }
    }
}

pub trait EditgroupAnnotationCrud {
    fn db_get(conn: &DbConn, annotation_id: Uuid) -> Result<EditgroupAnnotationRow>;
    fn db_expand(&mut self, conn: &DbConn, expand: ExpandFlags) -> Result<()>;
    fn db_get_range_for_editor(
        conn: &DbConn,
        editor_id: FatcatId,
        limit: u64,
        since: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<EditgroupAnnotationRow>>;
    fn db_get_range_for_editgroup(
        conn: &DbConn,
        editgroup_id: FatcatId,
        limit: u64,
        since: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<EditgroupAnnotationRow>>;
    fn db_create(&self, conn: &DbConn) -> Result<EditgroupAnnotationRow>;
}

impl EditgroupAnnotationCrud for EditgroupAnnotation {
    fn db_get(conn: &DbConn, annotation_id: Uuid) -> Result<EditgroupAnnotationRow> {
        let row: EditgroupAnnotationRow = match editgroup_annotation::table
            .find(annotation_id)
            .get_result(conn)
        {
            Ok(ea) => ea,
            Err(diesel::result::Error::NotFound) => {
                return Err(FatcatError::NotFound(
                    "editgroup_annotation".to_string(),
                    annotation_id.to_string(),
                )
                .into());
            }
            other => other?,
        };
        Ok(row)
    }

    fn db_expand(&mut self, conn: &DbConn, expand: ExpandFlags) -> Result<()> {
        if expand.editors {
            let editor_id = FatcatId::from_str(
                self.editor_id
                    .as_ref()
                    .expect("tried to expand bare Editor model"),
            )?;
            self.editor = Some(Editor::db_get(conn, editor_id)?.into_model());
        }
        Ok(())
    }

    fn db_get_range_for_editor(
        conn: &DbConn,
        editor_id: FatcatId,
        limit: u64,
        since: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<EditgroupAnnotationRow>> {
        let rows: Vec<EditgroupAnnotationRow> = match (since, before) {
            (Some(since), None) => editgroup_annotation::table
                .filter(editgroup_annotation::editor_id.eq(editor_id.to_uuid()))
                .filter(editgroup_annotation::created.gt(since))
                .order_by(editgroup_annotation::created.asc())
                .limit(limit as i64)
                .get_results(conn)?,
            (_, Some(before)) => editgroup_annotation::table
                .filter(editgroup_annotation::editor_id.eq(editor_id.to_uuid()))
                .filter(editgroup_annotation::created.lt(before))
                .order_by(editgroup_annotation::created.desc())
                .limit(limit as i64)
                .get_results(conn)?,
            (None, None) => editgroup_annotation::table
                .filter(editgroup_annotation::editor_id.eq(editor_id.to_uuid()))
                .order_by(editgroup_annotation::created.desc())
                .limit(limit as i64)
                .get_results(conn)?,
        };
        Ok(rows)
    }

    fn db_get_range_for_editgroup(
        conn: &DbConn,
        editgroup_id: FatcatId,
        limit: u64,
        since: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<EditgroupAnnotationRow>> {
        let rows: Vec<EditgroupAnnotationRow> = match (since, before) {
            (Some(since), None) => editgroup_annotation::table
                .filter(editgroup_annotation::editgroup_id.eq(editgroup_id.to_uuid()))
                .filter(editgroup_annotation::created.gt(since))
                .order_by(editgroup_annotation::created.asc())
                .limit(limit as i64)
                .get_results(conn)?,
            (_, Some(before)) => editgroup_annotation::table
                .filter(editgroup_annotation::editgroup_id.eq(editgroup_id.to_uuid()))
                .filter(editgroup_annotation::created.lt(before))
                .order_by(editgroup_annotation::created.desc())
                .limit(limit as i64)
                .get_results(conn)?,
            (None, None) => editgroup_annotation::table
                .filter(editgroup_annotation::editgroup_id.eq(editgroup_id.to_uuid()))
                .order_by(editgroup_annotation::created.desc())
                .limit(limit as i64)
                .get_results(conn)?,
        };
        Ok(rows)
    }

    fn db_create(&self, conn: &DbConn) -> Result<EditgroupAnnotationRow> {
        let editor_id = self
            .editor_id
            .clone()
            .ok_or_else(|| FatcatError::BadRequest("missing editor_id".to_string()))?;
        let editor_id = FatcatId::from_str(&editor_id)?;
        let editgroup_id = self
            .editgroup_id
            .clone()
            .ok_or_else(|| FatcatError::BadRequest("missing editgroup_id".to_string()))?;
        let editgroup_id = FatcatId::from_str(&editgroup_id)?;
        let ed: EditgroupAnnotationRow = diesel::insert_into(editgroup_annotation::table)
            .values((
                editgroup_annotation::editor_id.eq(editor_id.to_uuid()),
                editgroup_annotation::editgroup_id.eq(editgroup_id.to_uuid()),
                editgroup_annotation::comment_markdown.eq(&self.comment_markdown),
                editgroup_annotation::extra_json.eq(&self.extra),
            ))
            .get_result(conn)?;
        Ok(ed)
    }
}
