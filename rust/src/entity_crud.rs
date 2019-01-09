use crate::database_models::*;
use crate::database_schema::*;
use crate::editing::*;
use crate::endpoint_handlers::get_release_files;
use crate::errors::*;
use crate::identifiers::*;
use crate::server::*;
use chrono;
use diesel::prelude::*;
use diesel::{self, insert_into};
use fatcat_api_spec::models::*;
use sha1::Sha1;
use std::marker::Sized;
use std::str::FromStr;
use uuid::Uuid;

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
 *   db_get_edit
 *   db_delete_edit
 *   db_get_redirects
 *   db_accept_edits
 *
 * For now, these will probably be macros, until we can level up our trait/generics foo.
 */

// Associated Type, not parametric
pub trait EntityCrud
where
    Self: Sized,
{
    // TODO: could EditRow and IdentRow be generic structs? Or do they need to be bound to a
    // specific table?
    type EditRow; // EntityEditRow
    type EditNewRow;
    type IdentRow; // EntityIdentRow
    type IdentNewRow;
    type RevRow;

    // Generic Methods
    fn from_deleted_row(ident_row: Self::IdentRow) -> Result<Self>;
    fn db_get(conn: &DbConn, ident: FatCatId, hide: HideFlags) -> Result<Self>;
    fn db_get_rev(conn: &DbConn, rev_id: Uuid, hide: HideFlags) -> Result<Self>;
    fn db_expand(&mut self, conn: &DbConn, expand: ExpandFlags) -> Result<()>;
    fn db_create(&self, conn: &DbConn, edit_context: &EditContext) -> Result<Self::EditRow>;
    fn db_create_batch(
        conn: &DbConn,
        edit_context: &EditContext,
        models: &[&Self],
    ) -> Result<Vec<Self::EditRow>>;
    fn db_update(
        &self,
        conn: &DbConn,
        edit_context: &EditContext,
        ident: FatCatId,
    ) -> Result<Self::EditRow>;
    fn db_delete(
        conn: &DbConn,
        edit_context: &EditContext,
        ident: FatCatId,
    ) -> Result<Self::EditRow>;
    fn db_get_history(
        conn: &DbConn,
        ident: FatCatId,
        limit: Option<i64>,
    ) -> Result<Vec<EntityHistoryEntry>>;
    fn db_get_edit(conn: &DbConn, edit_id: Uuid) -> Result<Self::EditRow>;
    fn db_delete_edit(conn: &DbConn, edit_id: Uuid) -> Result<()>;
    fn db_get_redirects(conn: &DbConn, ident: FatCatId) -> Result<Vec<FatCatId>>;
    fn db_accept_edits(conn: &DbConn, editgroup_id: FatCatId) -> Result<u64>;

    // Entity-specific Methods
    fn db_from_row(
        conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
        hide: HideFlags,
    ) -> Result<Self>;
    fn db_insert_rev(&self, conn: &DbConn) -> Result<Uuid>;
    fn db_insert_revs(conn: &DbConn, models: &[&Self]) -> Result<Vec<Uuid>>;
}

#[derive(Clone, Copy, PartialEq)]
pub struct ExpandFlags {
    pub files: bool,
    pub filesets: bool,
    pub webcaptures: bool,
    pub container: bool,
    pub releases: bool,
    pub creators: bool,
}

impl FromStr for ExpandFlags {
    type Err = Error;
    fn from_str(param: &str) -> Result<ExpandFlags> {
        let list: Vec<&str> = param.split_terminator(',').collect();
        Ok(ExpandFlags::from_str_list(&list))
    }
}

impl ExpandFlags {
    pub fn from_str_list(list: &[&str]) -> ExpandFlags {
        ExpandFlags {
            files: list.contains(&"files"),
            filesets: list.contains(&"filesets"),
            webcaptures: list.contains(&"webcaptures"),
            container: list.contains(&"container"),
            releases: list.contains(&"releases"),
            creators: list.contains(&"creators"),
        }
    }
    pub fn none() -> ExpandFlags {
        ExpandFlags {
            files: false,
            filesets: false,
            webcaptures: false,
            container: false,
            releases: false,
            creators: false,
        }
    }
}

#[test]
fn test_expand_flags() {
    assert!(ExpandFlags::from_str_list(&vec![]).files == false);
    assert!(ExpandFlags::from_str_list(&vec!["files"]).files == true);
    assert!(ExpandFlags::from_str_list(&vec!["file"]).files == false);
    let all = ExpandFlags::from_str_list(&vec![
        "files",
        "filesets",
        "webcaptures",
        "container",
        "other_thing",
        "releases",
        "creators",
    ]);
    assert!(
        all == ExpandFlags {
            files: true,
            filesets: true,
            webcaptures: true,
            container: true,
            releases: true,
            creators: true
        }
    );
    assert!(ExpandFlags::from_str("").unwrap().files == false);
    assert!(ExpandFlags::from_str("files").unwrap().files == true);
    assert!(ExpandFlags::from_str("something,,files").unwrap().files == true);
    assert!(ExpandFlags::from_str("file").unwrap().files == false);
    let all =
        ExpandFlags::from_str("files,container,other_thing,releases,creators,filesets,webcaptures")
            .unwrap();
    assert!(
        all == ExpandFlags {
            files: true,
            filesets: true,
            webcaptures: true,
            container: true,
            releases: true,
            creators: true
        }
    );
}

#[derive(Clone, Copy, PartialEq)]
pub struct HideFlags {
    // release
    pub abstracts: bool,
    pub refs: bool,
    pub contribs: bool,
    // fileset
    pub manifest: bool,
    // webcapture
    pub cdx: bool,
}

impl FromStr for HideFlags {
    type Err = Error;
    fn from_str(param: &str) -> Result<HideFlags> {
        let list: Vec<&str> = param.split_terminator(',').collect();
        Ok(HideFlags::from_str_list(&list))
    }
}

impl HideFlags {
    pub fn from_str_list(list: &[&str]) -> HideFlags {
        HideFlags {
            abstracts: list.contains(&"abstracts"),
            refs: list.contains(&"refs"),
            contribs: list.contains(&"contribs"),
            manifest: list.contains(&"contribs"),
            cdx: list.contains(&"contribs"),
        }
    }
    pub fn none() -> HideFlags {
        HideFlags {
            abstracts: false,
            refs: false,
            contribs: false,
            manifest: false,
            cdx: false,
        }
    }
}

#[test]
fn test_hide_flags() {
    assert!(HideFlags::from_str_list(&vec![]).abstracts == false);
    assert!(HideFlags::from_str_list(&vec!["abstracts"]).abstracts == true);
    assert!(HideFlags::from_str_list(&vec!["abstract"]).abstracts == false);
    let all = HideFlags::from_str_list(&vec![
        "abstracts",
        "refs",
        "other_thing",
        "contribs",
        "manifest",
        "cdx",
    ]);
    assert!(
        all == HideFlags {
            abstracts: true,
            refs: true,
            contribs: true,
            manifest: true,
            cdx: true,
        }
    );
    assert!(HideFlags::from_str("").unwrap().abstracts == false);
    assert!(HideFlags::from_str("abstracts").unwrap().abstracts == true);
    assert!(
        HideFlags::from_str("something,,abstracts")
            .unwrap()
            .abstracts
            == true
    );
    assert!(HideFlags::from_str("file").unwrap().abstracts == false);
    let all = HideFlags::from_str("abstracts,cdx,refs,manifest,other_thing,contribs").unwrap();
    assert!(
        all == HideFlags {
            abstracts: true,
            refs: true,
            contribs: true,
            manifest: true,
            cdx: true,
        }
    );
}

macro_rules! generic_db_get {
    ($ident_table:ident, $rev_table:ident) => {
        fn db_get(conn: &DbConn, ident: FatCatId, hide: HideFlags) -> Result<Self> {
            let res: Option<(Self::IdentRow, Self::RevRow)> = $ident_table::table
                .find(ident.to_uuid())
                .inner_join($rev_table::table)
                .first(conn)
                .optional()?;

            match res {
                Some((ident, rev)) => {
                    Self::db_from_row(conn, rev, Some(ident), hide)
                },
                None => {
                    // return a stub (deleted) entity if it's just deleted state
                    let ident_row: Self::IdentRow = $ident_table::table.find(ident.to_uuid()).first(conn)?;
                    if ident_row.rev_id.is_none() {
                        Self::from_deleted_row(ident_row)
                    } else {
                        bail!("unexpected condition: entity ident/rev join failed, yet row isn't in deleted state")
                    }
                },
            }
        }
    };
}

macro_rules! generic_db_get_rev {
    ($rev_table:ident) => {
        fn db_get_rev(conn: &DbConn, rev_id: Uuid, hide: HideFlags) -> Result<Self> {
            let rev = $rev_table::table.find(rev_id).first(conn)?;

            Self::db_from_row(conn, rev, None, hide)
        }
    };
}

macro_rules! generic_db_expand {
    () => {
        fn db_expand(&mut self, _conn: &DbConn, _expand: ExpandFlags) -> Result<()> {
            Ok(())
        }
    };
}

macro_rules! generic_db_create {
    // TODO: this path should call generic_db_create_batch
    ($ident_table: ident, $edit_table: ident) => {
        fn db_create(&self, conn: &DbConn, edit_context: &EditContext) -> Result<Self::EditRow> {
            if self.redirect.is_some() {
                return Err(ErrorKind::OtherBadRequest(
                    "can't create an entity that redirects from the start".to_string()).into());
            }
            let rev_id = self.db_insert_rev(conn)?;
            let ident: Uuid = insert_into($ident_table::table)
                .values($ident_table::rev_id.eq(&rev_id))
                .returning($ident_table::id)
                .get_result(conn)?;
            let edit: Self::EditRow = insert_into($edit_table::table)
                .values((
                    $edit_table::editgroup_id.eq(edit_context.editgroup_id.to_uuid()),
                    $edit_table::rev_id.eq(&rev_id),
                    $edit_table::ident_id.eq(&ident),
                ))
                .get_result(conn)?;
            Ok(edit)
        }
    }
}

macro_rules! generic_db_create_batch {
    ($ident_table:ident, $edit_table:ident) => {
        fn db_create_batch(
            conn: &DbConn,
            edit_context: &EditContext,
            models: &[&Self],
        ) -> Result<Vec<Self::EditRow>> {
            if models.iter().any(|m| m.redirect.is_some()) {
                return Err(ErrorKind::OtherBadRequest(
                    "can't create an entity that redirects from the start".to_string(),
                )
                .into());
            }
            let rev_ids: Vec<Uuid> = Self::db_insert_revs(conn, models)?;
            let ident_ids: Vec<Uuid> = insert_into($ident_table::table)
                .values(
                    rev_ids
                        .iter()
                        .map(|rev_id| Self::IdentNewRow {
                            rev_id: Some(rev_id.clone()),
                            is_live: edit_context.autoaccept,
                            redirect_id: None,
                        })
                        .collect::<Vec<Self::IdentNewRow>>(),
                )
                .returning($ident_table::id)
                .get_results(conn)?;
            let edits: Vec<Self::EditRow> = insert_into($edit_table::table)
                .values(
                    rev_ids
                        .into_iter()
                        .zip(ident_ids.into_iter())
                        .map(|(rev_id, ident_id)| Self::EditNewRow {
                            editgroup_id: edit_context.editgroup_id.to_uuid(),
                            rev_id: Some(rev_id),
                            ident_id,
                            redirect_id: None,
                            prev_rev: None,
                            extra_json: edit_context.extra_json.clone(),
                        })
                        .collect::<Vec<Self::EditNewRow>>(),
                )
                .get_results(conn)?;
            Ok(edits)
        }
    };
}

macro_rules! generic_db_update {
    ($ident_table: ident, $edit_table: ident) => {
        fn db_update(&self, conn: &DbConn, edit_context: &EditContext, ident: FatCatId) -> Result<Self::EditRow> {
            let current: Self::IdentRow = $ident_table::table.find(ident.to_uuid()).first(conn)?;
            let no_redirect: Option<Uuid> = None;
            // TODO: is this actually true? or should we allow updates in the same editgroup?
            if current.is_live != true {
                return Err(ErrorKind::InvalidEntityStateTransform(
                    "can't update an entity that doesn't exist yet".to_string()).into());
            }
            // Don't set prev_rev if current status is redirect
            let prev_rev = match current.redirect_id {
                Some(_) => None,
                None => current.rev_id,
            };

            if self.state.is_none() {

                if Some(ident.to_string()) == self.redirect {
                    return Err(ErrorKind::OtherBadRequest(
                        "tried to redirect entity to itself".to_string()).into());
                }
                // special case: redirect to another entity
                if let Some(ref redirect_ident) = self.redirect {
                    let redirect_ident = FatCatId::from_str(&redirect_ident)?.to_uuid();
                    if Some(redirect_ident) == current.redirect_id {
                        return Err(ErrorKind::OtherBadRequest(
                            "redundantly redirecting entity to it's current target currently isn't supported".to_string()).into());
                    }
                    // TODO: if we get a diesel not-found here, should be a special error response?
                    let target: Self::IdentRow = $ident_table::table.find(redirect_ident).first(conn)?;
                    if target.is_live != true {
                        // there is no race condition on this check because WIP -> is_live=true is
                        // a one-way operation
                        // XXX:
                        return Err(ErrorKind::OtherBadRequest(
                            "attempted to redirect to a WIP entity".to_string()).into());
                    }
                    // Note: there is a condition where the target is already a redirect, but we
                    // don't handle that here because the state of the redirect could change before
                    // we accept this editgroup
                    let edit: Self::EditRow = insert_into($edit_table::table)
                        .values((
                            $edit_table::editgroup_id.eq(edit_context.editgroup_id.to_uuid()),
                            $edit_table::ident_id.eq(&ident.to_uuid()),
                            $edit_table::rev_id.eq(target.rev_id),
                            $edit_table::redirect_id.eq(redirect_ident),
                            $edit_table::prev_rev.eq(prev_rev),
                            $edit_table::extra_json.eq(&self.edit_extra),
                        ))
                        .get_result(conn)?;
                    return Ok(edit)
                }
                // special case: revert to point to an existing revision
                if let Some(ref rev_id) = self.revision {
                    let rev_id = Uuid::from_str(&rev_id)?;
                    if Some(rev_id) == current.rev_id {
                        return Err(ErrorKind::OtherBadRequest(
                            "reverted entity to it's current state; this isn't currently supported".to_string()).into());
                    }
                    let edit: Self::EditRow = insert_into($edit_table::table)
                        .values((
                            $edit_table::editgroup_id.eq(edit_context.editgroup_id.to_uuid()),
                            $edit_table::ident_id.eq(&ident.to_uuid()),
                            $edit_table::rev_id.eq(&rev_id),
                            $edit_table::redirect_id.eq(no_redirect),
                            $edit_table::prev_rev.eq(prev_rev),
                            $edit_table::extra_json.eq(&self.edit_extra),
                        ))
                        .get_result(conn)?;
                    return Ok(edit)
                }
            }

            // regular insert/update
            let rev_id = self.db_insert_rev(conn)?;
            let edit: Self::EditRow = insert_into($edit_table::table)
                .values((
                    $edit_table::editgroup_id.eq(edit_context.editgroup_id.to_uuid()),
                    $edit_table::ident_id.eq(&ident.to_uuid()),
                    $edit_table::rev_id.eq(&rev_id),
                    $edit_table::redirect_id.eq(no_redirect),
                    $edit_table::prev_rev.eq(prev_rev),
                    $edit_table::extra_json.eq(&self.edit_extra),
                ))
                .get_result(conn)?;
            Ok(edit)
        }
    }
}

macro_rules! generic_db_delete {
    ($ident_table:ident, $edit_table:ident) => {
        fn db_delete(
            conn: &DbConn,
            edit_context: &EditContext,
            ident: FatCatId,
        ) -> Result<Self::EditRow> {
            let current: Self::IdentRow = $ident_table::table.find(ident.to_uuid()).first(conn)?;
            if current.is_live != true {
                return Err(ErrorKind::InvalidEntityStateTransform(
                    "can't update an entity that doesn't exist yet; delete edit object instead"
                        .to_string(),
                )
                .into());
            }
            if current.state()? == EntityState::Deleted {
                return Err(ErrorKind::InvalidEntityStateTransform(
                    "entity was already deleted".to_string(),
                )
                .into());
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
    };
}

macro_rules! generic_db_get_history {
    ($edit_table:ident) => {
        fn db_get_history(
            conn: &DbConn,
            ident: FatCatId,
            limit: Option<i64>,
        ) -> Result<Vec<EntityHistoryEntry>> {
            let limit = limit.unwrap_or(50); // TODO: make a static

            let rows: Vec<(EditgroupRow, ChangelogRow, Self::EditRow)> = editgroup::table
                .inner_join(changelog::table)
                .inner_join($edit_table::table)
                .filter($edit_table::ident_id.eq(ident.to_uuid()))
                .order(changelog::id.desc())
                .limit(limit)
                .get_results(conn)?;

            let history: Result<Vec<EntityHistoryEntry>> = rows
                .into_iter()
                .map(|(eg_row, cl_row, e_row)| {
                    Ok(EntityHistoryEntry {
                        edit: e_row.into_model()?,
                        editgroup: eg_row.into_model_partial(),
                        changelog_entry: cl_row.into_model(),
                    })
                })
                .collect();
            history
        }
    };
}

macro_rules! generic_db_get_edit {
    ($edit_table:ident) => {
        fn db_get_edit(conn: &DbConn, edit_id: Uuid) -> Result<Self::EditRow> {
            Ok($edit_table::table.find(edit_id).first(conn)?)
        }
    };
}

macro_rules! generic_db_delete_edit {
    ($edit_table:ident) => {
        /// This method assumes the connection is already in a transaction
        fn db_delete_edit(conn: &DbConn, edit_id: Uuid) -> Result<()> {
            // ensure that edit hasn't been accepted
            let accepted_rows: Vec<(EditgroupRow, ChangelogRow, Self::EditRow)> = editgroup::table
                .inner_join(changelog::table)
                .inner_join($edit_table::table)
                .filter($edit_table::id.eq(edit_id))
                .limit(1)
                .get_results(conn)?;
            if accepted_rows.len() != 0 {
                return Err(ErrorKind::EditgroupAlreadyAccepted(
                    "attempted to delete an already accepted edit".to_string(),
                )
                .into());
            }
            diesel::delete($edit_table::table.filter($edit_table::id.eq(edit_id))).execute(conn)?;
            Ok(())
        }
    };
}

macro_rules! generic_db_get_redirects {
    ($ident_table:ident) => {
        fn db_get_redirects(conn: &DbConn, ident: FatCatId) -> Result<Vec<FatCatId>> {
            let res: Vec<Uuid> = $ident_table::table
                .select($ident_table::id)
                .filter($ident_table::redirect_id.eq(ident.to_uuid()))
                .get_results(conn)?;
            Ok(res.iter().map(|u| FatCatId::from_uuid(u)).collect())
        }
    };
}

/*
// This would be the clean and efficient way, but see:
// https://github.com/diesel-rs/diesel/issues/1478
//
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

// Was previously:

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
*/

// UPDATE FROM version: single query for many rows
// Works with Postgres, not Cockroach
#[allow(unused_macros)]
macro_rules! generic_db_accept_edits_batch {
    ($entity_name_str:expr, $ident_table:ident, $edit_table:ident) => {
        fn db_accept_edits(conn: &DbConn, editgroup_id: FatCatId) -> Result<u64> {
            // NOTE: the checks and redirects can be skipped for accepts that are all inserts
            // (which I guess we only know for batch inserts with auto-accept?)

            // assert that we aren't redirecting to anything which is a redirect already
            let forward_recursive_redirects: i64 = $edit_table::table
                .inner_join(
                    $ident_table::table
                        .on($edit_table::redirect_id.eq($ident_table::id.nullable())),
                )
                .filter($edit_table::redirect_id.is_not_null())
                .filter($edit_table::editgroup_id.eq(&editgroup_id.to_uuid()))
                .filter($ident_table::redirect_id.is_not_null())
                .count()
                .get_result(conn)?;
            if forward_recursive_redirects != 0 {
                // TODO: revert transaction?
                return Err(ErrorKind::OtherBadRequest(
                    "one or more (forward) recurisve redirects".to_string(),
                )
                .into());
            }

            // assert that we aren't redirecting while something already redirects to us
            let backward_recursive_redirects: i64 = $ident_table::table
                .inner_join(
                    $edit_table::table
                        .on($ident_table::redirect_id.eq($edit_table::ident_id.nullable())),
                )
                .filter($ident_table::redirect_id.is_not_null())
                .filter($edit_table::editgroup_id.eq(editgroup_id.to_uuid()))
                .filter($edit_table::redirect_id.is_not_null())
                .count()
                .get_result(conn)?;
            if backward_recursive_redirects != 0 {
                // TODO: revert transaction?
                return Err(ErrorKind::OtherBadRequest(
                    "one or more (backward) recurisve redirects".to_string(),
                )
                .into());
            }

            let count = diesel::sql_query(format!(
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
                entity = $entity_name_str
            ))
            .bind::<diesel::sql_types::Uuid, _>(editgroup_id.to_uuid())
            .execute(conn)?;

            // update any/all redirects for updated entities
            let _redir_count = diesel::sql_query(format!(
                "
                    UPDATE {entity}_ident
                    SET
                        rev_id = {entity}_edit.rev_id
                    FROM {entity}_edit
                    WHERE
                        {entity}_ident.redirect_id = {entity}_edit.ident_id
                        AND {entity}_edit.editgroup_id = $1",
                entity = $entity_name_str
            ))
            .bind::<diesel::sql_types::Uuid, _>(editgroup_id.to_uuid())
            .execute(conn)?;
            Ok(count as u64)
        }
    };
}

// UPDATE ROW version: single query per row
// CockroachDB version (slow, single query per row)
#[allow(unused_macros)]
macro_rules! generic_db_accept_edits_each {
    ($ident_table:ident, $edit_table:ident) => {
        fn db_accept_edits(conn: &DbConn, editgroup_id: FatCatId) -> Result<u64> {
            // 1. select edit rows (in sql)
            let edit_rows: Vec<Self::EditRow> = $edit_table::table
                .filter($edit_table::editgroup_id.eq(&editgroup_id.to_uuid()))
                .get_results(conn)?;
            // 2. create ident rows (in rust)
            let ident_rows: Vec<Self::IdentRow> = edit_rows
                .iter()
                .map(|edit| Self::IdentRow {
                    id: edit.ident_id,
                    is_live: true,
                    rev_id: edit.rev_id,
                    redirect_id: edit.redirect_id,
                })
                .collect();
            /*
            // 3. upsert ident rows (in sql)
            let count: u64 = diesel::insert_into($ident_table::table)
                .values(ident_rows)
                .on_conflict()
                .do_update()
                .set(ident_rows)
                .execute(conn)?;
            */
            // 3. update every row individually
            let count = ident_rows.len() as u64;
            for row in ident_rows {
                diesel::update(&row).set(&row).execute(conn)?;
            }
            Ok(count)
        }
    };
}

macro_rules! generic_db_insert_rev {
    () => {
        fn db_insert_rev(&self, conn: &DbConn) -> Result<Uuid> {
            Self::db_insert_revs(conn, &[self]).map(|id_list| id_list[0])
        }
    }
}

impl EntityCrud for ContainerEntity {
    type EditRow = ContainerEditRow;
    type EditNewRow = ContainerEditNewRow;
    type IdentRow = ContainerIdentRow;
    type IdentNewRow = ContainerIdentNewRow;
    type RevRow = ContainerRevRow;

    generic_db_get!(container_ident, container_rev);
    generic_db_get_rev!(container_rev);
    generic_db_expand!();
    generic_db_create!(container_ident, container_edit);
    generic_db_create_batch!(container_ident, container_edit);
    generic_db_update!(container_ident, container_edit);
    generic_db_delete!(container_ident, container_edit);
    generic_db_get_history!(container_edit);
    generic_db_get_edit!(container_edit);
    generic_db_delete_edit!(container_edit);
    generic_db_get_redirects!(container_ident);
    generic_db_accept_edits_batch!("container", container_ident, container_edit);
    generic_db_insert_rev!();

    fn from_deleted_row(ident_row: Self::IdentRow) -> Result<Self> {
        if ident_row.rev_id.is_some() {
            bail!("called from_deleted_row with a non-deleted-state row")
        }

        Ok(ContainerEntity {
            issnl: None,
            wikidata_qid: None,
            publisher: None,
            name: None,
            abbrev: None,
            coden: None,
            state: Some(ident_row.state().unwrap().shortname()),
            ident: Some(FatCatId::from_uuid(&ident_row.id).to_string()),
            revision: ident_row.rev_id.map(|u| u.to_string()),
            redirect: ident_row
                .redirect_id
                .map(|u| FatCatId::from_uuid(&u).to_string()),
            extra: None,
            edit_extra: None,
        })
    }

    fn db_from_row(
        _conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
        _hide: HideFlags,
    ) -> Result<Self> {
        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };

        Ok(ContainerEntity {
            issnl: rev_row.issnl,
            wikidata_qid: rev_row.wikidata_qid,
            publisher: rev_row.publisher,
            name: Some(rev_row.name),
            abbrev: rev_row.abbrev,
            coden: rev_row.coden,
            state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            extra: rev_row.extra_json,
            edit_extra: None,
        })
    }

    fn db_insert_revs(conn: &DbConn, models: &[&Self]) -> Result<Vec<Uuid>> {
        // first verify external identifier syntax
        for entity in models {
            if let Some(ref extid) = entity.wikidata_qid {
                check_wikidata_qid(extid)?;
            }
            if let Some(ref extid) = entity.issnl {
                check_issn(extid)?;
            }
        }

        if models.iter().any(|m| m.name.is_none()) {
            return Err(ErrorKind::OtherBadRequest(
                "name is required for all Container entities".to_string(),
            )
            .into());
        }

        let rev_ids: Vec<Uuid> = insert_into(container_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| ContainerRevNewRow {
                        name: model.name.clone().unwrap(), // unwrap checked above
                        publisher: model.publisher.clone(),
                        issnl: model.issnl.clone(),
                        wikidata_qid: model.wikidata_qid.clone(),
                        abbrev: model.abbrev.clone(),
                        coden: model.coden.clone(),
                        extra_json: model.extra.clone(),
                    })
                    .collect::<Vec<ContainerRevNewRow>>(),
            )
            .returning(container_rev::id)
            .get_results(conn)?;
        Ok(rev_ids)
    }
}

impl EntityCrud for CreatorEntity {
    type EditRow = CreatorEditRow;
    type EditNewRow = CreatorEditNewRow;
    type IdentRow = CreatorIdentRow;
    type IdentNewRow = CreatorIdentNewRow;
    type RevRow = CreatorRevRow;

    generic_db_get!(creator_ident, creator_rev);
    generic_db_get_rev!(creator_rev);
    generic_db_expand!();
    generic_db_create!(creator_ident, creator_edit);
    generic_db_create_batch!(creator_ident, creator_edit);
    generic_db_update!(creator_ident, creator_edit);
    generic_db_delete!(creator_ident, creator_edit);
    generic_db_get_history!(creator_edit);
    generic_db_get_edit!(creator_edit);
    generic_db_delete_edit!(creator_edit);
    generic_db_get_redirects!(creator_ident);
    generic_db_accept_edits_batch!("creator", creator_ident, creator_edit);
    generic_db_insert_rev!();

    fn from_deleted_row(ident_row: Self::IdentRow) -> Result<Self> {
        if ident_row.rev_id.is_some() {
            bail!("called from_deleted_row with a non-deleted-state row")
        }

        Ok(CreatorEntity {
            extra: None,
            edit_extra: None,
            display_name: None,
            given_name: None,
            surname: None,
            orcid: None,
            wikidata_qid: None,
            state: Some(ident_row.state().unwrap().shortname()),
            ident: Some(FatCatId::from_uuid(&ident_row.id).to_string()),
            revision: ident_row.rev_id.map(|u| u.to_string()),
            redirect: ident_row
                .redirect_id
                .map(|u| FatCatId::from_uuid(&u).to_string()),
        })
    }

    fn db_from_row(
        _conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
        _hide: HideFlags,
    ) -> Result<Self> {
        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };
        Ok(CreatorEntity {
            display_name: Some(rev_row.display_name),
            given_name: rev_row.given_name,
            surname: rev_row.surname,
            orcid: rev_row.orcid,
            wikidata_qid: rev_row.wikidata_qid,
            state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            extra: rev_row.extra_json,
            edit_extra: None,
        })
    }

    fn db_insert_revs(conn: &DbConn, models: &[&Self]) -> Result<Vec<Uuid>> {
        // first verify external identifier syntax
        for entity in models {
            if let Some(ref extid) = entity.orcid {
                check_orcid(extid)?;
            }
            if let Some(ref extid) = entity.wikidata_qid {
                check_wikidata_qid(extid)?;
            }
        }

        if models.iter().any(|m| m.display_name.is_none()) {
            return Err(ErrorKind::OtherBadRequest(
                "display_name is required for all Creator entities".to_string(),
            )
            .into());
        }

        let rev_ids: Vec<Uuid> = insert_into(creator_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| CreatorRevNewRow {
                        display_name: model.display_name.clone().unwrap(), // unwrapped checked above
                        given_name: model.given_name.clone(),
                        surname: model.surname.clone(),
                        orcid: model.orcid.clone(),
                        wikidata_qid: model.wikidata_qid.clone(),
                        extra_json: model.extra.clone(),
                    })
                    .collect::<Vec<CreatorRevNewRow>>(),
            )
            .returning(creator_rev::id)
            .get_results(conn)?;
        Ok(rev_ids)
    }
}

impl EntityCrud for FileEntity {
    type EditRow = FileEditRow;
    type EditNewRow = FileEditNewRow;
    type IdentRow = FileIdentRow;
    type IdentNewRow = FileIdentNewRow;
    type RevRow = FileRevRow;

    generic_db_get!(file_ident, file_rev);
    generic_db_get_rev!(file_rev);
    generic_db_expand!();
    generic_db_create!(file_ident, file_edit);
    generic_db_create_batch!(file_ident, file_edit);
    generic_db_update!(file_ident, file_edit);
    generic_db_delete!(file_ident, file_edit);
    generic_db_get_history!(file_edit);
    generic_db_get_edit!(file_edit);
    generic_db_delete_edit!(file_edit);
    generic_db_get_redirects!(file_ident);
    generic_db_accept_edits_batch!("file", file_ident, file_edit);
    generic_db_insert_rev!();

    fn from_deleted_row(ident_row: Self::IdentRow) -> Result<Self> {
        if ident_row.rev_id.is_some() {
            bail!("called from_deleted_row with a non-deleted-state row")
        }

        Ok(FileEntity {
            sha1: None,
            sha256: None,
            md5: None,
            size: None,
            urls: None,
            mimetype: None,
            release_ids: None,
            state: Some(ident_row.state().unwrap().shortname()),
            ident: Some(FatCatId::from_uuid(&ident_row.id).to_string()),
            revision: ident_row.rev_id.map(|u| u.to_string()),
            redirect: ident_row
                .redirect_id
                .map(|u| FatCatId::from_uuid(&u).to_string()),
            extra: None,
            edit_extra: None,
        })
    }

    fn db_from_row(
        conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
        _hide: HideFlags,
    ) -> Result<Self> {
        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };

        let urls: Vec<FileEntityUrls> = file_rev_url::table
            .filter(file_rev_url::file_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|r: FileRevUrlRow| FileEntityUrls {
                rel: r.rel,
                url: r.url,
            })
            .collect();

        let release_ids: Vec<FatCatId> = file_rev_release::table
            .filter(file_rev_release::file_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|r: FileRevReleaseRow| FatCatId::from_uuid(&r.target_release_ident_id))
            .collect();

        Ok(FileEntity {
            sha1: rev_row.sha1,
            sha256: rev_row.sha256,
            md5: rev_row.md5,
            size: rev_row.size_bytes.map(|v| v as i64),
            urls: Some(urls),
            mimetype: rev_row.mimetype,
            release_ids: Some(release_ids.iter().map(|fcid| fcid.to_string()).collect()),
            state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            extra: rev_row.extra_json,
            edit_extra: None,
        })
    }

    fn db_insert_revs(conn: &DbConn, models: &[&Self]) -> Result<Vec<Uuid>> {
        // first verify hash syntax
        for entity in models {
            if let Some(ref hash) = entity.md5 {
                check_md5(hash)?;
            }
            if let Some(ref hash) = entity.sha1 {
                check_sha1(hash)?;
            }
            if let Some(ref hash) = entity.sha256 {
                check_sha256(hash)?;
            }
        }

        let rev_ids: Vec<Uuid> = insert_into(file_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| FileRevNewRow {
                        size_bytes: model.size,
                        sha1: model.sha1.clone(),
                        sha256: model.sha256.clone(),
                        md5: model.md5.clone(),
                        mimetype: model.mimetype.clone(),
                        extra_json: model.extra.clone(),
                    })
                    .collect::<Vec<FileRevNewRow>>(),
            )
            .returning(file_rev::id)
            .get_results(conn)?;

        let mut file_rev_release_rows: Vec<FileRevReleaseRow> = vec![];
        let mut file_url_rows: Vec<FileRevUrlNewRow> = vec![];

        for (model, rev_id) in models.iter().zip(rev_ids.iter()) {
            match &model.release_ids {
                None => (),
                Some(release_list) => {
                    let these_release_rows: Result<Vec<FileRevReleaseRow>> = release_list
                        .iter()
                        .map(|r| {
                            Ok(FileRevReleaseRow {
                                file_rev: *rev_id,
                                target_release_ident_id: FatCatId::from_str(r)?.to_uuid(),
                            })
                        })
                        .collect();
                    file_rev_release_rows.extend(these_release_rows?);
                }
            };

            match &model.urls {
                None => (),
                Some(url_list) => {
                    let these_url_rows: Vec<FileRevUrlNewRow> = url_list
                        .into_iter()
                        .map(|u| FileRevUrlNewRow {
                            file_rev: *rev_id,
                            rel: u.rel.clone(),
                            url: u.url.clone(),
                        })
                        .collect();
                    file_url_rows.extend(these_url_rows);
                }
            };
        }

        if !file_rev_release_rows.is_empty() {
            insert_into(file_rev_release::table)
                .values(file_rev_release_rows)
                .execute(conn)?;
        }

        if !file_url_rows.is_empty() {
            insert_into(file_rev_url::table)
                .values(file_url_rows)
                .execute(conn)?;
        }

        Ok(rev_ids)
    }
}

impl EntityCrud for FilesetEntity {
    type EditRow = FilesetEditRow;
    type EditNewRow = FilesetEditNewRow;
    type IdentRow = FilesetIdentRow;
    type IdentNewRow = FilesetIdentNewRow;
    type RevRow = FilesetRevRow;

    generic_db_get!(fileset_ident, fileset_rev);
    generic_db_get_rev!(fileset_rev);
    generic_db_expand!();
    generic_db_create!(fileset_ident, fileset_edit);
    generic_db_create_batch!(fileset_ident, fileset_edit);
    generic_db_update!(fileset_ident, fileset_edit);
    generic_db_delete!(fileset_ident, fileset_edit);
    generic_db_get_history!(fileset_edit);
    generic_db_get_edit!(fileset_edit);
    generic_db_delete_edit!(fileset_edit);
    generic_db_get_redirects!(fileset_ident);
    generic_db_accept_edits_batch!("fileset", fileset_ident, fileset_edit);
    generic_db_insert_rev!();

    fn from_deleted_row(ident_row: Self::IdentRow) -> Result<Self> {
        if ident_row.rev_id.is_some() {
            bail!("called from_deleted_row with a non-deleted-state row")
        }

        Ok(FilesetEntity {
            manifest: None,
            urls: None,
            release_ids: None,
            state: Some(ident_row.state().unwrap().shortname()),
            ident: Some(FatCatId::from_uuid(&ident_row.id).to_string()),
            revision: ident_row.rev_id.map(|u| u.to_string()),
            redirect: ident_row
                .redirect_id
                .map(|u| FatCatId::from_uuid(&u).to_string()),
            extra: None,
            edit_extra: None,
        })
    }

    fn db_from_row(
        conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
        _hide: HideFlags,
    ) -> Result<Self> {
        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };

        let manifest: Vec<FilesetEntityManifest> = fileset_rev_file::table
            .filter(fileset_rev_file::fileset_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|r: FilesetRevFileRow| FilesetEntityManifest {
                path: r.path_name,
                size: r.size_bytes,
                md5: r.md5,
                sha1: r.sha1,
                sha256: r.sha256,
                extra: r.extra_json,
            })
            .collect();

        let urls: Vec<FileEntityUrls> = fileset_rev_url::table
            .filter(fileset_rev_url::fileset_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|r: FilesetRevUrlRow| FileEntityUrls {
                rel: r.rel,
                url: r.url,
            })
            .collect();

        let release_ids: Vec<FatCatId> = fileset_rev_release::table
            .filter(fileset_rev_release::fileset_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|r: FilesetRevReleaseRow| FatCatId::from_uuid(&r.target_release_ident_id))
            .collect();

        Ok(FilesetEntity {
            manifest: Some(manifest),
            urls: Some(urls),
            release_ids: Some(release_ids.iter().map(|fcid| fcid.to_string()).collect()),
            state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            extra: rev_row.extra_json,
            edit_extra: None,
        })
    }

    fn db_insert_revs(conn: &DbConn, models: &[&Self]) -> Result<Vec<Uuid>> {
        // first verify hash syntax
        for entity in models {
            if let Some(ref manifest) = entity.manifest {
                for file in manifest {
                    if let Some(ref hash) = file.md5 {
                        check_md5(hash)?;
                    }
                    if let Some(ref hash) = file.sha1 {
                        check_sha1(hash)?;
                    }
                    if let Some(ref hash) = file.sha256 {
                        check_sha256(hash)?;
                    }
                }
            }
        }

        let rev_ids: Vec<Uuid> = insert_into(fileset_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| FilesetRevNewRow {
                        extra_json: model.extra.clone(),
                    })
                    .collect::<Vec<FilesetRevNewRow>>(),
            )
            .returning(fileset_rev::id)
            .get_results(conn)?;

        let mut fileset_file_rows: Vec<FilesetRevFileNewRow> = vec![];
        let mut fileset_url_rows: Vec<FilesetRevUrlNewRow> = vec![];
        let mut fileset_release_rows: Vec<FilesetRevReleaseRow> = vec![];

        for (model, rev_id) in models.iter().zip(rev_ids.iter()) {
            match &model.manifest {
                None => (),
                Some(file_list) => {
                    let these_file_rows: Vec<FilesetRevFileNewRow> = file_list
                        .into_iter()
                        .map(|f| FilesetRevFileNewRow {
                            fileset_rev: *rev_id,
                            path_name: f.path.clone(),
                            size_bytes: f.size,
                            md5: f.md5.clone(),
                            sha1: f.sha1.clone(),
                            sha256: f.sha256.clone(),
                            extra_json: f.extra.clone(),
                        })
                        .collect();
                    fileset_file_rows.extend(these_file_rows);
                }
            };

            match &model.urls {
                None => (),
                Some(url_list) => {
                    let these_url_rows: Vec<FilesetRevUrlNewRow> = url_list
                        .into_iter()
                        .map(|u| FilesetRevUrlNewRow {
                            fileset_rev: *rev_id,
                            rel: u.rel.clone(),
                            url: u.url.clone(),
                        })
                        .collect();
                    fileset_url_rows.extend(these_url_rows);
                }
            };

            match &model.release_ids {
                None => (),
                Some(release_list) => {
                    let these_release_rows: Result<Vec<FilesetRevReleaseRow>> = release_list
                        .iter()
                        .map(|r| {
                            Ok(FilesetRevReleaseRow {
                                fileset_rev: *rev_id,
                                target_release_ident_id: FatCatId::from_str(r)?.to_uuid(),
                            })
                        })
                        .collect();
                    fileset_release_rows.extend(these_release_rows?);
                }
            };
        }

        if !fileset_file_rows.is_empty() {
            insert_into(fileset_rev_file::table)
                .values(fileset_file_rows)
                .execute(conn)?;
        }

        if !fileset_url_rows.is_empty() {
            insert_into(fileset_rev_url::table)
                .values(fileset_url_rows)
                .execute(conn)?;
        }

        if !fileset_release_rows.is_empty() {
            insert_into(fileset_rev_release::table)
                .values(fileset_release_rows)
                .execute(conn)?;
        }

        Ok(rev_ids)
    }
}

impl EntityCrud for WebcaptureEntity {
    type EditRow = WebcaptureEditRow;
    type EditNewRow = WebcaptureEditNewRow;
    type IdentRow = WebcaptureIdentRow;
    type IdentNewRow = WebcaptureIdentNewRow;
    type RevRow = WebcaptureRevRow;

    generic_db_get!(webcapture_ident, webcapture_rev);
    generic_db_get_rev!(webcapture_rev);
    generic_db_expand!();
    generic_db_create!(webcapture_ident, webcapture_edit);
    generic_db_create_batch!(webcapture_ident, webcapture_edit);
    generic_db_update!(webcapture_ident, webcapture_edit);
    generic_db_delete!(webcapture_ident, webcapture_edit);
    generic_db_get_history!(webcapture_edit);
    generic_db_get_edit!(webcapture_edit);
    generic_db_delete_edit!(webcapture_edit);
    generic_db_get_redirects!(webcapture_ident);
    generic_db_accept_edits_batch!("webcapture", webcapture_ident, webcapture_edit);
    generic_db_insert_rev!();

    fn from_deleted_row(ident_row: Self::IdentRow) -> Result<Self> {
        if ident_row.rev_id.is_some() {
            bail!("called from_deleted_row with a non-deleted-state row")
        }

        Ok(WebcaptureEntity {
            cdx: None,
            archive_urls: None,
            original_url: None,
            timestamp: None,
            release_ids: None,
            state: Some(ident_row.state().unwrap().shortname()),
            ident: Some(FatCatId::from_uuid(&ident_row.id).to_string()),
            revision: ident_row.rev_id.map(|u| u.to_string()),
            redirect: ident_row
                .redirect_id
                .map(|u| FatCatId::from_uuid(&u).to_string()),
            extra: None,
            edit_extra: None,
        })
    }

    fn db_from_row(
        conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
        _hide: HideFlags,
    ) -> Result<Self> {
        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };

        let cdx: Vec<WebcaptureEntityCdx> = webcapture_rev_cdx::table
            .filter(webcapture_rev_cdx::webcapture_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|c: WebcaptureRevCdxRow| WebcaptureEntityCdx {
                surt: c.surt,
                timestamp: c.timestamp,
                url: c.url,
                mimetype: c.mimetype,
                status_code: c.status_code,
                sha1: c.sha1,
                sha256: c.sha256,
            })
            .collect();

        let archive_urls: Vec<WebcaptureEntityArchiveUrls> = webcapture_rev_url::table
            .filter(webcapture_rev_url::webcapture_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|r: WebcaptureRevUrlRow| WebcaptureEntityArchiveUrls {
                rel: r.rel,
                url: r.url,
            })
            .collect();

        let release_ids: Vec<FatCatId> = webcapture_rev_release::table
            .filter(webcapture_rev_release::webcapture_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|r: WebcaptureRevReleaseRow| FatCatId::from_uuid(&r.target_release_ident_id))
            .collect();

        Ok(WebcaptureEntity {
            cdx: Some(cdx),
            archive_urls: Some(archive_urls),
            original_url: Some(rev_row.original_url),
            timestamp: Some(chrono::DateTime::from_utc(rev_row.timestamp, chrono::Utc)),
            release_ids: Some(release_ids.iter().map(|fcid| fcid.to_string()).collect()),
            state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            extra: rev_row.extra_json,
            edit_extra: None,
        })
    }

    fn db_insert_revs(conn: &DbConn, models: &[&Self]) -> Result<Vec<Uuid>> {
        // first verify hash syntax, and presence of required fields
        for entity in models {
            if let Some(ref cdx) = entity.cdx {
                for row in cdx {
                    check_sha1(&row.sha1)?;
                    if let Some(ref hash) = row.sha256 {
                        check_sha256(hash)?;
                    }
                }
            }
            if entity.timestamp.is_none() || entity.original_url.is_none() {
                return Err(ErrorKind::OtherBadRequest(
                    "timestamp and original_url are required for webcapture entities".to_string(),
                )
                .into());
            }
        }

        let rev_ids: Vec<Uuid> = insert_into(webcapture_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| WebcaptureRevNewRow {
                        // these unwraps safe because of check above
                        original_url: model.original_url.clone().unwrap(),
                        timestamp: model.timestamp.unwrap().naive_utc(),
                        extra_json: model.extra.clone(),
                    })
                    .collect::<Vec<WebcaptureRevNewRow>>(),
            )
            .returning(webcapture_rev::id)
            .get_results(conn)?;

        let mut webcapture_cdx_rows: Vec<WebcaptureRevCdxNewRow> = vec![];
        let mut webcapture_url_rows: Vec<WebcaptureRevUrlNewRow> = vec![];
        let mut webcapture_release_rows: Vec<WebcaptureRevReleaseRow> = vec![];

        for (model, rev_id) in models.iter().zip(rev_ids.iter()) {
            match &model.cdx {
                None => (),
                Some(cdx_list) => {
                    let these_cdx_rows: Vec<WebcaptureRevCdxNewRow> = cdx_list
                        .into_iter()
                        .map(|c| WebcaptureRevCdxNewRow {
                            webcapture_rev: *rev_id,
                            surt: c.surt.clone(),
                            timestamp: c.timestamp.clone(),
                            url: c.url.clone(),
                            mimetype: c.mimetype.clone(),
                            status_code: c.status_code,
                            sha1: c.sha1.clone(),
                            sha256: c.sha256.clone(),
                        })
                        .collect();
                    webcapture_cdx_rows.extend(these_cdx_rows);
                }
            };

            match &model.archive_urls {
                None => (),
                Some(url_list) => {
                    let these_url_rows: Vec<WebcaptureRevUrlNewRow> = url_list
                        .into_iter()
                        .map(|u| WebcaptureRevUrlNewRow {
                            webcapture_rev: *rev_id,
                            rel: u.rel.clone(),
                            url: u.url.clone(),
                        })
                        .collect();
                    webcapture_url_rows.extend(these_url_rows);
                }
            };

            match &model.release_ids {
                None => (),
                Some(release_list) => {
                    let these_release_rows: Result<Vec<WebcaptureRevReleaseRow>> = release_list
                        .iter()
                        .map(|r| {
                            Ok(WebcaptureRevReleaseRow {
                                webcapture_rev: *rev_id,
                                target_release_ident_id: FatCatId::from_str(r)?.to_uuid(),
                            })
                        })
                        .collect();
                    webcapture_release_rows.extend(these_release_rows?);
                }
            };
        }

        if !webcapture_cdx_rows.is_empty() {
            insert_into(webcapture_rev_cdx::table)
                .values(webcapture_cdx_rows)
                .execute(conn)?;
        }

        if !webcapture_url_rows.is_empty() {
            insert_into(webcapture_rev_url::table)
                .values(webcapture_url_rows)
                .execute(conn)?;
        }

        if !webcapture_release_rows.is_empty() {
            insert_into(webcapture_rev_release::table)
                .values(webcapture_release_rows)
                .execute(conn)?;
        }

        Ok(rev_ids)
    }
}

impl EntityCrud for ReleaseEntity {
    type EditRow = ReleaseEditRow;
    type EditNewRow = ReleaseEditNewRow;
    type IdentRow = ReleaseIdentRow;
    type IdentNewRow = ReleaseIdentNewRow;
    type RevRow = ReleaseRevRow;

    generic_db_get!(release_ident, release_rev);
    generic_db_get_rev!(release_rev);
    generic_db_update!(release_ident, release_edit);
    generic_db_delete!(release_ident, release_edit);
    generic_db_get_history!(release_edit);
    generic_db_get_edit!(release_edit);
    generic_db_delete_edit!(release_edit);
    generic_db_get_redirects!(release_ident);
    generic_db_accept_edits_batch!("release", release_ident, release_edit);
    generic_db_insert_rev!();

    fn from_deleted_row(ident_row: Self::IdentRow) -> Result<Self> {
        if ident_row.rev_id.is_some() {
            bail!("called from_deleted_row with a non-deleted-state row")
        }

        Ok(ReleaseEntity {
            title: None,
            release_type: None,
            release_status: None,
            release_date: None,
            release_year: None,
            doi: None,
            pmid: None,
            pmcid: None,
            isbn13: None,
            core_id: None,
            wikidata_qid: None,
            volume: None,
            issue: None,
            pages: None,
            files: None,
            filesets: None,
            webcaptures: None,
            container: None,
            container_id: None,
            publisher: None,
            language: None,
            work_id: None,
            refs: None,
            contribs: None,
            abstracts: None,

            state: Some(ident_row.state().unwrap().shortname()),
            ident: Some(FatCatId::from_uuid(&ident_row.id).to_string()),
            revision: ident_row.rev_id.map(|u| u.to_string()),
            redirect: ident_row
                .redirect_id
                .map(|u| FatCatId::from_uuid(&u).to_string()),
            extra: None,
            edit_extra: None,
        })
    }

    fn db_expand(&mut self, conn: &DbConn, expand: ExpandFlags) -> Result<()> {
        // Don't expand deleted entities
        if self.state == Some("deleted".to_string()) {
            return Ok(());
        }
        // TODO: should clarify behavior here. Would hit this path, eg, expanding files on a
        // release revision (not ident). Should we fail (Bad Request), or silently just not include
        // any files?
        if expand.files && self.ident.is_some() {
            let ident = match &self.ident {
                None => bail!("Can't expand files on a non-concrete entity"), // redundant with above is_some()
                Some(ident) => match &self.redirect {
                    // If we're a redirect, then expand for the *target* identifier, not *our*
                    // identifier. Tricky!
                    None => FatCatId::from_str(&ident)?,
                    Some(redir) => FatCatId::from_str(&redir)?,
                },
            };
            self.files = Some(get_release_files(ident, HideFlags::none(), conn)?);
        }
        if expand.container {
            if let Some(ref cid) = self.container_id {
                self.container = Some(ContainerEntity::db_get(
                    conn,
                    FatCatId::from_str(&cid)?,
                    HideFlags::none(),
                )?);
            }
        }
        if expand.creators {
            if let Some(ref mut contribs) = self.contribs {
                for contrib in contribs {
                    if let Some(ref creator_id) = contrib.creator_id {
                        contrib.creator = Some(CreatorEntity::db_get(
                            conn,
                            FatCatId::from_str(creator_id)?,
                            HideFlags::none(),
                        )?);
                    }
                }
            }
        }
        Ok(())
    }

    fn db_create(&self, conn: &DbConn, edit_context: &EditContext) -> Result<Self::EditRow> {
        if self.redirect.is_some() {
            return Err(ErrorKind::OtherBadRequest(
                "can't create an entity that redirects from the start".to_string(),
            )
            .into());
        }
        let mut edits = Self::db_create_batch(conn, edit_context, &[self])?;
        // probably a more elegant way to destroy the vec and take first element
        Ok(edits.pop().unwrap())
    }

    fn db_create_batch(
        conn: &DbConn,
        edit_context: &EditContext,
        models: &[&Self],
    ) -> Result<Vec<Self::EditRow>> {
        // This isn't the generic implementation because we need to create Work entities for each
        // of the release entities passed (at least in the common case)
        if models.iter().any(|m| m.redirect.is_some()) {
            return Err(ErrorKind::OtherBadRequest(
                "can't create an entity that redirects from the start".to_string(),
            )
            .into());
        }

        // Generate the set of new work entities to insert (usually one for each release, but some
        // releases might be pointed to a work already)
        let mut new_work_models: Vec<&WorkEntity> = vec![];
        for entity in models {
            if entity.work_id.is_none() {
                new_work_models.push(&WorkEntity {
                    ident: None,
                    revision: None,
                    redirect: None,
                    state: None,
                    extra: None,
                    edit_extra: None,
                });
            };
        }

        // create the works, then pluck the list of idents from the result
        let new_work_edits =
            WorkEntity::db_create_batch(conn, edit_context, new_work_models.as_slice())?;
        let mut new_work_ids: Vec<Uuid> = new_work_edits.iter().map(|edit| edit.ident_id).collect();

        // Copy all the release models, and ensure that each has work_id set, using the new work
        // idents. There should be one new work ident for each release missing one.
        let models_with_work_ids: Vec<Self> = models
            .iter()
            .map(|model| {
                let mut model = (*model).clone();
                if model.work_id.is_none() {
                    model.work_id =
                        Some(FatCatId::from_uuid(&new_work_ids.pop().unwrap()).to_string())
                }
                model
            })
            .collect();
        let model_refs: Vec<&Self> = models_with_work_ids.iter().map(|s| s).collect();
        let models = model_refs.as_slice();

        // The rest here is copy/pasta from the generic (how to avoid copypasta?)
        let rev_ids: Vec<Uuid> = Self::db_insert_revs(conn, models)?;
        let ident_ids: Vec<Uuid> = insert_into(release_ident::table)
            .values(
                rev_ids
                    .iter()
                    .map(|rev_id| Self::IdentNewRow {
                        rev_id: Some(*rev_id),
                        is_live: edit_context.autoaccept,
                        redirect_id: None,
                    })
                    .collect::<Vec<Self::IdentNewRow>>(),
            )
            .returning(release_ident::id)
            .get_results(conn)?;
        let edits: Vec<Self::EditRow> = insert_into(release_edit::table)
            .values(
                rev_ids
                    .into_iter()
                    .zip(ident_ids.into_iter())
                    .map(|(rev_id, ident_id)| Self::EditNewRow {
                        editgroup_id: edit_context.editgroup_id.to_uuid(),
                        rev_id: Some(rev_id),
                        ident_id,
                        redirect_id: None,
                        prev_rev: None,
                        extra_json: edit_context.extra_json.clone(),
                    })
                    .collect::<Vec<Self::EditNewRow>>(),
            )
            .get_results(conn)?;
        Ok(edits)
    }

    fn db_from_row(
        conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
        hide: HideFlags,
    ) -> Result<Self> {
        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };

        let refs: Option<Vec<ReleaseRef>> = match hide.refs {
            true => None,
            false => Some(
                release_ref::table
                    .filter(release_ref::release_rev.eq(rev_row.id))
                    .order(release_ref::index_val.asc())
                    .get_results(conn)?
                    .into_iter()
                    .map(|r: ReleaseRefRow| ReleaseRef {
                        index: r.index_val.map(|v| v as i64),
                        key: r.key,
                        extra: r.extra_json,
                        container_name: r.container_name,
                        year: r.year.map(|v| v as i64),
                        title: r.title,
                        locator: r.locator,
                        target_release_id: r
                            .target_release_ident_id
                            .map(|v| FatCatId::from_uuid(&v).to_string()),
                    })
                    .collect(),
            ),
        };

        let contribs: Option<Vec<ReleaseContrib>> = match hide.contribs {
            true => None,
            false => Some(
                release_contrib::table
                    .filter(release_contrib::release_rev.eq(rev_row.id))
                    .order((
                        release_contrib::role.asc(),
                        release_contrib::index_val.asc(),
                    ))
                    .get_results(conn)?
                    .into_iter()
                    .map(|c: ReleaseContribRow| ReleaseContrib {
                        index: c.index_val.map(|v| v as i64),
                        raw_name: c.raw_name,
                        role: c.role,
                        extra: c.extra_json,
                        creator_id: c
                            .creator_ident_id
                            .map(|v| FatCatId::from_uuid(&v).to_string()),
                        creator: None,
                    })
                    .collect(),
            ),
        };

        let abstracts: Option<Vec<ReleaseEntityAbstracts>> = if hide.abstracts {
            None
        } else {
            Some(
                release_rev_abstract::table
                    .inner_join(abstracts::table)
                    .filter(release_rev_abstract::release_rev.eq(rev_row.id))
                    .get_results(conn)?
                    .into_iter()
                    .map(
                        |r: (ReleaseRevAbstractRow, AbstractsRow)| ReleaseEntityAbstracts {
                            sha1: Some(r.0.abstract_sha1),
                            mimetype: r.0.mimetype,
                            lang: r.0.lang,
                            content: Some(r.1.content),
                        },
                    )
                    .collect(),
            )
        };

        Ok(ReleaseEntity {
            title: Some(rev_row.title),
            release_type: rev_row.release_type,
            release_status: rev_row.release_status,
            release_date: rev_row.release_date,
            release_year: rev_row.release_year,
            doi: rev_row.doi,
            pmid: rev_row.pmid,
            pmcid: rev_row.pmcid,
            isbn13: rev_row.isbn13,
            core_id: rev_row.core_id,
            wikidata_qid: rev_row.wikidata_qid,
            volume: rev_row.volume,
            issue: rev_row.issue,
            pages: rev_row.pages,
            files: None,
            filesets: None,
            webcaptures: None,
            container: None,
            container_id: rev_row
                .container_ident_id
                .map(|u| FatCatId::from_uuid(&u).to_string()),
            publisher: rev_row.publisher,
            language: rev_row.language,
            work_id: Some(FatCatId::from_uuid(&rev_row.work_ident_id).to_string()),
            refs,
            contribs,
            abstracts,
            state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            extra: rev_row.extra_json,
            edit_extra: None,
        })
    }

    fn db_insert_revs(conn: &DbConn, models: &[&Self]) -> Result<Vec<Uuid>> {
        // first verify external identifier syntax
        for entity in models {
            if let Some(ref extid) = entity.doi {
                check_doi(extid)?;
            }
            if let Some(ref extid) = entity.pmid {
                check_pmid(extid)?;
            }
            if let Some(ref extid) = entity.pmcid {
                check_pmcid(extid)?;
            }
            if let Some(ref extid) = entity.wikidata_qid {
                check_wikidata_qid(extid)?;
            }
            if let Some(ref release_type) = entity.release_type {
                check_release_type(release_type)?;
            }
            if let Some(ref contribs) = entity.contribs {
                for contrib in contribs {
                    if let Some(ref role) = contrib.role {
                        check_contrib_role(role)?;
                    }
                }
            }
        }

        if models.iter().any(|m| m.title.is_none()) {
            return Err(ErrorKind::OtherBadRequest(
                "title is required for all Release entities".to_string(),
            )
            .into());
        }

        let rev_ids: Vec<Uuid> = insert_into(release_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| {
                        Ok(ReleaseRevNewRow {
                    title: model.title.clone().unwrap(), // titles checked above
                    release_type: model.release_type.clone(),
                    release_status: model.release_status.clone(),
                    release_date: model.release_date,
                    release_year: model.release_year,
                    doi: model.doi.clone(),
                    pmid: model.pmid.clone(),
                    pmcid: model.pmcid.clone(),
                    wikidata_qid: model.wikidata_qid.clone(),
                    isbn13: model.isbn13.clone(),
                    core_id: model.core_id.clone(),
                    volume: model.volume.clone(),
                    issue: model.issue.clone(),
                    pages: model.pages.clone(),
                    work_ident_id: match model.work_id.clone() {
                        None => bail!("release_revs must have a work_id by the time they are inserted; this is an internal soundness error"),
                        Some(s) => FatCatId::from_str(&s)?.to_uuid(),
                    },
                    container_ident_id: match model.container_id.clone() {
                        None => None,
                        Some(s) => Some(FatCatId::from_str(&s)?.to_uuid()),
                    },
                    publisher: model.publisher.clone(),
                    language: model.language.clone(),
                    extra_json: model.extra.clone()
                })
                    })
                    .collect::<Result<Vec<ReleaseRevNewRow>>>()?,
            )
            .returning(release_rev::id)
            .get_results(conn)?;

        let mut release_ref_rows: Vec<ReleaseRefNewRow> = vec![];
        let mut release_contrib_rows: Vec<ReleaseContribNewRow> = vec![];
        let mut abstract_rows: Vec<AbstractsRow> = vec![];
        let mut release_abstract_rows: Vec<ReleaseRevAbstractNewRow> = vec![];

        for (model, rev_id) in models.iter().zip(rev_ids.iter()) {
            match &model.refs {
                None => (),
                Some(ref_list) => {
                    let these_ref_rows: Vec<ReleaseRefNewRow> = ref_list
                        .iter()
                        .map(|r| {
                            Ok(ReleaseRefNewRow {
                                release_rev: *rev_id,
                                target_release_ident_id: match r.target_release_id.clone() {
                                    None => None,
                                    Some(v) => Some(FatCatId::from_str(&v)?.to_uuid()),
                                },
                                index_val: r.index.map(|v| v as i32),
                                key: r.key.clone(),
                                container_name: r.container_name.clone(),
                                year: r.year.map(|v| v as i32),
                                title: r.title.clone(),
                                locator: r.locator.clone(),
                                extra_json: r.extra.clone(),
                            })
                        })
                        .collect::<Result<Vec<ReleaseRefNewRow>>>()?;
                    release_ref_rows.extend(these_ref_rows);
                }
            };

            match &model.contribs {
                None => (),
                Some(contrib_list) => {
                    let these_contrib_rows: Vec<ReleaseContribNewRow> = contrib_list
                        .iter()
                        .map(|c| {
                            Ok(ReleaseContribNewRow {
                                release_rev: *rev_id,
                                creator_ident_id: match c.creator_id.clone() {
                                    None => None,
                                    Some(v) => Some(FatCatId::from_str(&v)?.to_uuid()),
                                },
                                raw_name: c.raw_name.clone(),
                                index_val: c.index.map(|v| v as i32),
                                role: c.role.clone(),
                                extra_json: c.extra.clone(),
                            })
                        })
                        .collect::<Result<Vec<ReleaseContribNewRow>>>()?;
                    release_contrib_rows.extend(these_contrib_rows);
                }
            };

            if let Some(abstract_list) = &model.abstracts {
                // For rows that specify content, we need to insert the abstract if it doesn't exist
                // already
                let new_abstracts: Vec<AbstractsRow> = abstract_list
                    .iter()
                    .filter(|ea| ea.content.is_some())
                    .map(|c| AbstractsRow {
                        sha1: Sha1::from(c.content.clone().unwrap()).hexdigest(),
                        content: c.content.clone().unwrap(),
                    })
                    .collect();
                abstract_rows.extend(new_abstracts);
                let new_release_abstract_rows: Vec<ReleaseRevAbstractNewRow> = abstract_list
                    .into_iter()
                    .map(|c| {
                        Ok(ReleaseRevAbstractNewRow {
                            release_rev: *rev_id,
                            abstract_sha1: match c.content {
                                Some(ref content) => Sha1::from(content).hexdigest(),
                                None => match c.sha1.clone() {
                                    Some(v) => v,
                                    None => bail!("either abstract_sha1 or content is required"),
                                },
                            },
                            lang: c.lang.clone(),
                            mimetype: c.mimetype.clone(),
                        })
                    })
                    .collect::<Result<Vec<ReleaseRevAbstractNewRow>>>()?;
                release_abstract_rows.extend(new_release_abstract_rows);
            }
        }

        if !release_ref_rows.is_empty() {
            insert_into(release_ref::table)
                .values(release_ref_rows)
                .execute(conn)?;
        }

        if !release_contrib_rows.is_empty() {
            insert_into(release_contrib::table)
                .values(release_contrib_rows)
                .execute(conn)?;
        }

        if !abstract_rows.is_empty() {
            // Sort of an "upsert"; only inserts new abstract rows if they don't already exist
            insert_into(abstracts::table)
                .values(&abstract_rows)
                .on_conflict(abstracts::sha1)
                .do_nothing()
                .execute(conn)?;
            insert_into(release_rev_abstract::table)
                .values(release_abstract_rows)
                .execute(conn)?;
        }

        Ok(rev_ids)
    }
}

impl EntityCrud for WorkEntity {
    type EditRow = WorkEditRow;
    type EditNewRow = WorkEditNewRow;
    type IdentRow = WorkIdentRow;
    type IdentNewRow = WorkIdentNewRow;
    type RevRow = WorkRevRow;

    generic_db_get!(work_ident, work_rev);
    generic_db_get_rev!(work_rev);
    generic_db_expand!();
    generic_db_create!(work_ident, work_edit);
    generic_db_create_batch!(work_ident, work_edit);
    generic_db_update!(work_ident, work_edit);
    generic_db_delete!(work_ident, work_edit);
    generic_db_get_history!(work_edit);
    generic_db_get_edit!(work_edit);
    generic_db_delete_edit!(work_edit);
    generic_db_get_redirects!(work_ident);
    generic_db_accept_edits_batch!("work", work_ident, work_edit);
    generic_db_insert_rev!();

    fn from_deleted_row(ident_row: Self::IdentRow) -> Result<Self> {
        if ident_row.rev_id.is_some() {
            bail!("called from_deleted_row with a non-deleted-state row")
        }

        Ok(WorkEntity {
            state: Some(ident_row.state().unwrap().shortname()),
            ident: Some(FatCatId::from_uuid(&ident_row.id).to_string()),
            revision: ident_row.rev_id.map(|u| u.to_string()),
            redirect: ident_row
                .redirect_id
                .map(|u| FatCatId::from_uuid(&u).to_string()),
            extra: None,
            edit_extra: None,
        })
    }

    fn db_from_row(
        _conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
        _hide: HideFlags,
    ) -> Result<Self> {
        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };

        Ok(WorkEntity {
            state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            extra: rev_row.extra_json,
            edit_extra: None,
        })
    }

    fn db_insert_revs(conn: &DbConn, models: &[&Self]) -> Result<Vec<Uuid>> {
        let rev_ids: Vec<Uuid> = insert_into(work_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| WorkRevNewRow {
                        extra_json: model.extra.clone(),
                    })
                    .collect::<Vec<WorkRevNewRow>>(),
            )
            .returning(work_rev::id)
            .get_results(conn)?;
        Ok(rev_ids)
    }
}