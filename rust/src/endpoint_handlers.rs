//! API endpoint handlers
//!
//! This module contains actual implementations of endpoints with rust-style type signatures.
//!
//! The remaining functions here should probably be refactored away. The 'get_release_files' style
//! functions should go in entity_crud (or some new

use crate::database_models::*;
use crate::database_schema::*;
use crate::editing::*;
use crate::editing_crud::{EditgroupCrud, EditorCrud};
use crate::entity_crud::{EntityCrud, ExpandFlags, HideFlags};
use crate::errors::*;
use crate::identifiers::*;
use crate::server::*;
use diesel::prelude::*;
use fatcat_openapi::models;
use fatcat_openapi::models::*;

macro_rules! entity_auto_batch_handler {
    ($post_auto_batch_handler:ident, $model:ident) => {
        pub fn $post_auto_batch_handler(
            &self,
            conn: &DbConn,
            editgroup: Editgroup,
            entity_list: &[models::$model],
            editor_id: FatcatId,
        ) -> Result<Editgroup> {
            let editgroup_row = editgroup.db_create(conn, true)?;
            let editgroup_id = FatcatId::from_uuid(&editgroup_row.id);
            let edit_context = make_edit_context(editor_id, editgroup_id, true)?;
            edit_context.check(&conn)?;
            let model_list: Vec<&models::$model> = entity_list.iter().map(|e| e).collect();
            let _edits = $model::db_create_batch(conn, &edit_context, model_list.as_slice())?;

            let _clr: ChangelogRow = diesel::insert_into(changelog::table)
                .values((changelog::editgroup_id.eq(edit_context.editgroup_id.to_uuid()),))
                .get_result(conn)?;
            self.get_editgroup_handler(conn, editgroup_id)
        }
    };
}

pub fn get_release_files(
    conn: &DbConn,
    ident: FatcatId,
    hide_flags: HideFlags,
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
    conn: &DbConn,
    ident: FatcatId,
    hide_flags: HideFlags,
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
    conn: &DbConn,
    ident: FatcatId,
    hide_flags: HideFlags,
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
        conn: &DbConn,
        issnl: &Option<String>,
        issne: &Option<String>,
        issnp: &Option<String>,
        issn: &Option<String>,
        wikidata_qid: &Option<String>,
        expand_flags: ExpandFlags,
        hide_flags: HideFlags,
    ) -> Result<ContainerEntity> {
        let (ident, rev): (ContainerIdentRow, ContainerRevRow) =
            match (issnl, issnp, issne, issn, wikidata_qid) {
                (Some(issnl), None, None, None, None) => {
                    check_issn(issnl)?;
                    container_ident::table
                        .inner_join(container_rev::table)
                        .filter(container_rev::issnl.eq(&issnl))
                        .filter(container_ident::is_live.eq(true))
                        .filter(container_ident::redirect_id.is_null())
                        .first(conn)?
                }
                (None, Some(issnp), None, None, None) => {
                    check_issn(issnp)?;
                    container_ident::table
                        .inner_join(container_rev::table)
                        .filter(container_rev::issnp.eq(&issnp))
                        .filter(container_ident::is_live.eq(true))
                        .filter(container_ident::redirect_id.is_null())
                        .first(conn)?
                }
                (None, None, Some(issne), None, None) => {
                    check_issn(issne)?;
                    container_ident::table
                        .inner_join(container_rev::table)
                        .filter(container_rev::issne.eq(&issne))
                        .filter(container_ident::is_live.eq(true))
                        .filter(container_ident::redirect_id.is_null())
                        .first(conn)?
                }
                (None, None, None, Some(issn), None) => {
                    check_issn(issn)?;
                    container_ident::table
                        .inner_join(container_rev::table)
                        .filter(
                            container_rev::issnl
                                .eq(&issn)
                                .or(container_rev::issnp.eq(&issn))
                                .or(container_rev::issne.eq(&issn)),
                        )
                        .filter(container_ident::is_live.eq(true))
                        .filter(container_ident::redirect_id.is_null())
                        .first(conn)?
                }
                (None, None, None, None, Some(wikidata_qid)) => {
                    check_wikidata_qid(wikidata_qid)?;
                    container_ident::table
                        .inner_join(container_rev::table)
                        .filter(container_rev::wikidata_qid.eq(&wikidata_qid))
                        .filter(container_ident::is_live.eq(true))
                        .filter(container_ident::redirect_id.is_null())
                        .first(conn)?
                }
                _ => {
                    return Err(
                        FatcatError::MissingOrMultipleExternalId("in lookup".to_string()).into(),
                    );
                }
            };

        let mut entity = ContainerEntity::db_from_row(conn, rev, Some(ident), hide_flags)?;
        entity.db_expand(conn, expand_flags)?;
        Ok(entity)
    }

    pub fn lookup_creator_handler(
        &self,
        conn: &DbConn,
        orcid: &Option<String>,
        wikidata_qid: &Option<String>,
        expand_flags: ExpandFlags,
        hide_flags: HideFlags,
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
                return Err(
                    FatcatError::MissingOrMultipleExternalId("in lookup".to_string()).into(),
                );
            }
        };

        let mut entity = CreatorEntity::db_from_row(conn, rev, Some(ident), hide_flags)?;
        entity.db_expand(conn, expand_flags)?;
        Ok(entity)
    }

    pub fn get_creator_releases_handler(
        &self,
        conn: &DbConn,
        ident: FatcatId,
        hide_flags: HideFlags,
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
        conn: &DbConn,
        md5: &Option<String>,
        sha1: &Option<String>,
        sha256: &Option<String>,
        expand_flags: ExpandFlags,
        hide_flags: HideFlags,
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
                return Err(
                    FatcatError::MissingOrMultipleExternalId("in lookup".to_string()).into(),
                );
            }
        };

        let mut entity = FileEntity::db_from_row(conn, rev, Some(ident), hide_flags)?;
        entity.db_expand(conn, expand_flags)?;
        Ok(entity)
    }

    pub fn lookup_release_handler(
        &self,
        conn: &DbConn,
        doi: &Option<String>,
        wikidata_qid: &Option<String>,
        isbn13: &Option<String>,
        pmid: &Option<String>,
        pmcid: &Option<String>,
        core: &Option<String>,
        arxiv: &Option<String>,
        jstor: &Option<String>,
        ark: &Option<String>,
        mag: &Option<String>,
        doaj: &Option<String>,
        dblp: &Option<String>,
        oai: &Option<String>,
        hdl: &Option<String>,
        expand_flags: ExpandFlags,
        hide_flags: HideFlags,
    ) -> Result<ReleaseEntity> {
        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) = match (
            doi,
            wikidata_qid,
            isbn13,
            pmid,
            pmcid,
            core,
            arxiv,
            jstor,
            ark,
            mag,
            doaj,
            dblp,
            oai,
            hdl,
        ) {
            (
                Some(doi),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ) => {
                // DOIs always stored lower-case; lookups are case-insensitive
                let doi = doi.to_lowercase();
                check_doi(&doi)?;
                release_ident::table
                    .inner_join(release_rev::table)
                    .filter(release_rev::doi.eq(&doi))
                    .filter(release_ident::is_live.eq(true))
                    .filter(release_ident::redirect_id.is_null())
                    .first(conn)?
            }
            (
                None,
                Some(wikidata_qid),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ) => {
                check_wikidata_qid(wikidata_qid)?;
                release_ident::table
                    .inner_join(release_rev::table)
                    .filter(release_rev::wikidata_qid.eq(wikidata_qid))
                    .filter(release_ident::is_live.eq(true))
                    .filter(release_ident::redirect_id.is_null())
                    .first(conn)?
            }
            (
                None,
                None,
                Some(isbn13),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ) => {
                check_isbn13(isbn13)?;
                let (rev, ident, _extid): (ReleaseRevRow, ReleaseIdentRow, ReleaseExtidRow) =
                    release_rev::table
                        .inner_join(release_ident::table)
                        .inner_join(release_rev_extid::table)
                        .filter(release_rev_extid::extid_type.eq("isbn13".to_string()))
                        .filter(release_rev_extid::value.eq(isbn13))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?;
                (ident, rev)
            }
            (
                None,
                None,
                None,
                Some(pmid),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ) => {
                check_pmid(pmid)?;
                release_ident::table
                    .inner_join(release_rev::table)
                    .filter(release_rev::pmid.eq(pmid))
                    .filter(release_ident::is_live.eq(true))
                    .filter(release_ident::redirect_id.is_null())
                    .first(conn)?
            }
            (
                None,
                None,
                None,
                None,
                Some(pmcid),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ) => {
                check_pmcid(pmcid)?;
                release_ident::table
                    .inner_join(release_rev::table)
                    .filter(release_rev::pmcid.eq(pmcid))
                    .filter(release_ident::is_live.eq(true))
                    .filter(release_ident::redirect_id.is_null())
                    .first(conn)?
            }
            (
                None,
                None,
                None,
                None,
                None,
                Some(core),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ) => {
                check_core_id(core)?;
                release_ident::table
                    .inner_join(release_rev::table)
                    .filter(release_rev::core_id.eq(core))
                    .filter(release_ident::is_live.eq(true))
                    .filter(release_ident::redirect_id.is_null())
                    .first(conn)?
            }
            (
                None,
                None,
                None,
                None,
                None,
                None,
                Some(arxiv),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ) => {
                // TODO: this allows only lookup by full, versioned arxiv identifier. Probably also
                // want to allow lookup by "work" style identifier?
                check_arxiv_id(arxiv)?;
                let (rev, ident, _extid): (ReleaseRevRow, ReleaseIdentRow, ReleaseExtidRow) =
                    release_rev::table
                        .inner_join(release_ident::table)
                        .inner_join(release_rev_extid::table)
                        .filter(release_rev_extid::extid_type.eq("arxiv".to_string()))
                        .filter(release_rev_extid::value.eq(arxiv))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?;
                (ident, rev)
            }
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(jstor),
                None,
                None,
                None,
                None,
                None,
                None,
            ) => {
                check_jstor_id(jstor)?;
                let (rev, ident, _extid): (ReleaseRevRow, ReleaseIdentRow, ReleaseExtidRow) =
                    release_rev::table
                        .inner_join(release_ident::table)
                        .inner_join(release_rev_extid::table)
                        .filter(release_rev_extid::extid_type.eq("jstor".to_string()))
                        .filter(release_rev_extid::value.eq(jstor))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?;
                (ident, rev)
            }
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(ark),
                None,
                None,
                None,
                None,
                None,
            ) => {
                check_ark_id(ark)?;
                let (rev, ident, _extid): (ReleaseRevRow, ReleaseIdentRow, ReleaseExtidRow) =
                    release_rev::table
                        .inner_join(release_ident::table)
                        .inner_join(release_rev_extid::table)
                        .filter(release_rev_extid::extid_type.eq("ark".to_string()))
                        .filter(release_rev_extid::value.eq(ark))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?;
                (ident, rev)
            }
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(mag),
                None,
                None,
                None,
                None,
            ) => {
                check_mag_id(mag)?;
                let (rev, ident, _extid): (ReleaseRevRow, ReleaseIdentRow, ReleaseExtidRow) =
                    release_rev::table
                        .inner_join(release_ident::table)
                        .inner_join(release_rev_extid::table)
                        .filter(release_rev_extid::extid_type.eq("mag".to_string()))
                        .filter(release_rev_extid::value.eq(mag))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?;
                (ident, rev)
            }
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(doaj),
                None,
                None,
                None,
            ) => {
                check_doaj_id(doaj)?;
                let (rev, ident, _extid): (ReleaseRevRow, ReleaseIdentRow, ReleaseExtidRow) =
                    release_rev::table
                        .inner_join(release_ident::table)
                        .inner_join(release_rev_extid::table)
                        .filter(release_rev_extid::extid_type.eq("doaj".to_string()))
                        .filter(release_rev_extid::value.eq(doaj))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?;
                (ident, rev)
            }
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(dblp),
                None,
                None,
            ) => {
                check_dblp_key(dblp)?;
                let (rev, ident, _extid): (ReleaseRevRow, ReleaseIdentRow, ReleaseExtidRow) =
                    release_rev::table
                        .inner_join(release_ident::table)
                        .inner_join(release_rev_extid::table)
                        .filter(release_rev_extid::extid_type.eq("dblp".to_string()))
                        .filter(release_rev_extid::value.eq(dblp))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?;
                (ident, rev)
            }
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(oai),
                None,
            ) => {
                check_oai_id(oai)?;
                let (rev, ident, _extid): (ReleaseRevRow, ReleaseIdentRow, ReleaseExtidRow) =
                    release_rev::table
                        .inner_join(release_ident::table)
                        .inner_join(release_rev_extid::table)
                        .filter(release_rev_extid::extid_type.eq("oai".to_string()))
                        .filter(release_rev_extid::value.eq(oai))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?;
                (ident, rev)
            }
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(hdl),
            ) => {
                let hdl = hdl.to_lowercase();
                check_hdl(&hdl)?;
                let (rev, ident, _extid): (ReleaseRevRow, ReleaseIdentRow, ReleaseExtidRow) =
                    release_rev::table
                        .inner_join(release_ident::table)
                        .inner_join(release_rev_extid::table)
                        .filter(release_rev_extid::extid_type.eq("hdl".to_string()))
                        .filter(release_rev_extid::value.eq(hdl))
                        .filter(release_ident::is_live.eq(true))
                        .filter(release_ident::redirect_id.is_null())
                        .first(conn)?;
                (ident, rev)
            }
            _ => {
                return Err(
                    FatcatError::MissingOrMultipleExternalId("in lookup".to_string()).into(),
                );
            }
        };

        let mut entity = ReleaseEntity::db_from_row(conn, rev, Some(ident), hide_flags)?;
        entity.db_expand(conn, expand_flags)?;
        Ok(entity)
    }

    pub fn get_release_files_handler(
        &self,
        conn: &DbConn,
        ident: FatcatId,
        hide_flags: HideFlags,
    ) -> Result<Vec<FileEntity>> {
        get_release_files(conn, ident, hide_flags)
    }

    pub fn get_release_filesets_handler(
        &self,
        conn: &DbConn,
        ident: FatcatId,
        hide_flags: HideFlags,
    ) -> Result<Vec<FilesetEntity>> {
        get_release_filesets(conn, ident, hide_flags)
    }

    pub fn get_release_webcaptures_handler(
        &self,
        conn: &DbConn,
        ident: FatcatId,
        hide_flags: HideFlags,
    ) -> Result<Vec<WebcaptureEntity>> {
        get_release_webcaptures(conn, ident, hide_flags)
    }

    pub fn get_work_releases_handler(
        &self,
        conn: &DbConn,
        ident: FatcatId,
        hide_flags: HideFlags,
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

    pub fn accept_editgroup_handler(&self, conn: &DbConn, editgroup_id: FatcatId) -> Result<()> {
        accept_editgroup(conn, editgroup_id)?;
        Ok(())
    }

    pub fn create_editgroup_handler(
        &self,
        conn: &DbConn,
        editgroup: models::Editgroup,
    ) -> Result<Editgroup> {
        let row = editgroup.db_create(conn, false)?;
        Ok(row.into_model_partial(None, None))
    }

    pub fn get_editgroup_handler(
        &self,
        conn: &DbConn,
        editgroup_id: FatcatId,
    ) -> Result<Editgroup> {
        let (eg_row, cl_row) = Editgroup::db_get_with_changelog(conn, editgroup_id)?;
        let editor = Editor::db_get(&conn, FatcatId::from_uuid(&eg_row.editor_id))?.into_model();
        let mut editgroup = eg_row.into_model_partial(cl_row.map(|cl| cl.id), Some(editor));

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

        editgroup.edits = Some(edits);
        Ok(editgroup)
    }

    pub fn get_editor_handler(&self, conn: &DbConn, editor_id: FatcatId) -> Result<Editor> {
        let row: EditorRow = Editor::db_get(conn, editor_id)?;
        Ok(row.into_model())
    }

    pub fn get_changelog_handler(
        &self,
        conn: &DbConn,
        limit: Option<i64>,
    ) -> Result<Vec<ChangelogEntry>> {
        let limit = limit.unwrap_or(50);

        let changes: Vec<(EditgroupRow, ChangelogRow, EditorRow)> = editgroup::table
            .inner_join(changelog::table)
            .inner_join(editor::table)
            .order(changelog::id.desc())
            .limit(limit)
            .load(conn)?;

        let entries = changes
            .into_iter()
            .map(|(eg_row, cl_row, editor_row)| ChangelogEntry {
                index: cl_row.id,
                editgroup: Some(
                    eg_row.into_model_partial(Some(cl_row.id), Some(editor_row.into_model())),
                ),
                editgroup_id: uuid2fcid(&cl_row.editgroup_id),
                timestamp: chrono::DateTime::from_utc(cl_row.timestamp, chrono::Utc),
            })
            .collect();
        Ok(entries)
    }

    pub fn get_changelog_entry_handler(&self, conn: &DbConn, index: i64) -> Result<ChangelogEntry> {
        let cl_row: ChangelogRow = changelog::table.find(index).first(conn)?;
        let editgroup =
            self.get_editgroup_handler(conn, FatcatId::from_uuid(&cl_row.editgroup_id))?;

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
    pub fn auth_oidc_handler(&self, conn: &DbConn, params: AuthOidc) -> Result<(Editor, bool)> {
        let existing: Vec<(EditorRow, AuthOidcRow)> = editor::table
            .inner_join(auth_oidc::table)
            .filter(auth_oidc::oidc_sub.eq(params.sub.clone()))
            .filter(auth_oidc::oidc_iss.eq(params.iss.clone()))
            .load(conn)?;

        let (editor_row, created): (EditorRow, bool) = match existing.first() {
            Some((editor, _)) => (editor.clone(), false),
            None => {
                let mut username = format!("{}-{}", params.preferred_username, params.provider);
                username.truncate(24);
                let editor = Editor {
                    editor_id: None,
                    username,
                    is_admin: Some(false),
                    is_bot: Some(false),
                    is_active: Some(true),
                };
                let row = editor.db_create(conn)?;
                // create an auth login row so the user can log back in
                diesel::insert_into(auth_oidc::table)
                    .values((
                        auth_oidc::editor_id.eq(row.id),
                        auth_oidc::provider.eq(params.provider),
                        auth_oidc::oidc_iss.eq(params.iss),
                        auth_oidc::oidc_sub.eq(params.sub),
                    ))
                    .execute(conn)?;
                (row, true)
            }
        };

        Ok((editor_row.into_model(), created))
    }

    entity_auto_batch_handler!(create_container_auto_batch_handler, ContainerEntity);
    entity_auto_batch_handler!(create_creator_auto_batch_handler, CreatorEntity);
    entity_auto_batch_handler!(create_file_auto_batch_handler, FileEntity);
    entity_auto_batch_handler!(create_fileset_auto_batch_handler, FilesetEntity);
    entity_auto_batch_handler!(create_webcapture_auto_batch_handler, WebcaptureEntity);
    entity_auto_batch_handler!(create_release_auto_batch_handler, ReleaseEntity);
    entity_auto_batch_handler!(create_work_auto_batch_handler, WorkEntity);
}
