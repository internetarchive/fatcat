//! API endpoint handlers

use ConnectionPool;
use api_helpers::*;
use chrono;
use database_models::*;
use database_schema::{changelog, container_edit, container_ident, container_rev, creator_edit,
                      creator_ident, creator_rev, editgroup, editor, file_edit, file_ident,
                      file_release, file_rev, release_contrib, release_edit, release_ident,
                      release_ref, release_rev, work_edit, work_ident, work_rev};
use diesel::prelude::*;
use diesel::{self, insert_into};
use errors::*;
use fatcat_api::models;
use fatcat_api::models::*;
use fatcat_api::*;
use futures::{self, Future};
use uuid;

type DbConn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

/// Helper for generating wrappers (which return "Box::new(futures::done(Ok(BLAH)))" like the
/// codegen fatcat-api code wants) that call through to actual helpers (which have simple Result<>
/// return types)
macro_rules! wrap_entity_handlers {
    // Would much rather just have entity ident, then generate the other fields from that, but Rust
    // stable doesn't have a mechanism to "concat" or generate new identifiers in macros, at least
    // in the context of defining new functions.
    // The only stable approach I know of would be: https://github.com/dtolnay/mashup
    ($get_fn:ident, $get_handler:ident, $get_resp:ident, $post_fn:ident, $post_handler:ident,
            $post_resp:ident, $post_batch_fn:ident, $post_batch_handler:ident,
            $post_batch_resp:ident, $get_history_fn:ident, $get_history_handler:ident,
            $get_history_resp:ident, $model:ident) => {

        fn $get_fn(
            &self,
            id: String,
            _context: &Context,
        ) -> Box<Future<Item = $get_resp, Error = ApiError> + Send> {
            let ret = match self.$get_handler(id.clone()) {
                Ok(entity) =>
                    $get_resp::FoundEntity(entity),
                Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) =>
                    $get_resp::NotFound(ErrorResponse { message: format!("No such entity {}: {}", stringify!($model), id) }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $get_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(e) => {
                    error!("{}", e);
                    $get_resp::GenericError(ErrorResponse { message: e.to_string() })
                },
            };
            Box::new(futures::done(Ok(ret)))
        }

        fn $post_fn(
            &self,
            entity: models::$model,
            _context: &Context,
        ) -> Box<Future<Item = $post_resp, Error = ApiError> + Send> {
            let ret = match self.$post_handler(entity, None) {
                Ok(edit) =>
                    $post_resp::CreatedEntity(edit),
                Err(Error(ErrorKind::Diesel(e), _)) =>
                    $post_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $post_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(e) => {
                    error!("{}", e);
                    $post_resp::GenericError(ErrorResponse { message: e.to_string() })
                },
            };
            Box::new(futures::done(Ok(ret)))
        }

        fn $post_batch_fn(
            &self,
            entity_list: &Vec<models::$model>,
            _context: &Context,
        ) -> Box<Future<Item = $post_batch_resp, Error = ApiError> + Send> {
            let ret = match self.$post_batch_handler(entity_list) {
                Ok(edit) =>
                    $post_batch_resp::CreatedEntities(edit),
                Err(Error(ErrorKind::Diesel(e), _)) =>
                    $post_batch_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $post_batch_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(e) => {
                    error!("{}", e);
                    $post_batch_resp::GenericError(ErrorResponse { message: e.to_string() })
                },
            };
            Box::new(futures::done(Ok(ret)))
        }

        fn $get_history_fn(
            &self,
            id: String,
            limit: Option<i64>,
            _context: &Context,
        ) -> Box<Future<Item = $get_history_resp, Error = ApiError> + Send> {
            let ret = match self.$get_history_handler(id.clone(), limit) {
                Ok(history) =>
                    $get_history_resp::FoundEntityHistory(history),
                Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) =>
                    $get_history_resp::NotFound(ErrorResponse { message: format!("No such entity {}: {}", stringify!($model), id) }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $get_history_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(e) => {
                    error!("{}", e);
                    $get_history_resp::GenericError(ErrorResponse { message: e.to_string() })
                },
            };
            Box::new(futures::done(Ok(ret)))
        }
    }
}

macro_rules! wrap_lookup_handler {
    ($get_fn:ident, $get_handler:ident, $get_resp:ident, $idname:ident, $idtype:ident) => {
        fn $get_fn(
            &self,
            $idname: $idtype,
            _context: &Context,
        ) -> Box<Future<Item = $get_resp, Error = ApiError> + Send> {
            let ret = match self.$get_handler($idname.clone()) {
                Ok(entity) =>
                    $get_resp::FoundEntity(entity),
                Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) =>
                    $get_resp::NotFound(ErrorResponse { message: format!("Not found: {}", $idname) }),
                Err(e) => {
                    error!("{}", e);
                    $get_resp::BadRequest(ErrorResponse { message: e.to_string() })
                },
            };
            Box::new(futures::done(Ok(ret)))
        }
    }
}

macro_rules! entity_batch_handler {
    ($post_handler:ident, $post_batch_handler:ident, $model:ident) => {
        fn $post_batch_handler(&self, entity_list: &Vec<models::$model>) ->
                Result<Vec<EntityEdit>> {
            let conn = self.db_pool.get().expect("db_pool error");
            // TODO: start a transaction
            let mut ret: Vec<EntityEdit> = vec![];
            for entity in entity_list.into_iter() {
                ret.push(self.$post_handler(entity.clone(), Some(&conn))?);
            }
            Ok(ret)
        }
    }
}

macro_rules! entity_history_handler {
    ($history_handler:ident, $edit_row_type:ident, $edit_table:ident) => {
        fn $history_handler(
            &self,
            id: String,
            limit: Option<i64>,
        ) -> Result<Vec<EntityHistoryEntry>> {
            let conn = self.db_pool.get().expect("db_pool error");
            let id = uuid::Uuid::parse_str(&id)?;
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
                    edit: e_row.to_model().expect("edit row to model"),
                    editgroup: eg_row.to_model_partial(),
                    changelog_entry: cl_row.to_model(),
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
            Some(i.id.to_string()),
            i.redirect_id.map(|u| u.to_string()),
        ),
        None => (None, None, None),
    };
    Ok(ContainerEntity {
        issnl: rev.issnl,
        publisher: rev.publisher,
        name: rev.name,
        abbrev: rev.abbrev,
        coden: rev.coden,
        state: state,
        ident: ident_id,
        revision: Some(rev.id),
        redirect: redirect_id,
        extra: rev.extra_json,
        editgroup_id: None,
    })
}

fn creator_row2entity(ident: Option<CreatorIdentRow>, rev: CreatorRevRow) -> Result<CreatorEntity> {
    let (state, ident_id, redirect_id) = match ident {
        Some(i) => (
            Some(i.state().unwrap().shortname()),
            Some(i.id.to_string()),
            i.redirect_id.map(|u| u.to_string()),
        ),
        None => (None, None, None),
    };
    Ok(CreatorEntity {
        display_name: rev.display_name,
        given_name: rev.given_name,
        surname: rev.surname,
        orcid: rev.orcid,
        state: state,
        ident: ident_id,
        revision: Some(rev.id),
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
            Some(i.id.to_string()),
            i.redirect_id.map(|u| u.to_string()),
        ),
        None => (None, None, None),
    };

    let releases: Vec<String> = file_release::table
        .filter(file_release::file_rev.eq(rev.id))
        .get_results(conn)?
        .iter()
        .map(|r: &FileReleaseRow| r.target_release_ident_id.to_string())
        .collect();

    Ok(FileEntity {
        sha1: rev.sha1,
        sha256: rev.sha256,
        md5: rev.md5,
        size: rev.size.map(|v| v as i64),
        url: rev.url,
        mimetype: rev.mimetype,
        releases: Some(releases),
        state: state,
        ident: ident_id,
        revision: Some(rev.id),
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
            Some(i.id.to_string()),
            i.redirect_id.map(|u| u.to_string()),
        ),
        None => (None, None, None),
    };

    let refs: Vec<ReleaseRef> = release_ref::table
        .filter(release_ref::release_rev.eq(rev.id))
        .get_results(conn)
        .expect("fetch release refs")
        .iter()
        .map(|r: &ReleaseRefRow| ReleaseRef {
            index: r.index.clone(),
            key: r.key.clone(),
            raw: r.raw.clone(),
            container_title: r.container_title.clone(),
            year: r.year.clone(),
            title: r.title.clone(),
            locator: r.locator.clone(),
            target_release_id: r.target_release_ident_id.map(|v| v.to_string()),
        })
        .collect();

    let contribs: Vec<ReleaseContrib> = release_contrib::table
        .filter(release_contrib::release_rev.eq(rev.id))
        .get_results(conn)
        .expect("fetch release refs")
        .iter()
        .map(|c: &ReleaseContribRow| ReleaseContrib {
            index: c.index,
            role: c.role.clone(),
            raw: c.raw.clone(),
            creator_id: c.creator_ident_id.map(|v| v.to_string()),
        })
        .collect();

    Ok(ReleaseEntity {
        title: rev.title,
        release_type: rev.release_type,
        release_status: rev.release_status,
        release_date: rev.release_date
            .map(|v| chrono::DateTime::from_utc(v.and_hms(0, 0, 0), chrono::Utc)),
        doi: rev.doi,
        isbn13: rev.isbn13,
        volume: rev.volume,
        issue: rev.issue,
        pages: rev.pages,
        container_id: rev.container_ident_id.map(|u| u.to_string()),
        publisher: rev.publisher,
        language: rev.language,
        work_id: Some(rev.work_ident_id.to_string()),
        refs: Some(refs),
        contribs: Some(contribs),
        state: state,
        ident: ident_id,
        revision: Some(rev.id),
        redirect: redirect_id,
        editgroup_id: None,
        extra: rev.extra_json,
    })
}

fn work_row2entity(ident: Option<WorkIdentRow>, rev: WorkRevRow) -> Result<WorkEntity> {
    let (state, ident_id, redirect_id) = match ident {
        Some(i) => (
            Some(i.state().unwrap().shortname()),
            Some(i.id.to_string()),
            i.redirect_id.map(|u| u.to_string()),
        ),
        None => (None, None, None),
    };
    Ok(WorkEntity {
        work_type: rev.work_type,
        state: state,
        ident: ident_id,
        revision: Some(rev.id),
        redirect: redirect_id,
        editgroup_id: None,
        extra: rev.extra_json,
    })
}

impl Server {
    fn get_container_handler(&self, id: String) -> Result<ContainerEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        // TODO: handle Deletions
        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = container_ident::table
            .find(id)
            .inner_join(container_rev::table)
            .first(&conn)?;

        container_row2entity(Some(ident), rev)
    }

    fn lookup_container_handler(&self, issnl: String) -> Result<ContainerEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = container_ident::table
            .inner_join(container_rev::table)
            .filter(container_rev::issnl.eq(&issnl))
            .filter(container_ident::is_live.eq(true))
            .filter(container_ident::redirect_id.is_null())
            .first(&conn)?;

        container_row2entity(Some(ident), rev)
    }

    fn get_creator_handler(&self, id: String) -> Result<CreatorEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let (ident, rev): (CreatorIdentRow, CreatorRevRow) = creator_ident::table
            .find(id)
            .inner_join(creator_rev::table)
            .first(&conn)?;

        creator_row2entity(Some(ident), rev)
    }

    fn lookup_creator_handler(&self, orcid: String) -> Result<CreatorEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        let (ident, rev): (CreatorIdentRow, CreatorRevRow) = creator_ident::table
            .inner_join(creator_rev::table)
            .filter(creator_rev::orcid.eq(&orcid))
            .filter(creator_ident::is_live.eq(true))
            .filter(creator_ident::redirect_id.is_null())
            .first(&conn)?;

        creator_row2entity(Some(ident), rev)
    }

    fn get_creator_releases_handler(&self, id: String) -> Result<Vec<ReleaseEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

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

    fn get_file_handler(&self, id: String) -> Result<FileEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let (ident, rev): (FileIdentRow, FileRevRow) = file_ident::table
            .find(id)
            .inner_join(file_rev::table)
            .first(&conn)?;

        file_row2entity(Some(ident), rev, &conn)
    }

    fn lookup_file_handler(&self, sha1: String) -> Result<FileEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        let (ident, rev): (FileIdentRow, FileRevRow) = file_ident::table
            .inner_join(file_rev::table)
            .filter(file_rev::sha1.eq(&sha1))
            .filter(file_ident::is_live.eq(true))
            .filter(file_ident::redirect_id.is_null())
            .first(&conn)?;

        file_row2entity(Some(ident), rev, &conn)
    }

    fn get_release_handler(&self, id: String) -> Result<ReleaseEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) = release_ident::table
            .find(id)
            .inner_join(release_rev::table)
            .first(&conn)?;

        release_row2entity(Some(ident), rev, &conn)
    }

    fn lookup_release_handler(&self, doi: String) -> Result<ReleaseEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) = release_ident::table
            .inner_join(release_rev::table)
            .filter(release_rev::doi.eq(&doi))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .first(&conn)?;

        release_row2entity(Some(ident), rev, &conn)
    }

    fn get_release_files_handler(&self, id: String) -> Result<Vec<FileEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

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

    fn get_work_handler(&self, id: String) -> Result<WorkEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let (ident, rev): (WorkIdentRow, WorkRevRow) = work_ident::table
            .find(id)
            .inner_join(work_rev::table)
            .first(&conn)?;

        work_row2entity(Some(ident), rev)
    }

    fn get_work_releases_handler(&self, id: String) -> Result<Vec<ReleaseEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

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

    fn create_container_handler(
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
        let editor_id = 1; // TODO: auth
        let editgroup_id = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn)?,
            Some(param) => param as i64,
        };

        let edit: ContainerEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO container_rev (name, publisher, issnl, abbrev, coden, extra_json)
                        VALUES ($1, $2, $3, $4, $5, $6)
                        RETURNING id ),
                ident AS ( INSERT INTO container_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO container_edit (editgroup_id, ident_id, rev_id) VALUES
                ($7, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(entity.name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.publisher)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.issnl)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.abbrev)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.coden)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .get_result(conn)?;

        edit.to_model()
    }

    fn create_creator_handler(
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
        let editor_id = 1; // TODO: auth
        let editgroup_id = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let edit: CreatorEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO creator_rev (display_name, given_name, surname, orcid, extra_json)
                        VALUES ($1, $2, $3, $4, $5)
                        RETURNING id ),
                ident AS ( INSERT INTO creator_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO creator_edit (editgroup_id, ident_id, rev_id) VALUES
                ($6, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(entity.display_name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.given_name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.surname)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.orcid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .get_result(conn)?;

        edit.to_model()
    }

    fn create_file_handler(
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
        let editor_id = 1; // TODO: auth
        let editgroup_id = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let edit: FileEditRow =
            diesel::sql_query(
                "WITH rev AS ( INSERT INTO file_rev (size, sha1, sha256, md5, url, mimetype, extra_json)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        RETURNING id ),
                ident AS ( INSERT INTO file_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO file_edit (editgroup_id, ident_id, rev_id) VALUES
                ($8, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
            ).bind::<diesel::sql_types::Nullable<diesel::sql_types::Int8>, _>(entity.size)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.sha1)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.sha256)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.md5)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.url)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.mimetype)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
                .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
                .get_result(conn)?;

        let _releases: Option<Vec<FileReleaseRow>> = match entity.releases {
            None => None,
            Some(release_list) => {
                if release_list.len() == 0 {
                    Some(vec![])
                } else {
                    let release_rows: Vec<FileReleaseRow> = release_list
                        .iter()
                        .map(|r| FileReleaseRow {
                            file_rev: edit.rev_id.unwrap(),
                            target_release_ident_id: uuid::Uuid::parse_str(r).expect("valid UUID"),
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

        edit.to_model()
    }

    fn create_release_handler(
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
        let editor_id = 1; // TODO: auth
        let editgroup_id = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let work_id = match entity.work_id {
            Some(work_id) => uuid::Uuid::parse_str(&work_id)?,
            None => {
                // If a work_id wasn't passed, create a new work under the current editgroup
                let work_model = models::WorkEntity {
                    work_type: None,
                    ident: None,
                    revision: None,
                    redirect: None,
                    state: None,
                    editgroup_id: Some(editgroup_id),
                    extra: None,
                };
                let new_entity = self.create_work_handler(work_model, Some(&conn))?;
                uuid::Uuid::parse_str(&new_entity.ident)?
            }
        };

        let container_id: Option<uuid::Uuid> = match entity.container_id {
            Some(id) => Some(uuid::Uuid::parse_str(&id)?),
            None => None,
        };

        let edit: ReleaseEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO release_rev (title, release_type, release_status, release_date, doi, isbn13, volume, issue, pages, work_ident_id, container_ident_id, publisher, language, extra_json)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                        RETURNING id ),
                ident AS ( INSERT INTO release_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO release_edit (editgroup_id, ident_id, rev_id) VALUES
                ($15, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(entity.title)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.release_type)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.release_status)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Date>, _>(
                entity.release_date.map(|v| v.naive_utc().date()))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.doi)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.isbn13)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.volume)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.issue)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.pages)
            .bind::<diesel::sql_types::Uuid, _>(work_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(container_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.publisher)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.language)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .get_result(conn)?;

        let _refs: Option<Vec<ReleaseRefRow>> = match entity.refs {
            None => None,
            Some(ref_list) => {
                if ref_list.len() == 0 {
                    Some(vec![])
                } else {
                    let ref_rows: Vec<ReleaseRefNewRow> = ref_list
                        .iter()
                        .map(|r| ReleaseRefNewRow {
                            release_rev: edit.rev_id.unwrap(),
                            target_release_ident_id: r.target_release_id
                                .clone()
                                .map(|v| uuid::Uuid::parse_str(&v).expect("valid UUID")),
                            index: r.index,
                            key: r.key.clone(),
                            container_title: r.container_title.clone(),
                            year: r.year,
                            title: r.title.clone(),
                            locator: r.locator.clone(),
                            raw: r.raw.clone(),
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
                if contrib_list.len() == 0 {
                    Some(vec![])
                } else {
                    let contrib_rows: Vec<ReleaseContribNewRow> = contrib_list
                        .iter()
                        .map(|c| ReleaseContribNewRow {
                            release_rev: edit.rev_id.unwrap(),
                            creator_ident_id: c.creator_id
                                .clone()
                                .map(|v| uuid::Uuid::parse_str(&v).expect("valid UUID")),
                            index: c.index,
                            role: c.role.clone(),
                            raw: c.raw.clone(),
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

        edit.to_model()
    }

    fn create_work_handler(
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

        let editor_id = 1; // TODO: auth
        let editgroup_id = match entity.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let edit: WorkEditRow =
            diesel::sql_query(
                "WITH rev AS ( INSERT INTO work_rev (work_type, extra_json)
                        VALUES ($1, $2)
                        RETURNING id ),
                ident AS ( INSERT INTO work_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO work_edit (editgroup_id, ident_id, rev_id) VALUES
                ($3, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
            ).bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(entity.work_type)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(entity.extra)
                .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
                .get_result(conn)?;

        edit.to_model()
    }

    fn get_editgroup_handler(&self, id: i64) -> Result<Editgroup> {
        let conn = self.db_pool.get().expect("db_pool error");

        let row: EditgroupRow = editgroup::table.find(id as i64).first(&conn)?;

        let edits = EditgroupEdits {
            containers: Some(
                container_edit::table
                    .filter(container_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .into_iter()
                    .map(|e: ContainerEditRow| e.to_model().unwrap())
                    .collect(),
            ),
            creators: Some(
                creator_edit::table
                    .filter(creator_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .into_iter()
                    .map(|e: CreatorEditRow| e.to_model().unwrap())
                    .collect(),
            ),
            files: Some(
                file_edit::table
                    .filter(file_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .into_iter()
                    .map(|e: FileEditRow| e.to_model().unwrap())
                    .collect(),
            ),
            releases: Some(
                release_edit::table
                    .filter(release_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .into_iter()
                    .map(|e: ReleaseEditRow| e.to_model().unwrap())
                    .collect(),
            ),
            works: Some(
                work_edit::table
                    .filter(work_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .into_iter()
                    .map(|e: WorkEditRow| e.to_model().unwrap())
                    .collect(),
            ),
        };

        let eg = Editgroup {
            id: Some(row.id),
            editor_id: row.editor_id,
            description: row.description,
            edits: Some(edits),
            extra: row.extra_json,
        };
        Ok(eg)
    }

    fn get_editor_handler(&self, username: String) -> Result<Editor> {
        let conn = self.db_pool.get().expect("db_pool error");

        let row: EditorRow = editor::table
            .filter(editor::username.eq(&username))
            .first(&conn)?;

        let ed = Editor {
            username: row.username,
        };
        Ok(ed)
    }

    fn editor_changelog_get_handler(&self, username: String) -> Result<Vec<ChangelogEntry>> {
        let conn = self.db_pool.get().expect("db_pool error");

        // TODO: single query
        let editor: EditorRow = editor::table
            .filter(editor::username.eq(username))
            .first(&conn)?;
        let changes: Vec<(ChangelogRow, EditgroupRow)> = changelog::table
            .inner_join(editgroup::table)
            .filter(editgroup::editor_id.eq(editor.id))
            .load(&conn)?;

        let entries = changes
            .iter()
            .map(|(row, _)| ChangelogEntry {
                index: row.id,
                editgroup_id: row.editgroup_id,
                timestamp: chrono::DateTime::from_utc(row.timestamp, chrono::Utc),
            })
            .collect();
        Ok(entries)
    }

    /// "more" parameter isn't used, but could be to indicate that "expensive" database queries
    /// should be run
    fn get_stats_handler(&self, _more: Option<String>) -> Result<StatsResponse> {
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

        // these are probably expensive?
        // this query is slightly inaccurate and over-counts: it includes files that have release
        // links only to inactive releases
        let files_with_releases: i64 = file_rev::table
            .inner_join(file_ident::table)
            .inner_join(file_release::table)
            .filter(file_ident::is_live.eq(true))
            .filter(file_ident::redirect_id.is_null())
            .select(file_ident::id)
            .distinct()
            .count()
            .first(&conn)?;
        // this slightly overcounts also: it will include releases which are only linked to from
        // inactive files
        let releases_with_files: i64 = release_ident::table
            .inner_join(file_release::table)
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .select(file_release::target_release_ident_id)
            .distinct()
            .count()
            .first(&conn)?;

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

impl Api for Server {
    wrap_entity_handlers!(
        get_container,
        get_container_handler,
        GetContainerResponse,
        create_container,
        create_container_handler,
        CreateContainerResponse,
        create_container_batch,
        create_container_batch_handler,
        CreateContainerBatchResponse,
        get_container_history,
        get_container_history_handler,
        GetContainerHistoryResponse,
        ContainerEntity
    );

    wrap_entity_handlers!(
        get_creator,
        get_creator_handler,
        GetCreatorResponse,
        create_creator,
        create_creator_handler,
        CreateCreatorResponse,
        create_creator_batch,
        create_creator_batch_handler,
        CreateCreatorBatchResponse,
        get_creator_history,
        get_creator_history_handler,
        GetCreatorHistoryResponse,
        CreatorEntity
    );
    wrap_entity_handlers!(
        get_file,
        get_file_handler,
        GetFileResponse,
        create_file,
        create_file_handler,
        CreateFileResponse,
        create_file_batch,
        create_file_batch_handler,
        CreateFileBatchResponse,
        get_file_history,
        get_file_history_handler,
        GetFileHistoryResponse,
        FileEntity
    );
    wrap_entity_handlers!(
        get_release,
        get_release_handler,
        GetReleaseResponse,
        create_release,
        create_release_handler,
        CreateReleaseResponse,
        create_release_batch,
        create_release_batch_handler,
        CreateReleaseBatchResponse,
        get_release_history,
        get_release_history_handler,
        GetReleaseHistoryResponse,
        ReleaseEntity
    );
    wrap_entity_handlers!(
        get_work,
        get_work_handler,
        GetWorkResponse,
        create_work,
        create_work_handler,
        CreateWorkResponse,
        create_work_batch,
        create_work_batch_handler,
        CreateWorkBatchResponse,
        get_work_history,
        get_work_history_handler,
        GetWorkHistoryResponse,
        WorkEntity
    );

    wrap_lookup_handler!(
        lookup_container,
        lookup_container_handler,
        LookupContainerResponse,
        issnl,
        String
    );
    wrap_lookup_handler!(
        lookup_creator,
        lookup_creator_handler,
        LookupCreatorResponse,
        orcid,
        String
    );
    wrap_lookup_handler!(
        lookup_file,
        lookup_file_handler,
        LookupFileResponse,
        sha1,
        String
    );
    wrap_lookup_handler!(
        lookup_release,
        lookup_release_handler,
        LookupReleaseResponse,
        doi,
        String
    );

    wrap_lookup_handler!(
        get_release_files,
        get_release_files_handler,
        GetReleaseFilesResponse,
        id,
        String
    );
    wrap_lookup_handler!(
        get_work_releases,
        get_work_releases_handler,
        GetWorkReleasesResponse,
        id,
        String
    );
    wrap_lookup_handler!(
        get_creator_releases,
        get_creator_releases_handler,
        GetCreatorReleasesResponse,
        id,
        String
    );

    fn accept_editgroup(
        &self,
        id: i64,
        _context: &Context,
    ) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");

        accept_editgroup(id as i64, &conn).expect("failed to accept editgroup");

        let ret = AcceptEditgroupResponse::MergedSuccessfully(Success {
            message: "horray!".to_string(),
        });
        Box::new(futures::done(Ok(ret)))
    }

    fn get_editgroup(
        &self,
        id: i64,
        _context: &Context,
    ) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        let ret = match self.get_editgroup_handler(id) {
            Ok(entity) =>
                GetEditgroupResponse::FoundEntity(entity),
            Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) =>
                GetEditgroupResponse::NotFound(
                    ErrorResponse { message: format!("No such editgroup: {}", id) }),
            Err(e) =>
                // TODO: dig in to error type here
                GetEditgroupResponse::GenericError(
                    ErrorResponse { message: e.to_string() }),
        };
        Box::new(futures::done(Ok(ret)))
    }

    fn create_editgroup(
        &self,
        entity: models::Editgroup,
        _context: &Context,
    ) -> Box<Future<Item = CreateEditgroupResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");

        let row: EditgroupRow = insert_into(editgroup::table)
            .values((
                editgroup::editor_id.eq(entity.editor_id as i64),
                editgroup::description.eq(entity.description),
                editgroup::extra_json.eq(entity.extra),
            ))
            .get_result(&conn)
            .expect("error creating edit group");

        let new_eg = Editgroup {
            id: Some(row.id),
            editor_id: row.editor_id,
            description: row.description,
            edits: None,
            extra: row.extra_json,
        };
        Box::new(futures::done(Ok(
            CreateEditgroupResponse::SuccessfullyCreated(new_eg),
        )))
    }

    fn get_editor_changelog(
        &self,
        username: String,
        _context: &Context,
    ) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send> {
        let ret = match self.editor_changelog_get_handler(username.clone()) {
            Ok(entries) => GetEditorChangelogResponse::FoundMergedChanges(entries),
            Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) => {
                GetEditorChangelogResponse::NotFound(ErrorResponse {
                    message: format!("No such editor: {}", username.clone()),
                })
            }
            Err(e) => {
                // TODO: dig in to error type here
                error!("{}", e);
                GetEditorChangelogResponse::GenericError(ErrorResponse {
                    message: e.to_string(),
                })
            }
        };
        Box::new(futures::done(Ok(ret)))
    }

    fn get_editor(
        &self,
        username: String,
        _context: &Context,
    ) -> Box<Future<Item = GetEditorResponse, Error = ApiError> + Send> {
        let ret = match self.get_editor_handler(username.clone()) {
            Ok(entity) => GetEditorResponse::FoundEditor(entity),
            Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) => {
                GetEditorResponse::NotFound(ErrorResponse {
                    message: format!("No such editor: {}", username.clone()),
                })
            }
            Err(e) => {
                // TODO: dig in to error type here
                error!("{}", e);
                GetEditorResponse::GenericError(ErrorResponse {
                    message: e.to_string(),
                })
            }
        };
        Box::new(futures::done(Ok(ret)))
    }

    fn get_stats(
        &self,
        more: Option<String>,
        _context: &Context,
    ) -> Box<Future<Item = GetStatsResponse, Error = ApiError> + Send> {
        let ret = match self.get_stats_handler(more.clone()) {
            Ok(stats) => GetStatsResponse::Success(stats),
            Err(e) => {
                error!("{}", e);
                GetStatsResponse::GenericError(ErrorResponse {
                    message: e.to_string(),
                })
            }
        };
        Box::new(futures::done(Ok(ret)))
    }
}
