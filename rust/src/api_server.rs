//! API endpoint handlers

use crate::api_entity_crud::EntityCrud;
use crate::api_helpers::*;
use crate::auth::*;
use chrono;
use crate::database_models::*;
use crate::database_schema::*;
use diesel::prelude::*;
use diesel::{self, insert_into};
use crate::errors::*;
use fatcat_api_spec::models;
use fatcat_api_spec::models::*;
use std::str::FromStr;
use crate::ConnectionPool;

macro_rules! entity_batch_handler {
    ($post_batch_handler:ident, $model:ident) => {
        pub fn $post_batch_handler(
            &self,
            entity_list: &[models::$model],
            autoaccept: bool,
            editor_id: FatCatId,
            editgroup_id: Option<FatCatId>,
            conn: &DbConn,
        ) -> Result<Vec<EntityEdit>> {

            let edit_context = make_edit_context(conn, editor_id, editgroup_id, autoaccept)?;
            edit_context.check(&conn)?;
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

#[derive(Clone)]
pub struct Server {
    pub db_pool: ConnectionPool,
    pub auth_confectionary: AuthConfectionary,
}

pub fn get_release_files(
    ident: FatCatId,
    hide_flags: HideFlags,
    conn: &DbConn,
) -> Result<Vec<FileEntity>> {
    let rows: Vec<(FileRevRow, FileIdentRow, FileRevReleaseRow)> = file_rev::table
        .inner_join(file_ident::table)
        .inner_join(file_rev_release::table)
        .filter(file_rev_release::target_release_ident_id.eq(&ident.to_uuid()))
        .filter(file_ident::is_live.eq(true))
        .filter(file_ident::redirect_id.is_null())
        .load(conn)?;

    rows.into_iter()
        .map(|(rev, ident, _)| FileEntity::db_from_row(conn, rev, Some(ident), hide_flags))
        .collect()
}

pub fn get_release_filesets(
    ident: FatCatId,
    hide_flags: HideFlags,
    conn: &DbConn,
) -> Result<Vec<FilesetEntity>> {
    let rows: Vec<(FilesetRevRow, FilesetIdentRow, FilesetRevReleaseRow)> = fileset_rev::table
        .inner_join(fileset_ident::table)
        .inner_join(fileset_rev_release::table)
        .filter(fileset_rev_release::target_release_ident_id.eq(&ident.to_uuid()))
        .filter(fileset_ident::is_live.eq(true))
        .filter(fileset_ident::redirect_id.is_null())
        .load(conn)?;

    rows.into_iter()
        .map(|(rev, ident, _)| FilesetEntity::db_from_row(conn, rev, Some(ident), hide_flags))
        .collect()
}

pub fn get_release_webcaptures(
    ident: FatCatId,
    hide_flags: HideFlags,
    conn: &DbConn,
) -> Result<Vec<WebcaptureEntity>> {
    let rows: Vec<(
        WebcaptureRevRow,
        WebcaptureIdentRow,
        WebcaptureRevReleaseRow,
    )> = webcapture_rev::table
        .inner_join(webcapture_ident::table)
        .inner_join(webcapture_rev_release::table)
        .filter(webcapture_rev_release::target_release_ident_id.eq(&ident.to_uuid()))
        .filter(webcapture_ident::is_live.eq(true))
        .filter(webcapture_ident::redirect_id.is_null())
        .load(conn)?;

    rows.into_iter()
        .map(|(rev, ident, _)| WebcaptureEntity::db_from_row(conn, rev, Some(ident), hide_flags))
        .collect()
}

impl Server {
    pub fn lookup_container_handler(
        &self,
        issnl: &Option<String>,
        wikidata_qid: &Option<String>,
        expand_flags: ExpandFlags,
        hide_flags: HideFlags,
        conn: &DbConn,
    ) -> Result<ContainerEntity> {
        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = match (issnl, wikidata_qid) {
            (Some(issnl), None) => {
                check_issn(issnl)?;
                container_ident::table
                    .inner_join(container_rev::table)
                    .filter(container_rev::issnl.eq(&issnl))
                    .filter(container_ident::is_live.eq(true))
                    .filter(container_ident::redirect_id.is_null())
                    .first(conn)?
            }
            (None, Some(wikidata_qid)) => {
                check_wikidata_qid(wikidata_qid)?;
                container_ident::table
                    .inner_join(container_rev::table)
                    .filter(container_rev::wikidata_qid.eq(&wikidata_qid))
                    .filter(container_ident::is_live.eq(true))
                    .filter(container_ident::redirect_id.is_null())
                    .first(conn)?
            }
            _ => {
                return Err(ErrorKind::MissingOrMultipleExternalId("in lookup".to_string()).into());
            }
        };

        let mut entity = ContainerEntity::db_from_row(conn, rev, Some(ident), hide_flags)?;
        entity.db_expand(&conn, expand_flags)?;
        Ok(entity)
    }

    pub fn lookup_creator_handler(
        &self,
        orcid: &Option<String>,
        wikidata_qid: &Option<String>,
        expand_flags: ExpandFlags,
        hide_flags: HideFlags,
        conn: &DbConn,
    ) -> Result<CreatorEntity> {
        let (ident, rev): (CreatorIdentRow, CreatorRevRow) = match (orcid, wikidata_qid) {
            (Some(orcid), None) => {
                check_orcid(orcid)?;
                creator_ident::table
                    .inner_join(creator_rev::table)
                    .filter(creator_rev::orcid.eq(orcid))
                    .filter(creator_ident::is_live.eq(true))
                    .filter(creator_ident::redirect_id.is_null())
                    .first(conn)?
            }
            (None, Some(wikidata_qid)) => {
                check_wikidata_qid(wikidata_qid)?;
                creator_ident::table
                    .inner_join(creator_rev::table)
                    .filter(creator_rev::wikidata_qid.eq(wikidata_qid))
                    .filter(creator_ident::is_live.eq(true))
                    .filter(creator_ident::redirect_id.is_null())
                    .first(conn)?
            }
            _ => {
                return Err(ErrorKind::MissingOrMultipleExternalId("in lookup".to_string()).into());
            }
        };

        let mut entity = CreatorEntity::db_from_row(conn, rev, Some(ident), hide_flags)?;
        entity.db_expand(&conn, expand_flags)?;
        Ok(entity)
    }

    pub fn get_creator_releases_handler(
        &self,
        ident: FatCatId,
        hide_flags: HideFlags,
        conn: &DbConn,
    ) -> Result<Vec<ReleaseEntity>> {
        // TODO: some kind of unique or group-by?
        let rows: Vec<(ReleaseRevRow, ReleaseIdentRow, ReleaseContribRow)> = release_rev::table
            .inner_join(release_ident::table)
            .inner_join(release_contrib::table)
            .filter(release_contrib::creator_ident_id.eq(&ident.to_uuid()))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .load(conn)?;

        // TODO: from_rows, not from_row?
        rows.into_iter()
            .map(|(rev, ident, _)| ReleaseEntity::db_from_row(conn, rev, Some(ident), hide_flags))
            .collect()
    }

    pub fn lookup_file_handler(
        &self,
        md5: &Option<String>,
        sha1: &Option<String>,
        sha256: &Option<String>,
        expand_flags: ExpandFlags,
        hide_flags: HideFlags,
        conn: &DbConn,
    ) -> Result<FileEntity> {
        let (ident, rev): (FileIdentRow, FileRevRow) = match (md5, sha1, sha256) {
            (Some(md5), None, None) => {
                check_md5(md5)?;
                file_ident::table
                    .inner_join(file_rev::table)
                    .filter(file_rev::md5.eq(md5))
                    .filter(file_ident::is_live.eq(true))
                    .filter(file_ident::redirect_id.is_null())
                    .first(conn)?
            }
            (None, Some(sha1), None) => {
                check_sha1(sha1)?;
                file_ident::table
                    .inner_join(file_rev::table)
                    .filter(file_rev::sha1.eq(sha1))
                    .filter(file_ident::is_live.eq(true))
                    .filter(file_ident::redirect_id.is_null())
                    .first(conn)?
            }
            (None, None, Some(sha256)) => {
                check_sha256(sha256)?;
                file_ident::table
                    .inner_join(file_rev::table)
                    .filter(file_rev::sha256.eq(sha256))
                    .filter(file_ident::is_live.eq(true))
                    .filter(file_ident::redirect_id.is_null())
                    .first(conn)?
            }
            _ => {
                return Err(ErrorKind::MissingOrMultipleExternalId("in lookup".to_string()).into());
            }
        };

        let mut entity = FileEntity::db_from_row(conn, rev, Some(ident), hide_flags)?;
        entity.db_expand(&conn, expand_flags)?;
        Ok(entity)
    }

    pub fn lookup_release_handler(
        &self,
        doi: &Option<String>,
        wikidata_qid: &Option<String>,
        isbn13: &Option<String>,
        pmid: &Option<String>,
        pmcid: &Option<String>,
        core_id: &Option<String>,
        expand_flags: ExpandFlags,
        hide_flags: HideFlags,
        conn: &DbConn,
    ) -> Result<ReleaseEntity> {
        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) =
            match (doi, wikidata_qid, isbn13, pmid, pmcid, core_id) {
                (Some(doi), None, None, None, None, None) => {
                    check_doi(doi)?;
                    release_ident::table
                        .inner_join(release_rev::table)
                        .filter(release_rev::doi.eq(doi))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?
                }
                (None, Some(wikidata_qid), None, None, None, None) => {
                    check_wikidata_qid(wikidata_qid)?;
                    release_ident::table
                        .inner_join(release_rev::table)
                        .filter(release_rev::wikidata_qid.eq(wikidata_qid))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?
                }
                (None, None, Some(isbn13), None, None, None) => {
                    // TODO: check_isbn13(isbn13)?;
                    release_ident::table
                        .inner_join(release_rev::table)
                        .filter(release_rev::isbn13.eq(isbn13))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?
                }
                (None, None, None, Some(pmid), None, None) => {
                    check_pmid(pmid)?;
                    release_ident::table
                        .inner_join(release_rev::table)
                        .filter(release_rev::pmid.eq(pmid))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?
                }
                (None, None, None, None, Some(pmcid), None) => {
                    check_pmcid(pmcid)?;
                    release_ident::table
                        .inner_join(release_rev::table)
                        .filter(release_rev::pmcid.eq(pmcid))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?
                }
                (None, None, None, None, None, Some(core_id)) => {
                    // TODO: check_core_id(core_id)?;
                    release_ident::table
                        .inner_join(release_rev::table)
                        .filter(release_rev::core_id.eq(core_id))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?
                }
                _ => {
                    return Err(
                        ErrorKind::MissingOrMultipleExternalId("in lookup".to_string()).into(),
                    );
                }
            };

        let mut entity = ReleaseEntity::db_from_row(conn, rev, Some(ident), hide_flags)?;
        entity.db_expand(&conn, expand_flags)?;
        Ok(entity)
    }

    pub fn get_release_files_handler(
        &self,
        ident: FatCatId,
        hide_flags: HideFlags,
        conn: &DbConn,
    ) -> Result<Vec<FileEntity>> {
        get_release_files(ident, hide_flags, conn)
    }

    pub fn get_release_filesets_handler(
        &self,
        ident: FatCatId,
        hide_flags: HideFlags,
        conn: &DbConn,
    ) -> Result<Vec<FilesetEntity>> {
        get_release_filesets(ident, hide_flags, conn)
    }

    pub fn get_release_webcaptures_handler(
        &self,
        ident: FatCatId,
        hide_flags: HideFlags,
        conn: &DbConn,
    ) -> Result<Vec<WebcaptureEntity>> {
        get_release_webcaptures(ident, hide_flags, conn)
    }

    pub fn get_work_releases_handler(
        &self,
        ident: FatCatId,
        hide_flags: HideFlags,
        conn: &DbConn,
    ) -> Result<Vec<ReleaseEntity>> {
        let rows: Vec<(ReleaseRevRow, ReleaseIdentRow)> = release_rev::table
            .inner_join(release_ident::table)
            .filter(release_rev::work_ident_id.eq(&ident.to_uuid()))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .load(conn)?;

        rows.into_iter()
            .map(|(rev, ident)| ReleaseEntity::db_from_row(conn, rev, Some(ident), hide_flags))
            .collect()
    }

    pub fn accept_editgroup_handler(&self, editgroup_id: FatCatId, conn: &DbConn) -> Result<()> {
        accept_editgroup(editgroup_id, conn)?;
        Ok(())
    }

    pub fn create_editgroup_handler(
        &self,
        entity: models::Editgroup,
        conn: &DbConn,
    ) -> Result<Editgroup> {
        let row: EditgroupRow = insert_into(editgroup::table)
            .values((
                editgroup::editor_id.eq(FatCatId::from_str(&entity.editor_id.unwrap())?.to_uuid()),
                editgroup::description.eq(entity.description),
                editgroup::extra_json.eq(entity.extra),
            ))
            .get_result(conn)?;

        Ok(Editgroup {
            editgroup_id: Some(uuid2fcid(&row.id)),
            editor_id: Some(uuid2fcid(&row.editor_id)),
            description: row.description,
            edits: None,
            extra: row.extra_json,
        })
    }

    pub fn get_editgroup_handler(
        &self,
        editgroup_id: FatCatId,
        conn: &DbConn,
    ) -> Result<Editgroup> {
        let row: EditgroupRow = editgroup::table.find(editgroup_id.to_uuid()).first(conn)?;

        let edits = EditgroupEdits {
            containers: Some(
                container_edit::table
                    .filter(container_edit::editgroup_id.eq(editgroup_id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: ContainerEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            creators: Some(
                creator_edit::table
                    .filter(creator_edit::editgroup_id.eq(editgroup_id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: CreatorEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            files: Some(
                file_edit::table
                    .filter(file_edit::editgroup_id.eq(editgroup_id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: FileEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            filesets: Some(
                fileset_edit::table
                    .filter(fileset_edit::editgroup_id.eq(editgroup_id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: FilesetEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            webcaptures: Some(
                webcapture_edit::table
                    .filter(webcapture_edit::editgroup_id.eq(editgroup_id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: WebcaptureEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            releases: Some(
                release_edit::table
                    .filter(release_edit::editgroup_id.eq(editgroup_id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: ReleaseEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            works: Some(
                work_edit::table
                    .filter(work_edit::editgroup_id.eq(editgroup_id.to_uuid()))
                    .get_results(conn)?
                    .into_iter()
                    .map(|e: WorkEditRow| e.into_model().unwrap())
                    .collect(),
            ),
        };

        let eg = Editgroup {
            editgroup_id: Some(uuid2fcid(&row.id)),
            editor_id: Some(uuid2fcid(&row.editor_id)),
            description: row.description,
            edits: Some(edits),
            extra: row.extra_json,
        };
        Ok(eg)
    }

    pub fn get_editor_handler(&self, editor_id: FatCatId, conn: &DbConn) -> Result<Editor> {
        let row: EditorRow = editor::table.find(editor_id.to_uuid()).first(conn)?;
        Ok(row.into_model())
    }

    pub fn get_editor_changelog_handler(
        &self,
        editor_id: FatCatId,
        conn: &DbConn,
    ) -> Result<Vec<ChangelogEntry>> {
        // TODO: single query
        let editor: EditorRow = editor::table.find(editor_id.to_uuid()).first(conn)?;
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

    pub fn get_changelog_entry_handler(&self, index: i64, conn: &DbConn) -> Result<ChangelogEntry> {
        let cl_row: ChangelogRow = changelog::table.find(index).first(conn)?;
        let editgroup =
            self.get_editgroup_handler(FatCatId::from_uuid(&cl_row.editgroup_id), conn)?;

        let mut entry = cl_row.into_model();
        entry.editgroup = Some(editgroup);
        Ok(entry)
    }

    /// This helper either finds an Editor model by OIDC parameters (eg, remote domain and
    /// identifier), or creates one and inserts the appropriate auth rows. The semantics are
    /// basically an "upsert" of signup/account-creation.
    /// Returns an editor model and boolean flag indicating whether a new editor was created or
    /// not.
    /// If this function creates an editor, it sets the username to
    /// "{preferred_username}-{provider}"; the intent is for this to be temporary but unique. Might
    /// look like "bnewbold-github", or might look like "895139824-github". This is a hack to make
    /// check/creation idempotent.
    pub fn auth_oidc_handler(&self, params: AuthOidc, conn: &DbConn) -> Result<(Editor, bool)> {
        let existing: Vec<(EditorRow, AuthOidcRow)> = editor::table
            .inner_join(auth_oidc::table)
            .filter(auth_oidc::oidc_sub.eq(params.sub.clone()))
            .filter(auth_oidc::oidc_iss.eq(params.iss.clone()))
            .load(conn)?;

        let (editor_row, created): (EditorRow, bool) = match existing.first() {
            Some((editor, _)) => (editor.clone(), false),
            None => {
                let username = format!("{}-{}", params.preferred_username, params.provider);
                let editor = create_editor(conn, username, false, false)?;
                // create an auth login row so the user can log back in
                diesel::insert_into(auth_oidc::table)
                    .values((
                        auth_oidc::editor_id.eq(editor.id),
                        auth_oidc::provider.eq(params.provider),
                        auth_oidc::oidc_iss.eq(params.iss),
                        auth_oidc::oidc_sub.eq(params.sub),
                    ))
                    .execute(conn)?;
                (editor, true)
            }
        };

        Ok((editor_row.into_model(), created))
    }

    entity_batch_handler!(create_container_batch_handler, ContainerEntity);
    entity_batch_handler!(create_creator_batch_handler, CreatorEntity);
    entity_batch_handler!(create_file_batch_handler, FileEntity);
    entity_batch_handler!(create_fileset_batch_handler, FilesetEntity);
    entity_batch_handler!(create_webcapture_batch_handler, WebcaptureEntity);
    entity_batch_handler!(create_release_batch_handler, ReleaseEntity);
    entity_batch_handler!(create_work_batch_handler, WorkEntity);
}
