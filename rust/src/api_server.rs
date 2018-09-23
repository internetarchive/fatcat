//! API endpoint handlers

use api_entity_crud::EntityCrud;
use api_helpers::*;
use chrono;
use database_models::*;
use database_schema::*;
use diesel::prelude::*;
use diesel::{self, insert_into};
use errors::*;
use fatcat_api_spec::models;
use fatcat_api_spec::models::*;
use std::str::FromStr;
use ConnectionPool;

macro_rules! entity_batch_handler {
    ($post_batch_handler:ident, $model:ident) => {
        pub fn $post_batch_handler(
            &self,
            entity_list: &[models::$model],
            autoaccept: bool,
            editgroup_id: Option<FatCatId>,
            conn: &DbConn,
        ) -> Result<Vec<EntityEdit>> {

            let edit_context = make_edit_context(conn, editgroup_id, autoaccept)?;
            let model_list: Vec<&models::$model> = entity_list.iter().map(|e| e).collect();
            let edits = $model::db_create_batch(conn, &edit_context, model_list.as_slice())?;

            if autoaccept {
                let _clr: ChangelogRow = diesel::insert_into(changelog::table)
                    .values((changelog::editgroup_id.eq(edit_context.editgroup_id.to_uuid()),))
                    .get_result(conn)?;
            }
            edits.into_iter().map(|e| e.into_model()).collect()
        }
    }
}

macro_rules! count_entity {
    ($table:ident, $conn:expr) => {{
        let count: i64 = $table::table
            .filter($table::is_live.eq(true))
            .filter($table::redirect_id.is_null())
            .count()
            .first($conn)?;
        count
    }};
}

#[derive(Clone)]
pub struct Server {
    pub db_pool: ConnectionPool,
}

pub fn get_release_files(id: FatCatId, conn: &DbConn) -> Result<Vec<FileEntity>> {
    let rows: Vec<(FileRevRow, FileIdentRow, FileReleaseRow)> = file_rev::table
        .inner_join(file_ident::table)
        .inner_join(file_release::table)
        .filter(file_release::target_release_ident_id.eq(&id.to_uuid()))
        .filter(file_ident::is_live.eq(true))
        .filter(file_ident::redirect_id.is_null())
        .load(conn)?;

    rows.into_iter()
        .map(|(rev, ident, _)| FileEntity::db_from_row(conn, rev, Some(ident)))
        .collect()
}

impl Server {
    pub fn lookup_container_handler(&self, issnl: &str, conn: &DbConn) -> Result<ContainerEntity> {
        check_issn(issnl)?;
        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = container_ident::table
            .inner_join(container_rev::table)
            .filter(container_rev::issnl.eq(issnl))
            .filter(container_ident::is_live.eq(true))
            .filter(container_ident::redirect_id.is_null())
            .first(conn)?;

        ContainerEntity::db_from_row(conn, rev, Some(ident))
    }

    pub fn lookup_creator_handler(&self, orcid: &str, conn: &DbConn) -> Result<CreatorEntity> {
        check_orcid(orcid)?;
        let (ident, rev): (CreatorIdentRow, CreatorRevRow) = creator_ident::table
            .inner_join(creator_rev::table)
            .filter(creator_rev::orcid.eq(orcid))
            .filter(creator_ident::is_live.eq(true))
            .filter(creator_ident::redirect_id.is_null())
            .first(conn)?;

        CreatorEntity::db_from_row(conn, rev, Some(ident))
    }

    pub fn get_creator_releases_handler(
        &self,
        id: FatCatId,
        conn: &DbConn,
    ) -> Result<Vec<ReleaseEntity>> {
        // TODO: some kind of unique or group-by?
        let rows: Vec<(ReleaseRevRow, ReleaseIdentRow, ReleaseContribRow)> = release_rev::table
            .inner_join(release_ident::table)
            .inner_join(release_contrib::table)
            .filter(release_contrib::creator_ident_id.eq(&id.to_uuid()))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .load(conn)?;

        // TODO: from_rows, not from_row?
        rows.into_iter()
            .map(|(rev, ident, _)| ReleaseEntity::db_from_row(conn, rev, Some(ident)))
            .collect()
    }

    pub fn lookup_file_handler(&self, sha1: &str, conn: &DbConn) -> Result<FileEntity> {
        let (ident, rev): (FileIdentRow, FileRevRow) = file_ident::table
            .inner_join(file_rev::table)
            .filter(file_rev::sha1.eq(sha1))
            .filter(file_ident::is_live.eq(true))
            .filter(file_ident::redirect_id.is_null())
            .first(conn)?;

        FileEntity::db_from_row(conn, rev, Some(ident))
    }

    pub fn lookup_release_handler(&self, doi: &str, conn: &DbConn) -> Result<ReleaseEntity> {
        check_doi(doi)?;
        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) = release_ident::table
            .inner_join(release_rev::table)
            .filter(release_rev::doi.eq(doi))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .first(conn)?;

        ReleaseEntity::db_from_row(conn, rev, Some(ident))
    }

    pub fn get_release_files_handler(
        &self,
        id: FatCatId,
        conn: &DbConn,
    ) -> Result<Vec<FileEntity>> {
        get_release_files(id, conn)
    }

    pub fn get_work_releases_handler(
        &self,
        id: FatCatId,
        conn: &DbConn,
    ) -> Result<Vec<ReleaseEntity>> {
        let rows: Vec<(ReleaseRevRow, ReleaseIdentRow)> = release_rev::table
            .inner_join(release_ident::table)
            .filter(release_rev::work_ident_id.eq(&id.to_uuid()))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .load(conn)?;

        rows.into_iter()
            .map(|(rev, ident)| ReleaseEntity::db_from_row(conn, rev, Some(ident)))
            .collect()
    }

    pub fn accept_editgroup_handler(&self, id: FatCatId, conn: &DbConn) -> Result<()> {
        accept_editgroup(id, conn)?;
        Ok(())
    }

    pub fn create_editgroup_handler(
        &self,
        entity: models::Editgroup,
        conn: &DbConn,
    ) -> Result<Editgroup> {
        let row: EditgroupRow = insert_into(editgroup::table)
            .values((
                editgroup::editor_id.eq(FatCatId::from_str(&entity.editor_id)?.to_uuid()),
                editgroup::description.eq(entity.description),
                editgroup::extra_json.eq(entity.extra),
            )).get_result(conn)?;

        Ok(Editgroup {
            id: Some(uuid2fcid(&row.id)),
            editor_id: uuid2fcid(&row.editor_id),
            description: row.description,
            edits: None,
            extra: row.extra_json,
        })
    }

    pub fn get_editgroup_handler(&self, id: FatCatId, conn: &DbConn) -> Result<Editgroup> {
        let row: EditgroupRow = editgroup::table.find(id.to_uuid()).first(conn)?;

        let edits = EditgroupEdits {
            containers: Some(
                container_edit::table
                    .filter(container_edit::editgroup_id.eq(id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: ContainerEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            creators: Some(
                creator_edit::table
                    .filter(creator_edit::editgroup_id.eq(id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: CreatorEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            files: Some(
                file_edit::table
                    .filter(file_edit::editgroup_id.eq(id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: FileEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            releases: Some(
                release_edit::table
                    .filter(release_edit::editgroup_id.eq(id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: ReleaseEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            works: Some(
                work_edit::table
                    .filter(work_edit::editgroup_id.eq(id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: WorkEditRow| e.into_model().unwrap())
                    .collect(),
            ),
        };

        let eg = Editgroup {
            id: Some(uuid2fcid(&row.id)),
            editor_id: uuid2fcid(&row.editor_id),
            description: row.description,
            edits: Some(edits),
            extra: row.extra_json,
        };
        Ok(eg)
    }

    pub fn get_editor_handler(&self, id: FatCatId, conn: &DbConn) -> Result<Editor> {
        let row: EditorRow = editor::table.find(id.to_uuid()).first(conn)?;

        let ed = Editor {
            id: Some(uuid2fcid(&row.id)),
            username: row.username,
        };
        Ok(ed)
    }

    pub fn get_editor_changelog_handler(
        &self,
        id: FatCatId,
        conn: &DbConn,
    ) -> Result<Vec<ChangelogEntry>> {
        // TODO: single query
        let editor: EditorRow = editor::table.find(id.to_uuid()).first(conn)?;
        let changes: Vec<(ChangelogRow, EditgroupRow)> = changelog::table
            .inner_join(editgroup::table)
            .filter(editgroup::editor_id.eq(editor.id))
            .load(conn)?;

        let entries = changes
            .into_iter()
            .map(|(cl_row, eg_row)| ChangelogEntry {
                index: cl_row.id,
                editgroup: Some(eg_row.into_model_partial()),
                editgroup_id: uuid2fcid(&cl_row.editgroup_id),
                timestamp: chrono::DateTime::from_utc(cl_row.timestamp, chrono::Utc),
            }).collect();
        Ok(entries)
    }

    pub fn get_changelog_handler(
        &self,
        limit: Option<i64>,
        conn: &DbConn,
    ) -> Result<Vec<ChangelogEntry>> {
        let limit = limit.unwrap_or(50);

        let changes: Vec<(ChangelogRow, EditgroupRow)> = changelog::table
            .inner_join(editgroup::table)
            .order(changelog::id.desc())
            .limit(limit)
            .load(conn)?;

        let entries = changes
            .into_iter()
            .map(|(cl_row, eg_row)| ChangelogEntry {
                index: cl_row.id,
                editgroup: Some(eg_row.into_model_partial()),
                editgroup_id: uuid2fcid(&cl_row.editgroup_id),
                timestamp: chrono::DateTime::from_utc(cl_row.timestamp, chrono::Utc),
            }).collect();
        Ok(entries)
    }

    pub fn get_changelog_entry_handler(&self, id: i64, conn: &DbConn) -> Result<ChangelogEntry> {
        let cl_row: ChangelogRow = changelog::table.find(id).first(conn)?;
        let editgroup =
            self.get_editgroup_handler(FatCatId::from_uuid(&cl_row.editgroup_id), conn)?;

        let mut entry = cl_row.into_model();
        entry.editgroup = Some(editgroup);
        Ok(entry)
    }

    pub fn get_stats_handler(&self, more: &Option<String>, conn: &DbConn) -> Result<StatsResponse> {
        let merged_editgroups: i64 = changelog::table
            .select(diesel::dsl::count_star())
            .first(conn)?;
        let releases_with_dois: i64 = release_rev::table
            .inner_join(release_ident::table)
            .filter(release_rev::doi.is_not_null())
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .select(diesel::dsl::count_star())
            .first(conn)?;
        let creators_with_orcids: i64 = creator_rev::table
            .inner_join(creator_ident::table)
            .filter(creator_rev::orcid.is_not_null())
            .filter(creator_ident::is_live.eq(true))
            .filter(creator_ident::redirect_id.is_null())
            .select(diesel::dsl::count_star())
            .first(conn)?;
        let containers_with_issnls: i64 = container_rev::table
            .inner_join(container_ident::table)
            .filter(container_rev::issnl.is_not_null())
            .filter(container_ident::is_live.eq(true))
            .filter(container_ident::redirect_id.is_null())
            .count()
            .first(conn)?;

        let files_with_releases: Option<i64> = if more.is_some() {
            // this query is slightly inaccurate and over-counts: it includes files that have release
            // links only to inactive releases
            Some(
                file_rev::table
                    .inner_join(file_ident::table)
                    .inner_join(file_release::table)
                    .filter(file_ident::is_live.eq(true))
                    .filter(file_ident::redirect_id.is_null())
                    .select(file_ident::id)
                    .distinct()
                    .count()
                    .first(conn)?,
            )
        } else {
            None
        };
        let releases_with_files: Option<i64> = if more.is_some() {
            // this slightly overcounts also: it will include releases which are only linked to from
            // inactive files
            Some(
                release_ident::table
                    .inner_join(file_release::table)
                    .filter(release_ident::is_live.eq(true))
                    .filter(release_ident::redirect_id.is_null())
                    .select(file_release::target_release_ident_id)
                    .distinct()
                    .count()
                    .first(conn)?,
            )
        } else {
            None
        };

        let val = json!({
            "entity_counts": {
                "container": count_entity!(container_ident, conn),
                "creator": count_entity!(creator_ident, conn),
                "file": count_entity!(file_ident, conn),
                "release": count_entity!(release_ident, conn),
                "work": count_entity!(work_ident, conn),
            },
            "merged_editgroups": merged_editgroups,
            "releases_with_dois": releases_with_dois,
            "creators_with_orcids": creators_with_orcids,
            "containers_with_issnls": containers_with_issnls,
            "files_with_releases": files_with_releases,
            "releases_with_files": releases_with_files,
        });
        Ok(StatsResponse { extra: Some(val) })
    }

    entity_batch_handler!(create_container_batch_handler, ContainerEntity);
    entity_batch_handler!(create_creator_batch_handler, CreatorEntity);
    entity_batch_handler!(create_file_batch_handler, FileEntity);
    entity_batch_handler!(create_release_batch_handler, ReleaseEntity);
    entity_batch_handler!(create_work_batch_handler, WorkEntity);
}
