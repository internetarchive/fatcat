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

// Helper for calling through to handlers
macro_rules! wrap_entity_handlers {
    ($get_fn:ident, $get_handler:ident, $get_resp:ident, $post_fn:ident, $post_handler:ident,
            $post_resp:ident, $model:ident) => {
        fn $get_fn(
            &self,
            id: String,
            _context: &Context,
        ) -> Box<Future<Item = $get_resp, Error = ApiError> + Send> {
            let ret = match self.$get_handler(id.clone()) {
                Ok(Some(entity)) =>
                    $get_resp::FoundEntity(entity),
                Ok(None) =>
                    $get_resp::NotFound(ErrorResponse { message: format!("No such entity {}: {}", stringify!($model), id) }),
                Err(e) => $get_resp::BadRequest(ErrorResponse { message: e.to_string() }),
            };
            Box::new(futures::done(Ok(ret)))
        }

        fn $post_fn(
            &self,
            body: models::$model,
            _context: &Context,
        ) -> Box<Future<Item = $post_resp, Error = ApiError> + Send> {
            // TODO: look for diesel foreign key and other type errors, return as BadRequest; other
            // errors are a 500.
            let ret = match self.$post_handler(body) {
                Ok(edit) =>
                    $post_resp::CreatedEntity(edit),
                Err(e) => $post_resp::BadRequest(ErrorResponse { message: e.to_string() }),
            };
            Box::new(futures::done(Ok(ret)))
        }
    }
}
macro_rules! wrap_lookup_handler {
    ($get_fn:ident, $handler:ident, $resp:ident, $idname:ident, $idtype:ident) => {
        fn $get_fn(
            &self,
            $idname: $idtype,
            _context: &Context,
        ) -> Box<Future<Item = $resp, Error = ApiError> + Send> {
            let ret = match self.$handler($idname) {
                Ok(Some(entity)) =>
                    $resp::FoundEntity(entity),
                Ok(None) =>
                    $resp::NotFound(ErrorResponse { message: "No such entity".to_string() }),
                Err(e) =>
                    // TODO: dig in to error type here
                    $resp::BadRequest(ErrorResponse { message: e.to_string() }),
            };
            Box::new(futures::done(Ok(ret)))
        }
    }
}

#[derive(Clone)]
pub struct Server {
    pub db_pool: ConnectionPool,
}

impl Server {
    fn container_id_get_handler(&self, id: String) -> Result<Option<ContainerEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let res: ::std::result::Result<(ContainerIdentRow, ContainerRevRow), _> =
            container_ident::table
                .find(id)
                .inner_join(container_rev::table)
                .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let entity = ContainerEntity {
            issnl: rev.issnl,
            publisher: rev.publisher,
            name: rev.name,
            abbrev: rev.abbrev,
            coden: rev.coden,
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id,
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    fn container_lookup_get_handler(&self, issnl: String) -> Result<Option<ContainerEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");

        let res: ::std::result::Result<(ContainerIdentRow, ContainerRevRow), _> =
            container_ident::table
                .inner_join(container_rev::table)
                .filter(container_rev::issnl.eq(&issnl))
                .filter(container_ident::is_live.eq(true))
                .filter(container_ident::redirect_id.is_null())
                .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let entity = ContainerEntity {
            issnl: rev.issnl,
            publisher: rev.publisher,
            name: rev.name,
            abbrev: rev.abbrev,
            coden: rev.coden,
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id,
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    fn creator_id_get_handler(&self, id: String) -> Result<Option<CreatorEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let res: ::std::result::Result<(CreatorIdentRow, CreatorRevRow), _> = creator_ident::table
            .find(id)
            .inner_join(creator_rev::table)
            .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let entity = CreatorEntity {
            full_name: rev.full_name,
            orcid: rev.orcid,
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id,
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    fn creator_lookup_get_handler(&self, orcid: String) -> Result<Option<CreatorEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");

        let res: ::std::result::Result<(CreatorIdentRow, CreatorRevRow), _> = creator_ident::table
            .inner_join(creator_rev::table)
            .filter(creator_rev::orcid.eq(&orcid))
            .filter(creator_ident::is_live.eq(true))
            .filter(creator_ident::redirect_id.is_null())
            .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let entity = CreatorEntity {
            full_name: rev.full_name,
            orcid: rev.orcid,
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id,
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    fn file_id_get_handler(&self, id: String) -> Result<Option<FileEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let res: ::std::result::Result<(FileIdentRow, FileRevRow), _> = file_ident::table
            .find(id)
            .inner_join(file_rev::table)
            .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let releases: Vec<String> = file_release::table
            .filter(file_release::file_rev.eq(rev.id))
            .get_results(&conn)
            .expect("fetch file releases")
            .iter()
            .map(|r: &FileReleaseRow| r.target_release_ident_id.to_string())
            .collect();

        let entity = FileEntity {
            sha1: rev.sha1,
            md5: rev.md5,
            size: rev.size.map(|v| v as i64),
            url: rev.url,
            releases: Some(releases),
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id.map(|v| v),
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    // TODO: refactor this to not be redundant with file_id_get_handler() code
    fn file_lookup_get_handler(&self, sha1: String) -> Result<Option<FileEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");

        let res: ::std::result::Result<(FileIdentRow, FileRevRow), _> = file_ident::table
            .inner_join(file_rev::table)
            .filter(file_rev::sha1.eq(&sha1))
            .filter(file_ident::is_live.eq(true))
            .filter(file_ident::redirect_id.is_null())
            .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let releases: Vec<String> = file_release::table
            .filter(file_release::file_rev.eq(rev.id))
            .get_results(&conn)
            .expect("fetch file releases")
            .iter()
            .map(|r: &FileReleaseRow| r.target_release_ident_id.to_string())
            .collect();

        let entity = FileEntity {
            sha1: rev.sha1,
            md5: rev.md5,
            size: rev.size.map(|v| v as i64),
            url: rev.url,
            releases: Some(releases),
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id.map(|v| v),
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    fn work_id_get_handler(&self, id: String) -> Result<Option<WorkEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let res: ::std::result::Result<(WorkIdentRow, WorkRevRow), _> = work_ident::table
            .find(id)
            .inner_join(work_rev::table)
            .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let entity = WorkEntity {
            work_type: rev.work_type,
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id,
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    fn release_id_get_handler(&self, id: String) -> Result<Option<ReleaseEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let res: ::std::result::Result<(ReleaseIdentRow, ReleaseRevRow), _> = release_ident::table
            .find(id)
            .inner_join(release_rev::table)
            .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
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

        let entity = ReleaseEntity {
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
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id,
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    fn release_lookup_get_handler(&self, doi: String) -> Result<Option<ReleaseEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");

        let res: ::std::result::Result<(ReleaseIdentRow, ReleaseRevRow), _> = release_ident::table
            .inner_join(release_rev::table)
            .filter(release_rev::doi.eq(&doi))
            .filter(release_ident::is_live.eq(true))
            .filter(release_ident::redirect_id.is_null())
            .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
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

        let entity = ReleaseEntity {
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
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id,
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
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
        let edit = &edit;

        Ok(EntityEdit {
            editgroup_id: edit.editgroup_id,
            revision: Some(edit.rev_id.unwrap()),
            redirect_ident: None,
            ident: edit.ident_id.to_string(),
            edit_id: edit.id,
            extra: edit.extra_json.clone(),
        })
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
        let edit = &edit;

        Ok(EntityEdit {
            editgroup_id: edit.editgroup_id,
            revision: Some(edit.rev_id.unwrap()),
            redirect_ident: None,
            ident: edit.ident_id.to_string(),
            edit_id: edit.id,
            extra: edit.extra_json.clone(),
        })
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
        let edit = &edit;

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

        Ok(EntityEdit {
            editgroup_id: edit.editgroup_id,
            revision: Some(edit.rev_id.unwrap()),
            redirect_ident: None,
            ident: edit.ident_id.to_string(),
            edit_id: edit.id,
            extra: edit.extra_json.clone(),
        })
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
        let edit = &edit;

        Ok(EntityEdit {
            editgroup_id: edit.editgroup_id,
            revision: Some(edit.rev_id.unwrap()),
            redirect_ident: None,
            ident: edit.ident_id.to_string(),
            edit_id: edit.id,
            extra: edit.extra_json.clone(),
        })
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
        let edit = &edit;

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

        Ok(EntityEdit {
            editgroup_id: edit.editgroup_id,
            revision: Some(edit.rev_id.unwrap()),
            redirect_ident: None,
            ident: edit.ident_id.to_string(),
            edit_id: edit.id,
            extra: edit.extra_json.clone(),
        })
    }

    fn editgroup_id_get_handler(&self, id: i64) -> Result<Option<Editgroup>> {
        let conn = self.db_pool.get().expect("db_pool error");

        let row: EditgroupRow = editgroup::table.find(id as i64).first(&conn)?;

        let edits = EditgroupEdits {
            containers: Some(
                container_edit::table
                    .filter(container_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .iter()
                    .map(|e: &ContainerEditRow| EntityEdit {
                        edit_id: e.id,
                        editgroup_id: e.editgroup_id,
                        revision: e.rev_id,
                        redirect_ident: e.redirect_id.map(|v| v.to_string()),
                        ident: e.ident_id.to_string(),
                        extra: e.extra_json.clone(),
                    })
                    .collect(),
            ),
            creators: Some(
                creator_edit::table
                    .filter(creator_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .iter()
                    .map(|e: &CreatorEditRow| EntityEdit {
                        edit_id: e.id,
                        editgroup_id: e.editgroup_id,
                        revision: e.rev_id,
                        redirect_ident: e.redirect_id.map(|v| v.to_string()),
                        ident: e.ident_id.to_string(),
                        extra: e.extra_json.clone(),
                    })
                    .collect(),
            ),
            files: Some(
                file_edit::table
                    .filter(file_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .iter()
                    .map(|e: &FileEditRow| EntityEdit {
                        edit_id: e.id,
                        editgroup_id: e.editgroup_id,
                        revision: e.rev_id,
                        redirect_ident: e.redirect_id.map(|v| v.to_string()),
                        ident: e.ident_id.to_string(),
                        extra: e.extra_json.clone(),
                    })
                    .collect(),
            ),
            releases: Some(
                release_edit::table
                    .filter(release_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .iter()
                    .map(|e: &ReleaseEditRow| EntityEdit {
                        edit_id: e.id,
                        editgroup_id: e.editgroup_id,
                        revision: e.rev_id,
                        redirect_ident: e.redirect_id.map(|v| v.to_string()),
                        ident: e.ident_id.to_string(),
                        extra: e.extra_json.clone(),
                    })
                    .collect(),
            ),
            works: Some(
                work_edit::table
                    .filter(work_edit::editgroup_id.eq(id))
                    .get_results(&conn)?
                    .iter()
                    .map(|e: &WorkEditRow| EntityEdit {
                        edit_id: e.id,
                        editgroup_id: e.editgroup_id,
                        revision: e.rev_id,
                        redirect_ident: e.redirect_id.map(|v| v.to_string()),
                        ident: e.ident_id.to_string(),
                        extra: e.extra_json.clone(),
                    })
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
