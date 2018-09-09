use api_helpers::*;
use chrono;
use database_models::*;
use database_schema::*;
use diesel::prelude::*;
use diesel::{self, insert_into};
use errors::*;
use fatcat_api::models::*;
use serde_json;
use sha1::Sha1;
use std::marker::Sized;
use std::str::FromStr;
use uuid::Uuid;

pub struct EditContext {
    pub editor_id: FatCatId,
    pub editgroup_id: FatCatId,
    pub extra_json: Option<serde_json::Value>,
    pub autoaccept: bool,
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

    fn parse_editgroup_id(&self) -> Result<Option<FatCatId>>;

    // Generic Methods
    fn db_get(
        conn: &DbConn,
        ident: FatCatId
    ) -> Result<Self>;
    fn db_get_rev(
        conn: &DbConn,
        rev_id: Uuid
    ) -> Result<Self>;
    fn db_create(
        &self,
        conn: &DbConn,
        edit_context: &EditContext
    ) -> Result<Self::EditRow>;
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
    fn db_accept_edits(
        conn: &DbConn,
        editgroup_id: FatCatId
    ) -> Result<u64>;

    // Entity-specific Methods
    fn db_from_row(
        conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
    ) -> Result<Self>;
    fn db_insert_rev(
        &self,
        conn: &DbConn
    ) -> Result<Uuid>;
    fn db_insert_revs(
        conn: &DbConn,
        models: &[&Self]
    ) -> Result<Vec<Uuid>>;
}

// TODO: this could be a separate trait on all entities
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
    ($ident_table:ident, $rev_table:ident) => {
        fn db_get(conn: &DbConn, ident: FatCatId) -> Result<Self> {
            let (ident, rev): (Self::IdentRow, Self::RevRow) = $ident_table::table
                .find(ident.to_uuid())
                .inner_join($rev_table::table)
                .first(conn)?;

            Self::db_from_row(conn, rev, Some(ident))
        }
    };
}

macro_rules! generic_db_get_rev {
    ($rev_table:ident) => {
        fn db_get_rev(conn: &DbConn, rev_id: Uuid) -> Result<Self> {
            let rev = $rev_table::table.find(rev_id).first(conn)?;

            Self::db_from_row(conn, rev, None)
        }
    };
}

macro_rules! generic_db_create {
    // TODO: this path should call generic_db_create_batch
    ($ident_table: ident, $edit_table: ident) => {
        fn db_create(&self, conn: &DbConn, edit_context: &EditContext) -> Result<Self::EditRow> {
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
                            ident_id: ident_id,
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
    ($ident_table:ident, $edit_table:ident) => {
        fn db_delete(
            conn: &DbConn,
            edit_context: &EditContext,
            ident: FatCatId,
        ) -> Result<Self::EditRow> {
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

            let history: Result<Vec<EntityHistoryEntry>> = rows.into_iter()
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
macro_rules! generic_db_accept_edits_batch {
    ($entity_name_str:expr) => {
        fn db_accept_edits(
            conn: &DbConn,
            editgroup_id: FatCatId,
        ) -> Result<u64> {

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
            )).bind::<diesel::sql_types::Uuid, _>(editgroup_id.to_uuid())
                .execute(conn)?;
            Ok(count as u64)
        }
    }
}

// UPDATE ROW version: single query per row
// CockroachDB version (slow, single query per row)
macro_rules! generic_db_accept_edits_each {
    ($ident_table:ident, $edit_table:ident) => {
        fn db_accept_edits(
            conn: &DbConn,
            editgroup_id: FatCatId,
        ) -> Result<u64> {

            // 1. select edit rows (in sql)
            let edit_rows: Vec<Self::EditRow> = $edit_table::table
                .filter($edit_table::editgroup_id.eq(&editgroup_id.to_uuid()))
                .get_results(conn)?;
            // 2. create ident rows (in rust)
            let ident_rows: Vec<Self::IdentRow> = edit_rows
                .iter()
                .map(|edit|
                    Self::IdentRow {
                        id: edit.ident_id,
                        is_live: true,
                        rev_id: edit.rev_id,
                        redirect_id: edit.redirect_id,

                    }
                )
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
                diesel::update(&row)
                    .set(&row)
                    .execute(conn)?;
            }
            Ok(count)
        }
    };
}

macro_rules! generic_db_insert_rev {
    () => {
        fn db_insert_rev(&self, conn: &DbConn) -> Result<Uuid> {
            Self::db_insert_revs(conn, &vec![self]).map(|id_list| id_list[0])
        }
    }
}

impl EntityCrud for ContainerEntity {
    type EditRow = ContainerEditRow;
    type EditNewRow = ContainerEditNewRow;
    type IdentRow = ContainerIdentRow;
    type IdentNewRow = ContainerIdentNewRow;
    type RevRow = ContainerRevRow;

    generic_parse_editgroup_id!();
    generic_db_get!(container_ident, container_rev);
    generic_db_get_rev!(container_rev);
    generic_db_create!(container_ident, container_edit);
    generic_db_create_batch!(container_ident, container_edit);
    generic_db_update!(container_ident, container_edit);
    generic_db_delete!(container_ident, container_edit);
    generic_db_get_history!(container_edit);
    //generic_db_accept_edits_batch!("container");
    generic_db_accept_edits_each!(container_ident, container_edit);
    generic_db_insert_rev!();

    fn db_from_row(
        _conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
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
            name: rev_row.name,
            abbrev: rev_row.abbrev,
            coden: rev_row.coden,
            state: state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            extra: rev_row.extra_json,
            editgroup_id: None,
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

        let rev_ids: Vec<Uuid> = insert_into(container_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| ContainerRevNewRow {
                        name: model.name.clone(),
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

    generic_parse_editgroup_id!();
    generic_db_get!(creator_ident, creator_rev);
    generic_db_get_rev!(creator_rev);
    generic_db_create!(creator_ident, creator_edit);
    generic_db_create_batch!(creator_ident, creator_edit);
    generic_db_update!(creator_ident, creator_edit);
    generic_db_delete!(creator_ident, creator_edit);
    generic_db_get_history!(creator_edit);
    //generic_db_accept_edits_batch!("creator");
    generic_db_accept_edits_each!(creator_ident, creator_edit);
    generic_db_insert_rev!();

    fn db_from_row(
        _conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
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
            display_name: rev_row.display_name,
            given_name: rev_row.given_name,
            surname: rev_row.surname,
            orcid: rev_row.orcid,
            wikidata_qid: rev_row.wikidata_qid,
            state: state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            editgroup_id: None,
            extra: rev_row.extra_json,
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

        let rev_ids: Vec<Uuid> = insert_into(creator_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| CreatorRevNewRow {
                        display_name: model.display_name.clone(),
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

    generic_parse_editgroup_id!();
    generic_db_get!(file_ident, file_rev);
    generic_db_get_rev!(file_rev);
    generic_db_create!(file_ident, file_edit);
    generic_db_create_batch!(file_ident, file_edit);
    generic_db_update!(file_ident, file_edit);
    generic_db_delete!(file_ident, file_edit);
    generic_db_get_history!(file_edit);
    //generic_db_accept_edits_batch!("file");
    generic_db_accept_edits_each!(file_ident, file_edit);
    generic_db_insert_rev!();

    fn db_from_row(
        conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
    ) -> Result<Self> {
        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };

        let releases: Vec<FatCatId> = file_release::table
            .filter(file_release::file_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|r: FileReleaseRow| FatCatId::from_uuid(&r.target_release_ident_id))
            .collect();

        let urls: Vec<FileEntityUrls> = file_rev_url::table
            .filter(file_rev_url::file_rev.eq(rev_row.id))
            .get_results(conn)?
            .into_iter()
            .map(|r: FileRevUrlRow| FileEntityUrls {
                rel: r.rel,
                url: r.url,
            })
            .collect();

        Ok(FileEntity {
            sha1: rev_row.sha1,
            sha256: rev_row.sha256,
            md5: rev_row.md5,
            size: rev_row.size.map(|v| v as i64),
            urls: Some(urls),
            mimetype: rev_row.mimetype,
            releases: Some(releases.iter().map(|fcid| fcid.to_string()).collect()),
            state: state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            editgroup_id: None,
            extra: rev_row.extra_json,
        })
    }

    fn db_insert_revs(conn: &DbConn, models: &[&Self]) -> Result<Vec<Uuid>> {
        let rev_ids: Vec<Uuid> = insert_into(file_rev::table)
            .values(
                models
                    .iter()
                    .map(|model| FileRevNewRow {
                        size: model.size,
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

        let mut file_release_rows: Vec<FileReleaseRow> = vec![];
        let mut file_url_rows: Vec<FileRevUrlNewRow> = vec![];

        for (model, rev_id) in models.iter().zip(rev_ids.iter()) {
            match &model.releases {
                None => (),
                Some(release_list) => {
                    let these_release_rows: Result<Vec<FileReleaseRow>> = release_list
                        .iter()
                        .map(|r| {
                            Ok(FileReleaseRow {
                                file_rev: rev_id.clone(),
                                target_release_ident_id: FatCatId::from_str(r)?.to_uuid(),
                            })
                        })
                        .collect();
                    file_release_rows.extend(these_release_rows?);
                }
            };

            match &model.urls {
                None => (),
                Some(url_list) => {
                    let these_url_rows: Vec<FileRevUrlNewRow> = url_list
                        .into_iter()
                        .map(|u| FileRevUrlNewRow {
                            file_rev: rev_id.clone(),
                            rel: u.rel.clone(),
                            url: u.url.clone(),
                        })
                        .collect();
                    file_url_rows.extend(these_url_rows);
                }
            };
        }

        if !file_release_rows.is_empty() {
            // TODO: shouldn't it be "file_rev_release"?
            insert_into(file_release::table)
                .values(file_release_rows)
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

impl EntityCrud for ReleaseEntity {
    type EditRow = ReleaseEditRow;
    type EditNewRow = ReleaseEditNewRow;
    type IdentRow = ReleaseIdentRow;
    type IdentNewRow = ReleaseIdentNewRow;
    type RevRow = ReleaseRevRow;

    generic_parse_editgroup_id!();
    generic_db_get!(release_ident, release_rev);
    generic_db_get_rev!(release_rev);
    //generic_db_create!(release_ident, release_edit);
    //generic_db_create_batch!(release_ident, release_edit);
    generic_db_update!(release_ident, release_edit);
    generic_db_delete!(release_ident, release_edit);
    generic_db_get_history!(release_edit);
    //generic_db_accept_edits_batch!("release");
    generic_db_accept_edits_each!(release_ident, release_edit);
    generic_db_insert_rev!();

    fn db_create(&self, conn: &DbConn, edit_context: &EditContext) -> Result<Self::EditRow> {
        let mut edits = Self::db_create_batch(conn, edit_context, &vec![self])?;
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
                    editgroup_id: None,
                    extra: None,
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
                        rev_id: Some(rev_id.clone()),
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
                        ident_id: ident_id,
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
    ) -> Result<Self> {
        let (state, ident_id, redirect_id) = match ident_row {
            Some(i) => (
                Some(i.state().unwrap().shortname()),
                Some(FatCatId::from_uuid(&i.id).to_string()),
                i.redirect_id.map(|u| FatCatId::from_uuid(&u).to_string()),
            ),
            None => (None, None, None),
        };

        let refs: Vec<ReleaseRef> = release_ref::table
            .filter(release_ref::release_rev.eq(rev_row.id))
            .order(release_ref::index_val.asc())
            .get_results(conn)?
            .into_iter()
            .map(|r: ReleaseRefRow| ReleaseRef {
                index: r.index_val,
                key: r.key,
                extra: r.extra_json,
                container_title: r.container_title,
                year: r.year,
                title: r.title,
                locator: r.locator,
                target_release_id: r.target_release_ident_id
                    .map(|v| FatCatId::from_uuid(&v).to_string()),
            })
            .collect();

        let contribs: Vec<ReleaseContrib> = release_contrib::table
            .filter(release_contrib::release_rev.eq(rev_row.id))
            .order((
                release_contrib::role.asc(),
                release_contrib::index_val.asc(),
            ))
            .get_results(conn)?
            .into_iter()
            .map(|c: ReleaseContribRow| ReleaseContrib {
                index: c.index_val,
                raw_name: c.raw_name,
                role: c.role,
                extra: c.extra_json,
                creator_id: c.creator_ident_id
                    .map(|v| FatCatId::from_uuid(&v).to_string()),
                creator: None,
            })
            .collect();

        let abstracts: Vec<ReleaseEntityAbstracts> = release_rev_abstract::table
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
            .collect();

        Ok(ReleaseEntity {
            title: rev_row.title,
            release_type: rev_row.release_type,
            release_status: rev_row.release_status,
            release_date: rev_row
                .release_date
                .map(|v| chrono::DateTime::from_utc(v.and_hms(0, 0, 0), chrono::Utc)),
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
            container: None,
            container_id: rev_row
                .container_ident_id
                .map(|u| FatCatId::from_uuid(&u).to_string()),
            publisher: rev_row.publisher,
            language: rev_row.language,
            work_id: Some(FatCatId::from_uuid(&rev_row.work_ident_id).to_string()),
            refs: Some(refs),
            contribs: Some(contribs),
            abstracts: Some(abstracts),
            state: state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            editgroup_id: None,
            extra: rev_row.extra_json,
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
        }

        let rev_ids: Vec<Uuid> = insert_into(release_rev::table)
            .values(models
                .iter()
                .map(|model| {
                    Ok(ReleaseRevNewRow {
                    title: model.title.clone(),
                    release_type: model.release_type.clone(),
                    release_status: model.release_status.clone(),
                    release_date: model.release_date.map(|v| v.naive_utc().date()),
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
                .collect::<Result<Vec<ReleaseRevNewRow>>>()?)
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
                                release_rev: rev_id.clone(),
                                target_release_ident_id: match r.target_release_id.clone() {
                                    None => None,
                                    Some(v) => Some(FatCatId::from_str(&v)?.to_uuid()),
                                },
                                index_val: r.index,
                                key: r.key.clone(),
                                container_title: r.container_title.clone(),
                                year: r.year,
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
                                release_rev: rev_id.clone(),
                                creator_ident_id: match c.creator_id.clone() {
                                    None => None,
                                    Some(v) => Some(FatCatId::from_str(&v)?.to_uuid()),
                                },
                                raw_name: c.raw_name.clone(),
                                index_val: c.index,
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
                            release_rev: rev_id.clone(),
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

    generic_parse_editgroup_id!();
    generic_db_get!(work_ident, work_rev);
    generic_db_get_rev!(work_rev);
    generic_db_create!(work_ident, work_edit);
    generic_db_create_batch!(work_ident, work_edit);
    generic_db_update!(work_ident, work_edit);
    generic_db_delete!(work_ident, work_edit);
    generic_db_get_history!(work_edit);
    //generic_db_accept_edits_batch!("work");
    generic_db_accept_edits_each!(work_ident, work_edit);
    generic_db_insert_rev!();

    fn db_from_row(
        _conn: &DbConn,
        rev_row: Self::RevRow,
        ident_row: Option<Self::IdentRow>,
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
            state: state,
            ident: ident_id,
            revision: Some(rev_row.id.to_string()),
            redirect: redirect_id,
            editgroup_id: None,
            extra: rev_row.extra_json,
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
