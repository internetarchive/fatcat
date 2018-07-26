//! API endpoint handlers

use api_helpers::*;
use chrono;
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
use uuid::Uuid;
use ConnectionPool;

type DbConn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

macro_rules! entity_batch_handler {
    ($post_handler:ident, $post_batch_handler:ident, $model:ident) => {
        pub fn $post_batch_handler(&self, entity_list: &[models::$model]) ->
                Result<Vec<EntityEdit>> {
            let conn = self.db_pool.get().expect("db_pool error");
            // TODO: start a transaction
            let mut ret: Vec<EntityEdit> = vec![];
            for entity in entity_list {
                ret.push(self.$post_handler(entity.clone(), Some(&conn))?);
            }
            Ok(ret)
        }
    }
}

macro_rules! entity_history_handler {
    ($history_handler:ident, $edit_row_type:ident, $edit_table:ident) => {
        pub fn $history_handler(
            &self,
            id: &str,
            limit: Option<i64>,
        ) -> Result<Vec<EntityHistoryEntry>> {
            let conn = self.db_pool.get().expect("db_pool error");
            let id = fcid2uuid(id)?;
            let limit = limit.unwrap_or(50);

            let rows: Vec<(EditgroupRow, ChangelogRow, $edit_row_type)> = editgroup::table
                .inner_join(changelog::table)
                .inner_join($edit_table::table)
                .filter($edit_table::ident_id.eq(id))
                .order(changelog::id.desc())
                .limit(limit)
                .get_results(&conn)?;

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

fn container_row2entity(
    ident: Option<ContainerIdentRow>,
    rev: ContainerRevRow,
) -> Result<ContainerEntity> {
    let (state, ident_id, redirect_id) = match ident {
        Some(i) => (
            Some(i.state().unwrap().shortname()),
            Some(uuid2fcid(&i.id)),
            i.redirect_id.map(|u| uuid2fcid(&u)),
        ),
        None => (None, None, None),
    };
    Ok(ContainerEntity {
        issnl: rev.issnl,
        wikidata_qid: rev.wikidata_qid,
        publisher: rev.publisher,
        name: rev.name,
        abbrev: rev.abbrev,
        coden: rev.coden,
        state: state,
        ident: ident_id,
        revision: Some(rev.id.to_string()),
        redirect: redirect_id,
        extra: rev.extra_json,
        editgroup_id: None,
    })
}

fn creator_row2entity(ident: Option<CreatorIdentRow>, rev: CreatorRevRow) -> Result<CreatorEntity> {
    let (state, ident_id, redirect_id) = match ident {
        Some(i) => (
            Some(i.state().unwrap().shortname()),
            Some(uuid2fcid(&i.id)),
            i.redirect_id.map(|u| uuid2fcid(&u)),
        ),
        None => (None, None, None),
    };
    Ok(CreatorEntity {
        display_name: rev.display_name,
        given_name: rev.given_name,
        surname: rev.surname,
        orcid: rev.orcid,
        wikidata_qid: rev.wikidata_qid,
        state: state,
        ident: ident_id,
        revision: Some(rev.id.to_string()),
        redirect: redirect_id,
        editgroup_id: None,
        extra: rev.extra_json,
    })
}

fn file_row2entity(
    ident: Option<FileIdentRow>,
    rev: FileRevRow,
    conn: &DbConn,
) -> Result<FileEntity> {
    let (state, ident_id, redirect_id) = match ident {
        Some(i) => (
            Some(i.state().unwrap().shortname()),
            Some(uuid2fcid(&i.id)),
            i.redirect_id.map(|u| uuid2fcid(&u)),
        ),
        None => (None, None, None),
    };

    let releases: Vec<String> = file_release::table
        .filter(file_release::file_rev.eq(rev.id))
        .get_results(conn)?
        .into_iter()
        .map(|r: FileReleaseRow| uuid2fcid(&r.target_release_ident_id))
        .collect();

    let urls: Vec<FileEntityUrls> = file_rev_url::table
        .filter(file_rev_url::file_rev.eq(rev.id))
        .get_results(conn)?
        .into_iter()
        .map(|r: FileRevUrlRow| FileEntityUrls {
            rel: r.rel,
            url: r.url,
        })
        .collect();

    Ok(FileEntity {
        sha1: rev.sha1,
        sha256: rev.sha256,
        md5: rev.md5,
        size: rev.size.map(|v| v as i64),
        urls: Some(urls),
        mimetype: rev.mimetype,
        releases: Some(releases),
        state: state,
        ident: ident_id,
        revision: Some(rev.id.to_string()),
        redirect: redirect_id,
        editgroup_id: None,
        extra: rev.extra_json,
    })
}

fn release_row2entity(
    ident: Option<ReleaseIdentRow>,
    rev: ReleaseRevRow,
    conn: &DbConn,
) -> Result<ReleaseEntity> {
    let (state, ident_id, redirect_id) = match ident {
        Some(i) => (
            Some(i.state().unwrap().shortname()),
            Some(uuid2fcid(&i.id)),
            i.redirect_id.map(|u| uuid2fcid(&u)),
        ),
        None => (None, None, None),
    };

    let refs: Vec<ReleaseRef> = release_ref::table
        .filter(release_ref::release_rev.eq(rev.id))
        .order(release_ref::index.asc())
        .get_results(conn)
        .expect("fetch release refs")
        .into_iter()
        .map(|r: ReleaseRefRow| ReleaseRef {
            index: r.index,
            key: r.key,
            extra: r.extra_json,
            container_title: r.container_title,
            year: r.year,
            title: r.title,
            locator: r.locator,
            target_release_id: r.target_release_ident_id.map(|v| uuid2fcid(&v)),
        })
        .collect();

    let contribs: Vec<ReleaseContrib> = release_contrib::table
        .filter(release_contrib::release_rev.eq(rev.id))
        .order((release_contrib::role.asc(), release_contrib::index.asc()))
        .get_results(conn)
        .expect("fetch release refs")
        .into_iter()
        .map(|c: ReleaseContribRow| ReleaseContrib {
            index: c.index,
            raw_name: c.raw_name,
            role: c.role,
            extra: c.extra_json,
            creator_id: c.creator_ident_id.map(|v| uuid2fcid(&v)),
        })
        .collect();

    let abstracts: Vec<ReleaseEntityAbstracts> = release_rev_abstract::table
        .inner_join(abstracts::table)
        .filter(release_rev_abstract::release_rev.eq(rev.id))
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
        title: rev.title,
        release_type: rev.release_type,
        release_status: rev.release_status,
        release_date: rev.release_date
            .map(|v| chrono::DateTime::from_utc(v.and_hms(0, 0, 0), chrono::Utc)),
        doi: rev.doi,
        pmid: rev.pmid,
        pmcid: rev.pmcid,
        isbn13: rev.isbn13,
        wikidata_qid: rev.wikidata_qid,
        volume: rev.volume,
        issue: rev.issue,
        pages: rev.pages,
        container_id: rev.container_ident_id.map(|u| uuid2fcid(&u)),
        publisher: rev.publisher,
        language: rev.language,
        work_id: Some(uuid2fcid(&rev.work_ident_id)),
        refs: Some(refs),
        contribs: Some(contribs),
        abstracts: Some(abstracts),
        state: state,
        ident: ident_id,
        revision: Some(rev.id.to_string()),
        redirect: redirect_id,
        editgroup_id: None,
        extra: rev.extra_json,
    })
}

fn work_row2entity(ident: Option<WorkIdentRow>, rev: WorkRevRow) -> Result<WorkEntity> {
    let (state, ident_id, redirect_id) = match ident {
        Some(i) => (
            Some(i.state().unwrap().shortname()),
            Some(uuid2fcid(&i.id)),
            i.redirect_id.map(|u| uuid2fcid(&u)),
        ),
        None => (None, None, None),
    };
    Ok(WorkEntity {
        state: state,
        ident: ident_id,
        revision: Some(rev.id.to_string()),
        redirect: redirect_id,
        editgroup_id: None,
        extra: rev.extra_json,
    })
}

impl Server {
    pub fn get_container_handler(&self, id: &str) -> Result<ContainerEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = fcid2uuid(id)?;

        // TODO: handle Deletions
        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = container_ident::table
            .find(id)
            .inner_join(container_rev::table)
            .first(&conn)?;

        container_row2entity(Some(ident), rev)
    }

    pub fn lookup_container_handler(&self, issnl: &str) -> Result<ContainerEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        check_issn(issnl)?;
        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = container_ident::table
            .inner_join(container_rev::table)
            .filter(container_rev::issnl.eq(issnl))
            .filter(container_ident::is_live.eq(true))
            .filter(container_ident::redirect_id.is_null())
            .first(&conn)?;

        container_row2entity(Some(ident), rev)
    }

    pub fn get_creator_handler(&self, id: &str) -> Result<CreatorEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = fcid2uuid(id)?;

        let (ident, rev): (CreatorIdentRow, CreatorRevRow) = creator_ident::table
            .find(id)
            .inner_join(creator_rev::table)
            .first(&conn)?;

        creator_row2entity(Some(ident), rev)
    }

    pub fn lookup_creator_handler(&self, orcid: &str) -> Result<CreatorEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        check_orcid(orcid)?;
        let (ident, rev): (CreatorIdentRow, CreatorRevRow) = creator_ident::table
            .inner_join(creator_rev::table)
            .filter(creator_rev::orcid.eq(orcid))
            .filter(creator_ident::is_live.eq(true))
            .filter(creator_ident::redirect_id.is_null())
            .first(&conn)?;

        creator_row2entity(Some(ident), rev)
    }

    pub fn get_creator_releases_handler(&self, id: &str) -> Result<Vec<ReleaseEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = fcid2uuid(id)?;

        // TODO: some kind of unique or group-by?
        let rows: Vec<(ReleaseRevRow, ReleaseIdentRow, ReleaseContribRow)> = release_rev::table
            .inner_join(release_ident::table)
            .inner_join(release_contrib::table)
            .filter(release_contrib::creator_ident_id.eq(&id))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .load(&conn)?;

        rows.into_iter()
            .map(|(rev, ident, _)| release_row2entity(Some(ident), rev, &conn))
            .collect()
    }

    pub fn get_file_handler(&self, id: &str) -> Result<FileEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = fcid2uuid(id)?;

        let (ident, rev): (FileIdentRow, FileRevRow) = file_ident::table
            .find(id)
            .inner_join(file_rev::table)
            .first(&conn)?;

        file_row2entity(Some(ident), rev, &conn)
    }

    pub fn lookup_file_handler(&self, sha1: &str) -> Result<FileEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        let (ident, rev): (FileIdentRow, FileRevRow) = file_ident::table
            .inner_join(file_rev::table)
            .filter(file_rev::sha1.eq(sha1))
            .filter(file_ident::is_live.eq(true))
            .filter(file_ident::redirect_id.is_null())
            .first(&conn)?;

        file_row2entity(Some(ident), rev, &conn)
    }

    pub fn get_release_handler(&self, id: &str) -> Result<ReleaseEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = fcid2uuid(id)?;

        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) = release_ident::table
            .find(id)
            .inner_join(release_rev::table)
            .first(&conn)?;

        release_row2entity(Some(ident), rev, &conn)
    }

    pub fn lookup_release_handler(&self, doi: &str) -> Result<ReleaseEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        check_doi(doi)?;
        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) = release_ident::table
            .inner_join(release_rev::table)
            .filter(release_rev::doi.eq(doi))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .first(&conn)?;

        release_row2entity(Some(ident), rev, &conn)
    }

    pub fn get_release_files_handler(&self, id: &str) -> Result<Vec<FileEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = fcid2uuid(id)?;

        let rows: Vec<(FileRevRow, FileIdentRow, FileReleaseRow)> = file_rev::table
            .inner_join(file_ident::table)
            .inner_join(file_release::table)
            .filter(file_release::target_release_ident_id.eq(&id))
            .filter(file_ident::is_live.eq(true))
            .filter(file_ident::redirect_id.is_null())
            .load(&conn)?;

        rows.into_iter()
            .map(|(rev, ident, _)| file_row2entity(Some(ident), rev, &conn))
            .collect()
    }

    pub fn get_work_handler(&self, id: &str) -> Result<WorkEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = fcid2uuid(id)?;

        let (ident, rev): (WorkIdentRow, WorkRevRow) = work_ident::table
            .find(id)
            .inner_join(work_rev::table)
            .first(&conn)?;

        work_row2entity(Some(ident), rev)
    }

    pub fn get_work_releases_handler(&self, id: &str) -> Result<Vec<ReleaseEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = fcid2uuid(id)?;

        let rows: Vec<(ReleaseRevRow, ReleaseIdentRow)> = release_rev::table
            .inner_join(release_ident::table)
            .filter(release_rev::work_ident_id.eq(&id))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .load(&conn)?;

        rows.into_iter()
            .map(|(rev, ident)| release_row2entity(Some(ident), rev, &conn))
            .collect()
    }

    pub fn create_container_handler(
        &self,
        entity: models::ContainerEntity,
        conn: Option<&DbConn>,
    ) -> Result<EntityEdit> {
        // TODO: still can't cast for some reason
        // There mut be a cleaner way to manage the lifetime here
        let real_conn = match conn {
            Some(_) => None,
            None => Some(self.db_pool.get().expect("database pool")),
        };
        let conn = match real_conn {
            Some(ref c) => c,
            None => conn.unwrap(),
        };
        let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?; // TODO: auth
        let editgroup_id: Uuid = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn)?,
            Some(param) => fcid2uuid(&param)?,
        };
        if let Some(ref extid) = entity.wikidata_qid {
            check_wikidata_qid(extid)?;
        }
        if let Some(ref extid) = entity.issnl {
            check_issn(extid)?;
        }

        let edit: ContainerEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO container_rev (name, publisher, issnl, wikidata_qid, abbrev, coden, extra_json)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        RETURNING id ),
                ident AS ( INSERT INTO container_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO container_edit (editgroup_id, ident_id, rev_id) VALUES
                ($8, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(entity.name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.publisher)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.issnl)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.wikidata_qid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.abbrev)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.coden)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
            .bind::<diesel::sql_types::Uuid, _>(editgroup_id)
            .get_result(conn)?;

        edit.into_model()
    }

    pub fn create_creator_handler(
        &self,
        entity: models::CreatorEntity,
        conn: Option<&DbConn>,
    ) -> Result<EntityEdit> {
        // There mut be a cleaner way to manage the lifetime here
        let real_conn = match conn {
            Some(_) => None,
            None => Some(self.db_pool.get().expect("database pool")),
        };
        let conn = match real_conn {
            Some(ref c) => c,
            None => conn.unwrap(),
        };
        let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?; // TODO: auth
        let editgroup_id = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => fcid2uuid(&param)?,
        };
        if let Some(ref extid) = entity.orcid {
            check_orcid(extid)?;
        }
        if let Some(ref extid) = entity.wikidata_qid {
            check_wikidata_qid(extid)?;
        }

        let edit: CreatorEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO creator_rev (display_name, given_name, surname, orcid, wikidata_qid, extra_json)
                        VALUES ($1, $2, $3, $4, $5, $6)
                        RETURNING id ),
                ident AS ( INSERT INTO creator_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO creator_edit (editgroup_id, ident_id, rev_id) VALUES
                ($7, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(entity.display_name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.given_name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.surname)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.orcid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.wikidata_qid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
            .bind::<diesel::sql_types::Uuid, _>(editgroup_id)
            .get_result(conn)?;

        edit.into_model()
    }

    pub fn create_file_handler(
        &self,
        entity: models::FileEntity,
        conn: Option<&DbConn>,
    ) -> Result<EntityEdit> {
        // There mut be a cleaner way to manage the lifetime here
        let real_conn = match conn {
            Some(_) => None,
            None => Some(self.db_pool.get().expect("database pool")),
        };
        let conn = match real_conn {
            Some(ref c) => c,
            None => conn.unwrap(),
        };
        let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?; // TODO: auth
        let editgroup_id = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => fcid2uuid(&param)?,
        };

        let edit: FileEditRow =
            diesel::sql_query(
                "WITH rev AS ( INSERT INTO file_rev (size, sha1, sha256, md5, mimetype, extra_json)
                        VALUES ($1, $2, $3, $4, $5, $6)
                        RETURNING id ),
                ident AS ( INSERT INTO file_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO file_edit (editgroup_id, ident_id, rev_id) VALUES
                ($7, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
            ).bind::<diesel::sql_types::Nullable<diesel::sql_types::Int8>, _>(entity.size)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.sha1)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.sha256)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.md5)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.mimetype)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
                .bind::<diesel::sql_types::Uuid, _>(editgroup_id)
                .get_result(conn)?;

        let _releases: Option<Vec<FileReleaseRow>> = match entity.releases {
            None => None,
            Some(release_list) => {
                if release_list.is_empty() {
                    Some(vec![])
                } else {
                    let release_rows: Vec<FileReleaseRow> = release_list
                        .iter()
                        .map(|r| FileReleaseRow {
                            file_rev: edit.rev_id.unwrap(),
                            target_release_ident_id: fcid2uuid(r)
                                .expect("invalid fatcat identifier"),
                        })
                        .collect();
                    let release_rows: Vec<FileReleaseRow> = insert_into(file_release::table)
                        .values(release_rows)
                        .get_results(conn)
                        .expect("error inserting file_releases");
                    Some(release_rows)
                }
            }
        };

        let _urls: Option<Vec<FileRevUrlRow>> = match entity.urls {
            None => None,
            Some(url_list) => {
                if url_list.is_empty() {
                    Some(vec![])
                } else {
                    let url_rows: Vec<FileRevUrlNewRow> = url_list
                        .into_iter()
                        .map(|u| FileRevUrlNewRow {
                            file_rev: edit.rev_id.unwrap(),
                            rel: u.rel,
                            url: u.url,
                        })
                        .collect();
                    let url_rows: Vec<FileRevUrlRow> = insert_into(file_rev_url::table)
                        .values(url_rows)
                        .get_results(conn)
                        .expect("error inserting file_rev_url");
                    Some(url_rows)
                }
            }
        };

        edit.into_model()
    }

    pub fn create_release_handler(
        &self,
        entity: models::ReleaseEntity,
        conn: Option<&DbConn>,
    ) -> Result<EntityEdit> {
        // There mut be a cleaner way to manage the lifetime here
        let real_conn = match conn {
            Some(_) => None,
            None => Some(self.db_pool.get().expect("database pool")),
        };
        let conn = match real_conn {
            Some(ref c) => c,
            None => conn.unwrap(),
        };
        let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?; // TODO: auth
        let editgroup_id = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => fcid2uuid(&param)?,
        };
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

        let work_id = match entity.work_id {
            Some(work_id) => fcid2uuid(&work_id)?,
            None => {
                // If a work_id wasn't passed, create a new work under the current editgroup
                let work_model = models::WorkEntity {
                    ident: None,
                    revision: None,
                    redirect: None,
                    state: None,
                    editgroup_id: Some(uuid2fcid(&editgroup_id)),
                    extra: None,
                };
                let new_entity = self.create_work_handler(work_model, Some(&conn))?;
                fcid2uuid(&new_entity.ident)?
            }
        };

        let container_id: Option<Uuid> = match entity.container_id {
            Some(id) => Some(fcid2uuid(&id)?),
            None => None,
        };

        let edit: ReleaseEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO release_rev (title, release_type, release_status, release_date, doi, pmid, pmcid, wikidata_qid, isbn13, volume, issue, pages, work_ident_id, container_ident_id, publisher, language, extra_json)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
                        RETURNING id ),
                ident AS ( INSERT INTO release_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO release_edit (editgroup_id, ident_id, rev_id) VALUES
                ($18, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(entity.title)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.release_type)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.release_status)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Date>, _>(
                entity.release_date.map(|v| v.naive_utc().date()))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.doi)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.pmid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.pmcid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.wikidata_qid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.isbn13)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.volume)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.issue)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.pages)
            .bind::<diesel::sql_types::Uuid, _>(work_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(container_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.publisher)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.language)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
            .bind::<diesel::sql_types::Uuid, _>(editgroup_id)
            .get_result(conn)?;

        let _refs: Option<Vec<ReleaseRefRow>> = match entity.refs {
            None => None,
            Some(ref_list) => {
                if ref_list.is_empty() {
                    Some(vec![])
                } else {
                    let ref_rows: Vec<ReleaseRefNewRow> = ref_list
                        .iter()
                        .map(|r| ReleaseRefNewRow {
                            release_rev: edit.rev_id.unwrap(),
                            target_release_ident_id: r.target_release_id
                                .clone()
                                .map(|v| fcid2uuid(&v).expect("valid fatcat identifier")),
                            index: r.index,
                            key: r.key.clone(),
                            container_title: r.container_title.clone(),
                            year: r.year,
                            title: r.title.clone(),
                            locator: r.locator.clone(),
                            extra_json: r.extra.clone(),
                        })
                        .collect();
                    let ref_rows: Vec<ReleaseRefRow> = insert_into(release_ref::table)
                        .values(ref_rows)
                        .get_results(conn)
                        .expect("error inserting release_refs");
                    Some(ref_rows)
                }
            }
        };

        let _contribs: Option<Vec<ReleaseContribRow>> = match entity.contribs {
            None => None,
            Some(contrib_list) => {
                if contrib_list.is_empty() {
                    Some(vec![])
                } else {
                    println!("{:#?}", contrib_list);
                    let contrib_rows: Vec<ReleaseContribNewRow> = contrib_list
                        .iter()
                        .map(|c| ReleaseContribNewRow {
                            release_rev: edit.rev_id.unwrap(),
                            creator_ident_id: c.creator_id
                                .clone()
                                .map(|v| fcid2uuid(&v).expect("valid fatcat identifier")),
                            raw_name: c.raw_name.clone(),
                            index: c.index,
                            role: c.role.clone(),
                            extra_json: c.extra.clone(),
                        })
                        .collect();
                    let contrib_rows: Vec<ReleaseContribRow> = insert_into(release_contrib::table)
                        .values(contrib_rows)
                        .get_results(conn)
                        .expect("error inserting release_contribs");
                    Some(contrib_rows)
                }
            }
        };

        if let Some(abstract_list) = entity.abstracts {
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
            if !new_abstracts.is_empty() {
                // Sort of an "upsert"; only inserts new abstract rows if they don't already exist
                insert_into(abstracts::table)
                    .values(&new_abstracts)
                    .on_conflict(abstracts::sha1)
                    .do_nothing()
                    .execute(conn)?;
            }
            let release_abstract_rows: Vec<ReleaseRevAbstractNewRow> = abstract_list
                .into_iter()
                .map(|c| ReleaseRevAbstractNewRow {
                    release_rev: edit.rev_id.unwrap(),
                    abstract_sha1: match c.content {
                        Some(ref content) => Sha1::from(content).hexdigest(),
                        None => c.sha1.expect("either abstract_sha1 or content is required"),
                    },
                    lang: c.lang,
                    mimetype: c.mimetype,
                })
                .collect();
            insert_into(release_rev_abstract::table)
                .values(release_abstract_rows)
                .execute(conn)?;
        }

        edit.into_model()
    }

    pub fn create_work_handler(
        &self,
        entity: models::WorkEntity,
        conn: Option<&DbConn>,
    ) -> Result<EntityEdit> {
        // There mut be a cleaner way to manage the lifetime here
        let real_conn = match conn {
            Some(_) => None,
            None => Some(self.db_pool.get().expect("database pool")),
        };
        let conn = match real_conn {
            Some(ref c) => c,
            None => conn.unwrap(),
        };

        let editor_id = Uuid::parse_str("00000000-0000-0000-AAAA-000000000001")?; // TODO: auth
        let editgroup_id = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => fcid2uuid(&param)?,
        };

        let edit: WorkEditRow =
            diesel::sql_query(
                "WITH rev AS ( INSERT INTO work_rev (extra_json)
                        VALUES ($1)
                        RETURNING id ),
                ident AS ( INSERT INTO work_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO work_edit (editgroup_id, ident_id, rev_id) VALUES
                ($2, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
            ).bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
                .bind::<diesel::sql_types::Uuid, _>(editgroup_id)
                .get_result(conn)?;

        edit.into_model()
    }

    pub fn accept_editgroup_handler(&self, id: &str) -> Result<()> {
        let conn = self.db_pool.get().expect("db_pool error");
        accept_editgroup(fcid2uuid(id)?, &conn)?;
        Ok(())
    }

    pub fn create_editgroup_handler(&self, entity: models::Editgroup) -> Result<Editgroup> {
        let conn = self.db_pool.get().expect("db_pool error");

        let row: EditgroupRow = insert_into(editgroup::table)
            .values((
                editgroup::editor_id.eq(fcid2uuid(&entity.editor_id)?),
                editgroup::description.eq(entity.description),
                editgroup::extra_json.eq(entity.extra),
            ))
            .get_result(&conn)
            .expect("error creating edit group");

        Ok(Editgroup {
            id: Some(uuid2fcid(&row.id)),
            editor_id: uuid2fcid(&row.editor_id),
            description: row.description,
            edits: None,
            extra: row.extra_json,
        })
    }

    pub fn get_editgroup_handler(&self, id: &str) -> Result<Editgroup> {
        let conn = self.db_pool.get().expect("db_pool error");

        let id = fcid2uuid(id)?;
        let row: EditgroupRow = editgroup::table.find(id).first(&conn)?;

        let edits = EditgroupEdits {
            containers: Some(
                container_edit::table
                    .filter(container_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .into_iter()
                    .map(|e: ContainerEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            creators: Some(
                creator_edit::table
                    .filter(creator_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .into_iter()
                    .map(|e: CreatorEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            files: Some(
                file_edit::table
                    .filter(file_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .into_iter()
                    .map(|e: FileEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            releases: Some(
                release_edit::table
                    .filter(release_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .into_iter()
                    .map(|e: ReleaseEditRow| e.into_model().unwrap())
                    .collect(),
            ),
            works: Some(
                work_edit::table
                    .filter(work_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
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

    pub fn get_editor_handler(&self, id: &str) -> Result<Editor> {
        let conn = self.db_pool.get().expect("db_pool error");

        let id = fcid2uuid(id)?;
        let row: EditorRow = editor::table.find(id).first(&conn)?;

        let ed = Editor {
            id: Some(uuid2fcid(&row.id)),
            username: row.username,
        };
        Ok(ed)
    }

    pub fn editor_changelog_get_handler(&self, id: &str) -> Result<Vec<ChangelogEntry>> {
        let conn = self.db_pool.get().expect("db_pool error");

        let id = fcid2uuid(id)?;
        // TODO: single query
        let editor: EditorRow = editor::table.find(id).first(&conn)?;
        let changes: Vec<(ChangelogRow, EditgroupRow)> = changelog::table
            .inner_join(editgroup::table)
            .filter(editgroup::editor_id.eq(editor.id))
            .load(&conn)?;

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

    pub fn get_changelog_handler(&self, limit: Option<i64>) -> Result<Vec<ChangelogEntry>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let limit = limit.unwrap_or(50);

        let changes: Vec<(ChangelogRow, EditgroupRow)> = changelog::table
            .inner_join(editgroup::table)
            .order(changelog::id.desc())
            .limit(limit)
            .load(&conn)?;

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

    pub fn get_changelog_entry_handler(&self, id: i64) -> Result<ChangelogEntry> {
        let conn = self.db_pool.get().expect("db_pool error");

        let cl_row: ChangelogRow = changelog::table.find(id).first(&conn)?;
        let editgroup = self.get_editgroup_handler(&uuid2fcid(&cl_row.editgroup_id))?;

        let mut entry = cl_row.into_model();
        entry.editgroup = Some(editgroup);
        Ok(entry)
    }

    pub fn get_stats_handler(&self, more: &Option<String>) -> Result<StatsResponse> {
        let conn = self.db_pool.get().expect("db_pool error");

        let merged_editgroups: i64 = changelog::table
            .select(diesel::dsl::count_star())
            .first(&conn)?;
        let releases_with_dois: i64 = release_rev::table
            .inner_join(release_ident::table)
            .filter(release_rev::doi.is_not_null())
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .select(diesel::dsl::count_star())
            .first(&conn)?;
        let creators_with_orcids: i64 = creator_rev::table
            .inner_join(creator_ident::table)
            .filter(creator_rev::orcid.is_not_null())
            .filter(creator_ident::is_live.eq(true))
            .filter(creator_ident::redirect_id.is_null())
            .select(diesel::dsl::count_star())
            .first(&conn)?;
        let containers_with_issnls: i64 = container_rev::table
            .inner_join(container_ident::table)
            .filter(container_rev::issnl.is_not_null())
            .filter(container_ident::is_live.eq(true))
            .filter(container_ident::redirect_id.is_null())
            .count()
            .first(&conn)?;

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
                .first(&conn)?)
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
                .first(&conn)?)
        } else {
            None
        };

        let val = json!({
            "entity_counts": {
                "container": count_entity!(container_ident, &conn),
                "creator": count_entity!(creator_ident, &conn),
                "file": count_entity!(file_ident, &conn),
                "release": count_entity!(release_ident, &conn),
                "work": count_entity!(work_ident, &conn),
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

    entity_batch_handler!(
        create_container_handler,
        create_container_batch_handler,
        ContainerEntity
    );
    entity_batch_handler!(
        create_creator_handler,
        create_creator_batch_handler,
        CreatorEntity
    );
    entity_batch_handler!(create_file_handler, create_file_batch_handler, FileEntity);
    entity_batch_handler!(
        create_release_handler,
        create_release_batch_handler,
        ReleaseEntity
    );
    entity_batch_handler!(create_work_handler, create_work_batch_handler, WorkEntity);

    entity_history_handler!(
        get_container_history_handler,
        ContainerEditRow,
        container_edit
    );
    entity_history_handler!(get_creator_history_handler, CreatorEditRow, creator_edit);
    entity_history_handler!(get_file_history_handler, FileEditRow, file_edit);
    entity_history_handler!(get_release_history_handler, ReleaseEditRow, release_edit);
    entity_history_handler!(get_work_history_handler, WorkEditRow, work_edit);
}
