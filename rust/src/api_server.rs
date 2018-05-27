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
use fatcat_api::{Api, ApiError, ContainerIdGetResponse, ContainerLookupGetResponse,
                 ContainerPostResponse, Context, CreatorIdGetResponse, CreatorLookupGetResponse,
                 CreatorPostResponse, EditgroupIdAcceptPostResponse, EditgroupIdGetResponse,
                 EditgroupPostResponse, EditorUsernameChangelogGetResponse,
                 EditorUsernameGetResponse, FileIdGetResponse, FileLookupGetResponse,
                 FilePostResponse, ReleaseIdGetResponse, ReleaseLookupGetResponse,
                 ReleasePostResponse, WorkIdGetResponse, WorkPostResponse};
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
            $post_resp:ident, $model:ident) => {
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
                Err(e) => $get_resp::GenericError(ErrorResponse { message: e.to_string() }),
            };
            Box::new(futures::done(Ok(ret)))
        }

        fn $post_fn(
            &self,
            body: models::$model,
            _context: &Context,
        ) -> Box<Future<Item = $post_resp, Error = ApiError> + Send> {
            let ret = match self.$post_handler(body) {
                Ok(edit) =>
                    $post_resp::CreatedEntity(edit),
                Err(Error(ErrorKind::Diesel(e), _)) =>
                    $post_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(e) => $post_resp::GenericError(ErrorResponse { message: e.to_string() }),
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
                Err(e) =>
                    $get_resp::BadRequest(ErrorResponse { message: e.to_string() }),
            };
            Box::new(futures::done(Ok(ret)))
        }
    }
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
        full_name: rev.full_name,
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
    conn: DbConn,
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
        .get_results(&conn)?
        .iter()
        .map(|r: &FileReleaseRow| r.target_release_ident_id.to_string())
        .collect();

    Ok(FileEntity {
        sha1: rev.sha1,
        md5: rev.md5,
        size: rev.size.map(|v| v as i64),
        url: rev.url,
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
    conn: DbConn,
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
        .get_results(&conn)
        .expect("fetch release refs")
        .iter()
        .map(|r: &ReleaseRefRow| ReleaseRef {
            index: r.index.clone(),
            stub: r.stub.clone(),
            target_release_id: r.target_release_ident_id.map(|v| v.to_string()),
        })
        .collect();

    let contribs: Vec<ReleaseContrib> = release_contrib::table
        .filter(release_contrib::release_rev.eq(rev.id))
        .get_results(&conn)
        .expect("fetch release refs")
        .iter()
        .map(|c: &ReleaseContribRow| ReleaseContrib {
            index: c.index,
            role: c.role.clone(),
            creator_stub: c.stub.clone(),
            creator_id: c.creator_ident_id.map(|v| v.to_string()),
        })
        .collect();

    Ok(ReleaseEntity {
        title: rev.title,
        release_type: rev.release_type,
        date: rev.date
            .map(|v| chrono::DateTime::from_utc(v.and_hms(0, 0, 0), chrono::Utc)),
        doi: rev.doi,
        isbn13: rev.isbn13,
        volume: rev.volume,
        pages: rev.pages,
        issue: rev.issue,
        container_id: rev.container_ident_id.map(|u| u.to_string()),
        publisher: rev.publisher,
        work_id: rev.work_ident_id.to_string(),
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
    fn container_id_get_handler(&self, id: String) -> Result<ContainerEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        // TODO: handle Deletions
        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = container_ident::table
            .find(id)
            .inner_join(container_rev::table)
            .first(&conn)?;

        container_row2entity(Some(ident), rev)
    }

    fn container_lookup_get_handler(&self, issnl: String) -> Result<ContainerEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = container_ident::table
            .inner_join(container_rev::table)
            .filter(container_rev::issnl.eq(&issnl))
            .filter(container_ident::is_live.eq(true))
            .filter(container_ident::redirect_id.is_null())
            .first(&conn)?;

        container_row2entity(Some(ident), rev)
    }

    fn creator_id_get_handler(&self, id: String) -> Result<CreatorEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let (ident, rev): (CreatorIdentRow, CreatorRevRow) = creator_ident::table
            .find(id)
            .inner_join(creator_rev::table)
            .first(&conn)?;

        creator_row2entity(Some(ident), rev)
    }

    fn creator_lookup_get_handler(&self, orcid: String) -> Result<CreatorEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        let (ident, rev): (CreatorIdentRow, CreatorRevRow) = creator_ident::table
            .inner_join(creator_rev::table)
            .filter(creator_rev::orcid.eq(&orcid))
            .filter(creator_ident::is_live.eq(true))
            .filter(creator_ident::redirect_id.is_null())
            .first(&conn)?;

        creator_row2entity(Some(ident), rev)
    }

    fn file_id_get_handler(&self, id: String) -> Result<FileEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let (ident, rev): (FileIdentRow, FileRevRow) = file_ident::table
            .find(id)
            .inner_join(file_rev::table)
            .first(&conn)?;

        file_row2entity(Some(ident), rev, conn)
    }

    fn file_lookup_get_handler(&self, sha1: String) -> Result<FileEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        let (ident, rev): (FileIdentRow, FileRevRow) = file_ident::table
            .inner_join(file_rev::table)
            .filter(file_rev::sha1.eq(&sha1))
            .filter(file_ident::is_live.eq(true))
            .filter(file_ident::redirect_id.is_null())
            .first(&conn)?;

        file_row2entity(Some(ident), rev, conn)
    }

    fn release_id_get_handler(&self, id: String) -> Result<ReleaseEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) = release_ident::table
            .find(id)
            .inner_join(release_rev::table)
            .first(&conn)?;

        release_row2entity(Some(ident), rev, conn)
    }

    fn release_lookup_get_handler(&self, doi: String) -> Result<ReleaseEntity> {
        let conn = self.db_pool.get().expect("db_pool error");

        let (ident, rev): (ReleaseIdentRow, ReleaseRevRow) = release_ident::table
            .inner_join(release_rev::table)
            .filter(release_rev::doi.eq(&doi))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .first(&conn)?;

        release_row2entity(Some(ident), rev, conn)
    }

    fn work_id_get_handler(&self, id: String) -> Result<WorkEntity> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let (ident, rev): (WorkIdentRow, WorkRevRow) = work_ident::table
            .find(id)
            .inner_join(work_rev::table)
            .first(&conn)?;

        work_row2entity(Some(ident), rev)
    }

    fn container_post_handler(&self, body: models::ContainerEntity) -> Result<EntityEdit> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
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
        ).bind::<diesel::sql_types::Text, _>(body.name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.publisher)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.issnl)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.abbrev)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.coden)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(body.extra)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .get_result(&conn)?;

        edit.to_model()
    }

    fn creator_post_handler(&self, body: models::CreatorEntity) -> Result<EntityEdit> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let edit: CreatorEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO creator_rev (full_name, orcid, extra_json)
                        VALUES ($1, $2, $3)
                        RETURNING id ),
                ident AS ( INSERT INTO creator_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO creator_edit (editgroup_id, ident_id, rev_id) VALUES
                ($4, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(body.full_name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.orcid)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(body.extra)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .get_result(&conn)?;

        edit.to_model()
    }

    fn file_post_handler(&self, body: models::FileEntity) -> Result<EntityEdit> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let edit: FileEditRow =
            diesel::sql_query(
                "WITH rev AS ( INSERT INTO file_rev (size, sha1, md5, url, extra_json)
                        VALUES ($1, $2, $3, $4, $5)
                        RETURNING id ),
                ident AS ( INSERT INTO file_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO file_edit (editgroup_id, ident_id, rev_id) VALUES
                ($6, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
            ).bind::<diesel::sql_types::Nullable<diesel::sql_types::Int8>, _>(body.size)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.sha1)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.md5)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.url)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(body.extra)
                .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
                .get_result(&conn)?;

        let _releases: Option<Vec<FileReleaseRow>> = match body.releases {
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
                        .get_results(&conn)
                        .expect("error inserting file_releases");
                    Some(release_rows)
                }
            }
        };

        edit.to_model()
    }

    fn work_post_handler(&self, body: models::WorkEntity) -> Result<EntityEdit> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
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
            ).bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.work_type)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(body.extra)
                .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
                .get_result(&conn)?;

        edit.to_model()
    }

    fn release_post_handler(&self, body: models::ReleaseEntity) -> Result<EntityEdit> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let work_id = uuid::Uuid::parse_str(&body.work_id).expect("invalid UUID");
        let container_id: Option<uuid::Uuid> = match body.container_id {
            Some(id) => Some(uuid::Uuid::parse_str(&id).expect("invalid UUID")),
            None => None,
        };

        let edit: ReleaseEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO release_rev (title, release_type, date, doi, isbn13, volume, pages, issue, work_ident_id, container_ident_id, publisher, extra_json)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                        RETURNING id ),
                ident AS ( INSERT INTO release_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO release_edit (editgroup_id, ident_id, rev_id) VALUES
                ($13, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(body.title)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.release_type)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Date>, _>(
                body.date.map(|v| v.naive_utc().date()))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.doi)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.isbn13)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.volume)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.pages)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.issue)
            .bind::<diesel::sql_types::Uuid, _>(work_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(container_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.publisher)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Json>, _>(body.extra)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .get_result(&conn)?;

        let _refs: Option<Vec<ReleaseRefRow>> = match body.refs {
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
                            stub: r.stub.clone(),
                        })
                        .collect();
                    let ref_rows: Vec<ReleaseRefRow> = insert_into(release_ref::table)
                        .values(ref_rows)
                        .get_results(&conn)
                        .expect("error inserting release_refs");
                    Some(ref_rows)
                }
            }
        };

        let _contribs: Option<Vec<ReleaseContribRow>> = match body.contribs {
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
                            stub: c.creator_stub.clone(),
                        })
                        .collect();
                    let contrib_rows: Vec<ReleaseContribRow> = insert_into(release_contrib::table)
                        .values(contrib_rows)
                        .get_results(&conn)
                        .expect("error inserting release_contribs");
                    Some(contrib_rows)
                }
            }
        };

        edit.to_model()
    }

    fn editgroup_id_get_handler(&self, id: i64) -> Result<Option<Editgroup>> {
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
        Ok(Some(eg))
    }

    fn editor_get_handler(&self, username: String) -> Result<Option<Editor>> {
        let conn = self.db_pool.get().expect("db_pool error");

        let row: EditorRow = editor::table
            .filter(editor::username.eq(&username))
            .first(&conn)?;

        let ed = Editor {
            username: row.username,
        };
        Ok(Some(ed))
    }

    fn editor_changelog_get_handler(&self, username: String) -> Result<Option<Changelogentries>> {
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
            .map(|(row, _)| ChangelogentriesInner {
                index: row.id,
                editgroup_id: row.editgroup_id,
                timestamp: chrono::DateTime::from_utc(row.timestamp, chrono::Utc),
            })
            .collect();
        Ok(Some(entries))
    }
}

impl Api for Server {
    wrap_entity_handlers!(
        container_id_get,
        container_id_get_handler,
        ContainerIdGetResponse,
        container_post,
        container_post_handler,
        ContainerPostResponse,
        ContainerEntity
    );
    wrap_entity_handlers!(
        creator_id_get,
        creator_id_get_handler,
        CreatorIdGetResponse,
        creator_post,
        creator_post_handler,
        CreatorPostResponse,
        CreatorEntity
    );
    wrap_entity_handlers!(
        file_id_get,
        file_id_get_handler,
        FileIdGetResponse,
        file_post,
        file_post_handler,
        FilePostResponse,
        FileEntity
    );
    wrap_entity_handlers!(
        release_id_get,
        release_id_get_handler,
        ReleaseIdGetResponse,
        release_post,
        release_post_handler,
        ReleasePostResponse,
        ReleaseEntity
    );
    wrap_entity_handlers!(
        work_id_get,
        work_id_get_handler,
        WorkIdGetResponse,
        work_post,
        work_post_handler,
        WorkPostResponse,
        WorkEntity
    );

    wrap_lookup_handler!(
        container_lookup_get,
        container_lookup_get_handler,
        ContainerLookupGetResponse,
        issnl,
        String
    );
    wrap_lookup_handler!(
        creator_lookup_get,
        creator_lookup_get_handler,
        CreatorLookupGetResponse,
        orcid,
        String
    );
    wrap_lookup_handler!(
        file_lookup_get,
        file_lookup_get_handler,
        FileLookupGetResponse,
        sha1,
        String
    );
    wrap_lookup_handler!(
        release_lookup_get,
        release_lookup_get_handler,
        ReleaseLookupGetResponse,
        doi,
        String
    );

    fn editgroup_id_accept_post(
        &self,
        id: i64,
        _context: &Context,
    ) -> Box<Future<Item = EditgroupIdAcceptPostResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");

        accept_editgroup(id as i64, &conn).expect("failed to accept editgroup");

        let ret = EditgroupIdAcceptPostResponse::MergedSuccessfully(Success {
            message: "horray!".to_string(),
        });
        Box::new(futures::done(Ok(ret)))
    }

    fn editgroup_id_get(
        &self,
        id: i64,
        _context: &Context,
    ) -> Box<Future<Item = EditgroupIdGetResponse, Error = ApiError> + Send> {
        let ret = match self.editgroup_id_get_handler(id) {
            Ok(Some(entity)) =>
                EditgroupIdGetResponse::FoundEntity(entity),
            Ok(None) =>
                EditgroupIdGetResponse::NotFound(
                    ErrorResponse { message: "No such entity".to_string() }),
            Err(e) =>
                // TODO: dig in to error type here
                EditgroupIdGetResponse::BadRequest(
                    ErrorResponse { message: e.to_string() }),
        };
        Box::new(futures::done(Ok(ret)))
    }

    fn editgroup_post(
        &self,
        body: models::Editgroup,
        _context: &Context,
    ) -> Box<Future<Item = EditgroupPostResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");

        let row: EditgroupRow = insert_into(editgroup::table)
            .values((
                editgroup::editor_id.eq(body.editor_id as i64),
                editgroup::description.eq(body.description),
                editgroup::extra_json.eq(body.extra),
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
            EditgroupPostResponse::SuccessfullyCreated(new_eg),
        )))
    }

    fn editor_username_changelog_get(
        &self,
        username: String,
        _context: &Context,
    ) -> Box<Future<Item = EditorUsernameChangelogGetResponse, Error = ApiError> + Send> {
        let ret = match self.editor_changelog_get_handler(username) {
            Ok(Some(entries)) =>
                EditorUsernameChangelogGetResponse::FoundMergedChanges(entries),
            Ok(None) =>
                EditorUsernameChangelogGetResponse::NotFound(
                    ErrorResponse { message: "No such entity".to_string() }),
            Err(e) =>
                // TODO: dig in to error type here
                EditorUsernameChangelogGetResponse::GenericError(
                    ErrorResponse { message: e.to_string() }),
        };
        Box::new(futures::done(Ok(ret)))
    }

    fn editor_username_get(
        &self,
        username: String,
        _context: &Context,
    ) -> Box<Future<Item = EditorUsernameGetResponse, Error = ApiError> + Send> {
        let ret = match self.editor_get_handler(username) {
            Ok(Some(entity)) =>
                EditorUsernameGetResponse::FoundEditor(entity),
            Ok(None) =>
                EditorUsernameGetResponse::NotFound(ErrorResponse { message: "No such entity".to_string() }),
            Err(e) =>
                // TODO: dig in to error type here
                EditorUsernameGetResponse::GenericError(ErrorResponse { message: e.to_string() }),
        };
        Box::new(futures::done(Ok(ret)))
    }
}
