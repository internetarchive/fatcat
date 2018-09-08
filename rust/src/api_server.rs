//! API endpoint handlers

use api_helpers::*;
use chrono;
use database_entity_crud::{EditContext, EntityCrud};
use database_models::*;
use database_schema::{
    abstracts, changelog, container_edit, container_ident, container_rev, creator_edit,
    creator_ident, creator_rev, editgroup, editor, file_edit, file_ident, file_release, file_rev,
    file_rev_url, release_contrib, release_edit, release_ident, release_ref, release_rev,
    release_rev_abstract, work_edit, work_ident, work_rev,
};
use diesel::prelude::*;
use diesel::{self, insert_into};
use errors::*;
use fatcat_api::models;
use fatcat_api::models::*;
use sha1::Sha1;
use std::str::FromStr;
use uuid::Uuid;
use ConnectionPool;

macro_rules! entity_batch_handler {
    ($post_batch_handler:ident, $model:ident) => {
        pub fn $post_batch_handler(
            &self,
            entity_list: &[models::$model],
            autoaccept: bool,
            editgroup: Option<String>,
            conn: &DbConn,
        ) -> Result<Vec<EntityEdit>> {
            // editgroup override logic based on parameters
            let eg_id: Option<Uuid> = match (editgroup, autoaccept) {
                (Some(eg_string), _) => Some(fcid2uuid(&eg_string)?),
                (None, true) => {
                    let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
                        .values((editgroup::editor_id.eq(editor_id),))
                        .get_result(conn)?;
                    Some(eg_row.id)
                },
                (None, false) => None
            };
            for entity in entity_list {
                let mut e = entity.clone();
                // override individual editgroup IDs (if set earlier)
                if let Some(inner_id) = eg_id {
                    e.editgroup_id = Some(uuid2fcid(&inner_id));
                }
                // actual wrapped function call here
                ret.push(self.$post_handler(e, autoaccept, conn)?);
            }
            let mut edit_context = make_edit_context(conn, eg_id)?;
            edit_context.autoaccept = autoaccept;
            let model_list: Vec<&models::$model> = entity_list.iter().map(|e| e).collect();
            let edits = $model::db_create_batch(conn, &edit_context, model_list.as_slice())?;
            if autoaccept {
                // if autoaccept, eg_id is always Some
                let _clr: ChangelogRow = diesel::insert_into(changelog::table)
                    .values((changelog::editgroup_id.eq(eg_id.unwrap()),))
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

fn make_edit_context(conn: &DbConn, editgroup_id: Option<FatCatId>) -> Result<EditContext> {
    let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?; // TODO: auth
    let editgroup_id = match editgroup_id {
        None => FatCatId::from_uuid(&get_or_create_editgroup(editor_id, conn)?),
        Some(param) => param,
    };
    Ok(EditContext {
        editor_id: FatCatId::from_uuid(&editor_id),
        editgroup_id: editgroup_id,
        extra_json: None,
        autoapprove: false,
    })
}

#[derive(Clone)]
pub struct Server {
    pub db_pool: ConnectionPool,
}

impl Server {
    pub fn get_container_handler(
        &self,
        id: &Uuid,
        _expand: Option<String>,
        conn: &DbConn,
    ) -> Result<ContainerEntity> {
        ContainerEntity::db_get(conn, FatCatId::from_uuid(id))
    }

    pub fn lookup_container_handler(&self, issnl: &str, conn: &DbConn) -> Result<ContainerEntity> {
        check_issn(issnl)?;
        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = container_ident::table
            .inner_join(container_rev::table)
            .filter(container_rev::issnl.eq(issnl))
            // This NOT NULL is here to ensure the postgresql query planner that it can use an
            // index
            .filter(container_rev::issnl.is_not_null())
            .filter(container_ident::is_live.eq(true))
            .filter(container_ident::redirect_id.is_null())
            .first(conn)?;

        ContainerEntity::db_from_row(conn, rev, Some(ident))
    }

    pub fn get_creator_handler(
        &self,
        id: &Uuid,
        _expand: Option<String>,
        conn: &DbConn,
    ) -> Result<CreatorEntity> {
        CreatorEntity::db_get(conn, FatCatId::from_uuid(id))
    }

    pub fn lookup_creator_handler(&self, orcid: &str, conn: &DbConn) -> Result<CreatorEntity> {
        check_orcid(orcid)?;
        let (ident, rev): (CreatorIdentRow, CreatorRevRow) = creator_ident::table
            .inner_join(creator_rev::table)
            .filter(creator_rev::orcid.eq(orcid))
            // This NOT NULL is here to ensure the postgresql query planner that it can use an
            // index
            .filter(creator_rev::orcid.is_not_null())
            .filter(creator_ident::is_live.eq(true))
            .filter(creator_ident::redirect_id.is_null())
            .first(conn)?;

        CreatorEntity::db_from_row(conn, rev, Some(ident))
    }

    pub fn get_creator_releases_handler(
        &self,
        id: &str,
        conn: &DbConn,
    ) -> Result<Vec<ReleaseEntity>> {
        let id = fcid2uuid(&id)?;

        // TODO: some kind of unique or group-by?
        let rows: Vec<(ReleaseRevRow, ReleaseIdentRow, ReleaseContribRow)> = release_rev::table
            .inner_join(release_ident::table)
            .inner_join(release_contrib::table)
            .filter(release_contrib::creator_ident_id.eq(&id))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .load(conn)?;

        // TODO: from_rows, not from_row?
        rows.into_iter()
            .map(|(rev, ident, _)| ReleaseEntity::db_from_row(conn, rev, Some(ident)))
            .collect()
    }

    pub fn get_file_handler(
        &self,
        id: &Uuid,
        _expand: Option<String>,
        conn: &DbConn,
    ) -> Result<FileEntity> {
        FileEntity::db_get(conn, FatCatId::from_uuid(id))
    }

    pub fn lookup_file_handler(&self, sha1: &str, conn: &DbConn) -> Result<FileEntity> {
        let (ident, rev): (FileIdentRow, FileRevRow) = file_ident::table
            .inner_join(file_rev::table)
            .filter(file_rev::sha1.eq(sha1))
            // This NOT NULL is here to ensure the postgresql query planner that it can use an
            // index
            .filter(file_rev::sha1.is_not_null())
            .filter(file_ident::is_live.eq(true))
            .filter(file_ident::redirect_id.is_null())
            .first(conn)?;

        FileEntity::db_from_row(conn, rev, Some(ident))
    }

    pub fn get_release_handler(
        &self,
        id: &Uuid,
        expand: Option<String>,
        conn: &DbConn,
    ) -> Result<ReleaseEntity> {
        let mut release = ReleaseEntity::db_get(conn, FatCatId::from_uuid(id))?;

        // For now, if there is any expand param we do them all
        if expand.is_some() {
            release.files =
                Some(self.get_release_files_handler(&release.ident.clone().unwrap(), conn)?);
            if let Some(ref cid) = release.container_id {
                release.container =
                    Some(self.get_container_handler(&fcid2uuid(&cid)?, None, conn)?);
            }
        }
        Ok(release)
    }

    pub fn lookup_release_handler(&self, doi: &str, conn: &DbConn) -> Result<ReleaseEntity> {
        check_doi(doi)?;
        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) = release_ident::table
            .inner_join(release_rev::table)
            .filter(release_rev::doi.eq(doi))
            // This NOT NULL is here to ensure the postgresql query planner that it can use an
            // index
            .filter(release_rev::doi.is_not_null())
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .first(conn)?;

        ReleaseEntity::db_from_row(conn, rev, Some(ident))
    }

    pub fn get_release_files_handler(&self, id: &str, conn: &DbConn) -> Result<Vec<FileEntity>> {
        let ident = FatCatId::from_str(id)?;

        let rows: Vec<(FileRevRow, FileIdentRow, FileReleaseRow)> = file_rev::table
            .inner_join(file_ident::table)
            .inner_join(file_release::table)
            .filter(file_release::target_release_ident_id.eq(&ident.to_uuid()))
            .filter(file_ident::is_live.eq(true))
            .filter(file_ident::redirect_id.is_null())
            .load(conn)?;

        rows.into_iter()
            .map(|(rev, ident, _)| FileEntity::db_from_row(conn, rev, Some(ident)))
            .collect()
    }

    pub fn get_work_handler(
        &self,
        id: &Uuid,
        _expand: Option<String>,
        conn: &DbConn,
    ) -> Result<WorkEntity> {
        WorkEntity::db_get(conn, FatCatId::from_uuid(id))
    }

    pub fn get_work_releases_handler(&self, id: &str, conn: &DbConn) -> Result<Vec<ReleaseEntity>> {
        let id = fcid2uuid(&id)?;

        let rows: Vec<(ReleaseRevRow, ReleaseIdentRow)> = release_rev::table
            .inner_join(release_ident::table)
            .filter(release_rev::work_ident_id.eq(&id))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .load(conn)?;

        rows.into_iter()
            .map(|(rev, ident)| ReleaseEntity::db_from_row(conn, rev, Some(ident)))
            .collect()
    }

    pub fn create_container_handler(
        &self,
        entity: models::ContainerEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_create(conn, &edit_context)?;
        edit.into_model()
    }

    pub fn update_container_handler(
        &self,
        id: &Uuid,
        entity: models::ContainerEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_update(conn, &edit_context, FatCatId::from_uuid(id))?;
        edit.into_model()
    }
    pub fn delete_container_handler(
        &self,
        id: &Uuid,
        editgroup_id: Option<Uuid>,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, editgroup_id.map(|u| FatCatId::from_uuid(&u)))?;
        let edit = ContainerEntity::db_delete(conn, &edit_context, FatCatId::from_uuid(id))?;
        edit.into_model()
    }

    pub fn create_creator_handler(
        &self,
        entity: models::CreatorEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_create(conn, &edit_context)?;
        edit.into_model()
    }

    pub fn update_creator_handler(
        &self,
        id: &Uuid,
        entity: models::CreatorEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_update(conn, &edit_context, FatCatId::from_uuid(id))?;
        edit.into_model()
    }
    pub fn delete_creator_handler(
        &self,
        id: &Uuid,
        editgroup_id: Option<Uuid>,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, editgroup_id.map(|u| FatCatId::from_uuid(&u)))?;
        let edit = CreatorEntity::db_delete(conn, &edit_context, FatCatId::from_uuid(id))?;
        edit.into_model()
    }

    pub fn create_file_handler(
        &self,
        entity: models::FileEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_create(conn, &edit_context)?;
        edit.into_model()
    }

    pub fn update_file_handler(
        &self,
        id: &Uuid,
        entity: models::FileEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_update(conn, &edit_context, FatCatId::from_uuid(id))?;
        edit.into_model()
    }
    pub fn delete_file_handler(
        &self,
        id: &Uuid,
        editgroup_id: Option<Uuid>,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, editgroup_id.map(|u| FatCatId::from_uuid(&u)))?;
        let edit = FileEntity::db_delete(conn, &edit_context, FatCatId::from_uuid(id))?;
        edit.into_model()
    }

    pub fn create_release_handler(
        &self,
        entity: models::ReleaseEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_create(conn, &edit_context)?;
        edit.into_model()
    }

    pub fn update_release_handler(
        &self,
        id: &Uuid,
        entity: models::ReleaseEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_update(conn, &edit_context, FatCatId::from_uuid(id))?;
        edit.into_model()
    }
    pub fn delete_release_handler(
        &self,
        id: &Uuid,
        editgroup_id: Option<Uuid>,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, editgroup_id.map(|u| FatCatId::from_uuid(&u)))?;
        let edit = ReleaseEntity::db_delete(conn, &edit_context, FatCatId::from_uuid(id))?;
        edit.into_model()
    }

    pub fn create_work_handler(
        &self,
        entity: models::WorkEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_create(conn, &edit_context)?;
        edit.into_model()
    }

    pub fn update_work_handler(
        &self,
        id: &Uuid,
        entity: models::WorkEntity,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, entity.parse_editgroup_id()?)?;
        let edit = entity.db_update(conn, &edit_context, FatCatId::from_uuid(id))?;
        edit.into_model()
    }

    pub fn delete_work_handler(
        &self,
        id: &Uuid,
        editgroup_id: Option<Uuid>,
        conn: &DbConn,
    ) -> Result<EntityEdit> {
        let edit_context = make_edit_context(conn, editgroup_id.map(|u| FatCatId::from_uuid(&u)))?;
        let edit = WorkEntity::db_delete(conn, &edit_context, FatCatId::from_uuid(id))?;

        edit.into_model()
    }

    pub fn accept_editgroup_handler(&self, id: &str, conn: &DbConn) -> Result<()> {
        accept_editgroup(fcid2uuid(id)?, conn)?;
        Ok(())
    }

    pub fn create_editgroup_handler(
        &self,
        entity: models::Editgroup,
        conn: &DbConn,
    ) -> Result<Editgroup> {
        let row: EditgroupRow = insert_into(editgroup::table)
            .values((
                editgroup::editor_id.eq(fcid2uuid(&entity.editor_id)?),
                editgroup::description.eq(entity.description),
                editgroup::extra_json.eq(entity.extra),
            ))
            .get_result(conn)?;

        Ok(Editgroup {
            id: Some(uuid2fcid(&row.id)),
            editor_id: uuid2fcid(&row.editor_id),
            description: row.description,
            edits: None,
            extra: row.extra_json,
        })
    }

    pub fn get_editgroup_handler(&self, id: &str, conn: &DbConn) -> Result<Editgroup> {
        let id = fcid2uuid(id)?;
        let row: EditgroupRow = editgroup::table.find(id).first(conn)?;

        let edits = EditgroupEdits {
            containers: Some(
                container_edit::table
                    .filter(container_edit::editgroup_id.eq(id))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: ContainerEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            creators: Some(
                creator_edit::table
                    .filter(creator_edit::editgroup_id.eq(id))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: CreatorEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            files: Some(
                file_edit::table
                    .filter(file_edit::editgroup_id.eq(id))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: FileEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            releases: Some(
                release_edit::table
                    .filter(release_edit::editgroup_id.eq(id))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: ReleaseEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            works: Some(
                work_edit::table
                    .filter(work_edit::editgroup_id.eq(id))
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

    pub fn get_editor_handler(&self, id: &str, conn: &DbConn) -> Result<Editor> {
        let id = fcid2uuid(id)?;
        let row: EditorRow = editor::table.find(id).first(conn)?;

        let ed = Editor {
            id: Some(uuid2fcid(&row.id)),
            username: row.username,
        };
        Ok(ed)
    }

    pub fn editor_changelog_get_handler(
        &self,
        id: &str,
        conn: &DbConn,
    ) -> Result<Vec<ChangelogEntry>> {
        let id = fcid2uuid(id)?;
        // TODO: single query
        let editor: EditorRow = editor::table.find(id).first(conn)?;
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
            })
            .collect();
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
            })
            .collect();
        Ok(entries)
    }

    pub fn get_changelog_entry_handler(&self, id: i64, conn: &DbConn) -> Result<ChangelogEntry> {
        let cl_row: ChangelogRow = changelog::table.find(id).first(conn)?;
        let editgroup = self.get_editgroup_handler(&uuid2fcid(&cl_row.editgroup_id), conn)?;

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
            Some(file_rev::table
                .inner_join(file_ident::table)
                .inner_join(file_release::table)
                .filter(file_ident::is_live.eq(true))
                .filter(file_ident::redirect_id.is_null())
                .select(file_ident::id)
                .distinct()
                .count()
                .first(conn)?)
        } else {
            None
        };
        let releases_with_files: Option<i64> = if more.is_some() {
            // this slightly overcounts also: it will include releases which are only linked to from
            // inactive files
            Some(release_ident::table
                .inner_join(file_release::table)
                .filter(release_ident::is_live.eq(true))
                .filter(release_ident::redirect_id.is_null())
                .select(file_release::target_release_ident_id)
                .distinct()
                .count()
                .first(conn)?)
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

    pub fn get_container_history_handler(
        &self,
        id: &Uuid,
        limit: Option<i64>,
        conn: &DbConn,
    ) -> Result<Vec<EntityHistoryEntry>> {
        ContainerEntity::db_get_history(conn, FatCatId::from_uuid(id), limit)
    }
    pub fn get_creator_history_handler(
        &self,
        id: &Uuid,
        limit: Option<i64>,
        conn: &DbConn,
    ) -> Result<Vec<EntityHistoryEntry>> {
        CreatorEntity::db_get_history(conn, FatCatId::from_uuid(id), limit)
    }
    pub fn get_file_history_handler(
        &self,
        id: &Uuid,
        limit: Option<i64>,
        conn: &DbConn,
    ) -> Result<Vec<EntityHistoryEntry>> {
        FileEntity::db_get_history(conn, FatCatId::from_uuid(id), limit)
    }
    pub fn get_release_history_handler(
        &self,
        id: &Uuid,
        limit: Option<i64>,
        conn: &DbConn,
    ) -> Result<Vec<EntityHistoryEntry>> {
        ReleaseEntity::db_get_history(conn, FatCatId::from_uuid(id), limit)
    }
    pub fn get_work_history_handler(
        &self,
        id: &Uuid,
        limit: Option<i64>,
        conn: &DbConn,
    ) -> Result<Vec<EntityHistoryEntry>> {
        WorkEntity::db_get_history(conn, FatCatId::from_uuid(id), limit)
    }
}
