
use diesel::prelude::*;
use diesel::{self, insert_into};
use database_schema::*;
use database_models::*;
use errors::*;
use fatcat_api::models::*;
use api_helpers::{FatCatId, DbConn};
use uuid::Uuid;
use std::marker::Sized;
use std::str::FromStr;
use serde_json;

pub struct EditContext {
    pub editor_id: FatCatId,
    pub editgroup_id: FatCatId,
    pub extra_json: Option<serde_json::Value>,
}

/* One goal here is to abstract the non-entity-specific bits into generic traits or functions,
 * instead of macros.
 *
 * Notably:
 *
 *   db_get
 *   db_get_rev
 *   db_create
 *   db_create_batch
 *   db_update
 *   db_delete
 *   db_get_history
 *
 * For now, these will probably be macros, until we can level up our trait/generics foo.
 */

// Associated Type, not parametric
pub trait EntityCrud where Self: Sized {
    type EditRow; // EntityEditRow
    type IdentRow; // EntityIdentRow
    type RevRow;

    fn parse_editgroup_id(&self) -> Result<Option<FatCatId>>;

    // Generic Methods
    fn db_get(conn: &DbConn, ident: FatCatId) -> Result<Self>;
    fn db_get_rev(conn: &DbConn, rev_id: Uuid) -> Result<Self>;
    fn db_create(&self, conn: &DbConn, edit_context: &EditContext) -> Result<Self::EditRow>;
    fn db_create_batch(conn: &DbConn, edit_context: &EditContext, models: &[Self]) -> Result<Vec<Self::EditRow>>;
    fn db_update(&self, conn: &DbConn, edit_context: &EditContext, ident: FatCatId) -> Result<Self::EditRow>;
    fn db_delete(conn: &DbConn, edit_context: &EditContext, ident: FatCatId) -> Result<Self::EditRow>;
    fn db_get_history(conn: &DbConn, ident: FatCatId, limit: Option<i64>) -> Result<Vec<EntityHistoryEntry>>;

    // Entity-specific Methods
    fn db_from_row(conn: &DbConn, rev_row: Self::RevRow, ident_row: Option<Self::IdentRow>) -> Result<Self>;
    fn db_insert_rev(&self, conn: &DbConn) -> Result<Uuid>;
}

// TODO: this could be a separate trait on all entities?
macro_rules! generic_parse_editgroup_id{
    () => {
        fn parse_editgroup_id(&self) -> Result<Option<FatCatId>> {
            match &self.editgroup_id {
                Some(s) => Ok(Some(FatCatId::from_str(&s)?)),
                None => Ok(None),
            }
        }
    }
}

macro_rules! generic_db_get {
    ($ident_table: ident, $rev_table: ident) => {
        fn db_get(conn: &DbConn, ident: FatCatId) -> Result<Self> {
            let (ident, rev): (Self::IdentRow, Self::RevRow) = $ident_table::table
                .find(ident.to_uuid())
                .inner_join($rev_table::table)
                .first(conn)?;

            Self::db_from_row(conn, rev, Some(ident))
        }
    }
}

macro_rules! generic_db_get_rev {
    ($rev_table: ident) => {
        fn db_get_rev(conn: &DbConn, rev_id: Uuid) -> Result<Self> {
            let rev = $rev_table::table
                .find(rev_id)
                .first(conn)?;
            
            Self::db_from_row(conn, rev, None)
        }
    }
}

macro_rules! generic_db_create_batch {
    () => {
        fn db_create_batch(conn: &DbConn, edit_context: &EditContext, models: &[Self]) -> Result<Vec<Self::EditRow>> {
            let mut ret: Vec<Self::EditRow> = vec![];
            for entity in models {
                ret.push(entity.db_create(conn, edit_context)?);
            }
            Ok(ret)
        }
    }
}

macro_rules! generic_db_update {
    ($ident_table: ident, $edit_table: ident) => {
        fn db_update(&self, conn: &DbConn, edit_context: &EditContext, ident: FatCatId) -> Result<Self::EditRow> {
            let current: Self::IdentRow = $ident_table::table.find(ident.to_uuid()).first(conn)?;
            if current.is_live != true {
                // TODO: what if isn't live? 4xx not 5xx
                bail!("can't delete an entity that doesn't exist yet");
            }
            if current.rev_id.is_none() {
                // TODO: what if it's already deleted? 4xx not 5xx
                bail!("entity was already deleted");
            }

            let rev_id = self.db_insert_rev(conn)?;
            let edit: Self::EditRow = insert_into($edit_table::table)
                .values((
                    $edit_table::editgroup_id.eq(edit_context.editgroup_id.to_uuid()),
                    $edit_table::ident_id.eq(&ident.to_uuid()),
                    $edit_table::rev_id.eq(&rev_id),
                    $edit_table::prev_rev.eq(current.rev_id.unwrap()),
                    $edit_table::extra_json.eq(&self.extra),
                ))
                .get_result(conn)?;

            Ok(edit)
        }
    }
}

macro_rules! generic_db_delete {
    ($ident_table: ident, $edit_table:ident) => {
        fn db_delete(conn: &DbConn, edit_context: &EditContext, ident: FatCatId) -> Result<Self::EditRow> {

            let current: Self::IdentRow = $ident_table::table.find(ident.to_uuid()).first(conn)?;
            if current.is_live != true {
                // TODO: what if isn't live? 4xx not 5xx
                bail!("can't delete an entity that doesn't exist yet");
            }
            if current.rev_id.is_none() {
                // TODO: what if it's already deleted? 4xx not 5xx
                bail!("entity was already deleted");
            }
            let edit: Self::EditRow = insert_into($edit_table::table)
                .values((
                    $edit_table::editgroup_id.eq(edit_context.editgroup_id.to_uuid()),
                    $edit_table::ident_id.eq(ident.to_uuid()),
                    $edit_table::rev_id.eq(None::<Uuid>),
                    $edit_table::redirect_id.eq(None::<Uuid>),
                    $edit_table::prev_rev.eq(current.rev_id),
                    $edit_table::extra_json.eq(&edit_context.extra_json),
                ))
                .get_result(conn)?;

            Ok(edit)
        }
    }
}

macro_rules! generic_db_get_history {
    // TODO: only actually need edit table? and maybe not that?
    ($edit_table:ident) => {
        fn db_get_history(conn: &DbConn, ident: FatCatId, limit: Option<i64>) -> Result<Vec<EntityHistoryEntry>> {
            let limit = limit.unwrap_or(50); // XXX: make a static

            let rows: Vec<(EditgroupRow, ChangelogRow, Self::EditRow)> = editgroup::table
                .inner_join(changelog::table)
                .inner_join($edit_table::table)
                .filter($edit_table::ident_id.eq(ident.to_uuid()))
                .order(changelog::id.desc())
                .limit(limit)
                .get_results(conn)?;

            let history: Vec<EntityHistoryEntry> = rows.into_iter()
                .map(|(eg_row, cl_row, e_row)| EntityHistoryEntry {
                    edit: e_row.into_model().expect("edit row to model"),
                    editgroup: eg_row.into_model_partial(),
                    changelog_entry: cl_row.into_model(),
                })
                .collect();
            Ok(history)
        }
    }
}

impl EntityCrud for WorkEntity {
    type EditRow = WorkEditRow;
    type IdentRow = WorkIdentRow;
    type RevRow = WorkRevRow;

    generic_parse_editgroup_id!();
    generic_db_get!(work_ident, work_rev);
    generic_db_get_rev!(work_rev);
    generic_db_create_batch!();
    generic_db_update!(work_ident, work_edit);
    generic_db_delete!(work_ident, work_edit);
    generic_db_get_history!(work_edit);

    fn db_create(&self, conn: &DbConn, edit_context: &EditContext) -> Result<Self::EditRow> {

        let rev_id = self.db_insert_rev(conn)?;
        let edit: WorkEditRow =
            diesel::sql_query(
                "WITH ident AS (
                    INSERT INTO work_ident (rev_id)
                           VALUES ($1)
                           RETURNING id )
            INSERT INTO work_edit (editgroup_id, ident_id, rev_id, extra) VALUES
                ($2, (SELECT ident.id FROM ident), $1, $3)
            RETURNING *",
            ).bind::<diesel::sql_types::Uuid, _>(rev_id)
                .bind::<diesel::sql_types::Uuid, _>(edit_context.editgroup_id.to_uuid())
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(&edit_context.extra_json)
                .get_result(conn)?;

        Ok(edit)
    }

    fn db_from_row(conn: &DbConn, rev_row: Self::RevRow, ident_row: Option<Self::IdentRow>) -> Result<Self> {

        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };

        Ok(WorkEntity {
            state: state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            editgroup_id: None,
            extra: rev_row.extra_json,
        })
    }

    fn db_insert_rev(&self, conn: &DbConn) -> Result<Uuid> {
        let rev_row: WorkRevRow = insert_into(work_rev::table)
            .values((
                work_rev::extra_json.eq(&self.extra)
            ))
            .get_result(conn)?;
        /* TODO: only return id
            .returning(work_rev::id)
            .first(conn)?;
        */
        Ok(rev_row.id)
    }
}

