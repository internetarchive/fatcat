//! API endpoint handlers

use ConnectionPool;
use api_helpers::*;
use chrono;
use database_models::*;
use database_schema::{changelog, container_ident, container_rev, creator_ident, creator_rev,
                      editgroup, editor, file_ident, file_rev, release_ident, release_rev,
                      work_ident, work_rev};
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
macro_rules! wrap_get_id_handler {
    ($get_fn:ident, $handler:ident, $resp:ident, $idtype:ident) => {
        fn $get_fn(
            &self,
            id: $idtype,
            _context: &Context,
        ) -> Box<Future<Item = $resp, Error = ApiError> + Send> {
            match self.$handler(id) {
                Ok(Some(entity)) =>
                    Box::new(futures::done(Ok($resp::FoundEntity(entity)))),
                Ok(None) =>
                    Box::new(futures::done(Ok($resp::NotFound(
                        ErrorResponse { message: "No such entity".to_string() }),
                    ))),
                Err(e) =>
                    // TODO: dig in to error type here
                    Box::new(futures::done(Ok($resp::BadRequest(
                        ErrorResponse { message: e.to_string() },
                    )))),
            }
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
            match self.$handler($idname) {
                Ok(Some(entity)) =>
                    Box::new(futures::done(Ok($resp::FoundEntity(entity)))),
                Ok(None) =>
                    Box::new(futures::done(Ok($resp::NotFound(
                        ErrorResponse { message: "No such entity".to_string() }),
                    ))),
                Err(e) =>
                    // TODO: dig in to error type here
                    Box::new(futures::done(Ok($resp::BadRequest(
                        ErrorResponse { message: e.to_string() },
                    )))),
            }
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
            issn: rev.issn,
            publisher: rev.publisher,
            name: rev.name,
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id,
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    fn container_lookup_get_handler(&self, issn: String) -> Result<Option<ContainerEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");

        let res: ::std::result::Result<(ContainerIdentRow, ContainerRevRow), _> =
            container_ident::table
                .inner_join(container_rev::table)
                .filter(container_rev::issn.eq(&issn))
                .filter(container_ident::is_live.eq(true))
                .filter(container_ident::redirect_id.is_null())
                .first(&conn);

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let entity = ContainerEntity {
            issn: rev.issn,
            publisher: rev.publisher,
            name: rev.name,
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
            name: rev.name,
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
            name: rev.name,
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

        let entity = FileEntity {
            sha1: rev.sha1,
            size: rev.size.map(|v| v as i64),
            url: rev.url,
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id.map(|v| v),
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

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

        let entity = FileEntity {
            sha1: rev.sha1,
            size: rev.size.map(|v| v as i64),
            url: rev.url,
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

        let entity = ReleaseEntity {
            title: rev.title,
            release_type: rev.release_type,
            //date: rev.date,
            doi: rev.doi,
            volume: rev.volume,
            pages: rev.pages,
            issue: rev.issue,
            container_id: None, // TODO
            work_id: rev.work_ident_id.to_string(),
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

        let entity = ReleaseEntity {
            title: rev.title,
            release_type: rev.release_type,
            //date: rev.date,
            doi: rev.doi,
            volume: rev.volume,
            pages: rev.pages,
            issue: rev.issue,
            container_id: None, // TODO
            work_id: rev.work_ident_id.to_string(),
            state: Some(ident.state().unwrap().shortname()),
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id,
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup_id: None,
            extra: rev.extra_json,
        };
        Ok(Some(entity))
    }

    fn editgroup_id_get_handler(&self, id: i64) -> Result<Option<Editgroup>> {
        let conn = self.db_pool.get().expect("db_pool error");

        let row: EditgroupRow = editgroup::table.find(id as i64).first(&conn)?;

        let eg = Editgroup {
            id: Some(row.id),
            editor_id: row.editor_id,
            description: row.description,
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
    wrap_get_id_handler!(
        container_id_get,
        container_id_get_handler,
        ContainerIdGetResponse,
        String
    );
    wrap_get_id_handler!(
        creator_id_get,
        creator_id_get_handler,
        CreatorIdGetResponse,
        String
    );
    wrap_get_id_handler!(file_id_get, file_id_get_handler, FileIdGetResponse, String);
    wrap_get_id_handler!(work_id_get, work_id_get_handler, WorkIdGetResponse, String);
    wrap_get_id_handler!(
        release_id_get,
        release_id_get_handler,
        ReleaseIdGetResponse,
        String
    );
    wrap_get_id_handler!(
        editgroup_id_get,
        editgroup_id_get_handler,
        EditgroupIdGetResponse,
        i64
    );

    wrap_lookup_handler!(
        container_lookup_get,
        container_lookup_get_handler,
        ContainerLookupGetResponse,
        issn,
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

    fn container_post(
        &self,
        body: models::ContainerEntity,
        _context: &Context,
    ) -> Box<Future<Item = ContainerPostResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let edit: ContainerEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO container_rev (name, publisher, issn)
                        VALUES ($1, $2, $3)
                        RETURNING id ),
                ident AS ( INSERT INTO container_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO container_edit (editgroup_id, ident_id, rev_id) VALUES
                ($4, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(body.name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.publisher)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.issn)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .get_result(&conn)
            .unwrap();
        let edit = &edit;

        let entity_edit = EntityEdit {
            editgroup_id: Some(edit.editgroup_id),
            revision: Some(edit.rev_id.unwrap()),
            ident: Some(edit.ident_id.to_string()),
            edit_id: Some(edit.id),
            extra: edit.extra_json.clone(),
        };
        Box::new(futures::done(Ok(ContainerPostResponse::CreatedEntity(
            entity_edit,
        ))))
    }

    fn creator_post(
        &self,
        body: models::CreatorEntity,
        _context: &Context,
    ) -> Box<Future<Item = CreatorPostResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let edit: CreatorEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO creator_rev (name, orcid)
                        VALUES ($1, $2)
                        RETURNING id ),
                ident AS ( INSERT INTO creator_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO creator_edit (editgroup_id, ident_id, rev_id) VALUES
                ($3, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(body.name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.orcid)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .get_result(&conn)
            .unwrap();
        let edit = &edit;

        let entity_edit = EntityEdit {
            editgroup_id: Some(edit.editgroup_id),
            revision: Some(edit.rev_id.unwrap()),
            ident: Some(edit.ident_id.to_string()),
            edit_id: Some(edit.id),
            extra: edit.extra_json.clone(),
        };
        Box::new(futures::done(Ok(CreatorPostResponse::CreatedEntity(
            entity_edit,
        ))))
    }

    fn file_post(
        &self,
        body: models::FileEntity,
        _context: &Context,
    ) -> Box<Future<Item = FilePostResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let edit: FileEditRow =
            diesel::sql_query(
                "WITH rev AS ( INSERT INTO file_rev (size, sha1, url)
                        VALUES ($1, $2, $3)
                        RETURNING id ),
                ident AS ( INSERT INTO file_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO file_edit (editgroup_id, ident_id, rev_id) VALUES
                ($4, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
            ).bind::<diesel::sql_types::Nullable<diesel::sql_types::Int8>, _>(body.size)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.sha1)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.url)
                .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
                .get_result(&conn)
                .unwrap();
        let edit = &edit;

        let entity_edit = EntityEdit {
            editgroup_id: Some(edit.editgroup_id),
            revision: Some(edit.rev_id.unwrap()),
            ident: Some(edit.ident_id.to_string()),
            edit_id: Some(edit.id),
            extra: edit.extra_json.clone(),
        };
        Box::new(futures::done(Ok(FilePostResponse::CreatedEntity(
            entity_edit,
        ))))
    }

    fn work_post(
        &self,
        body: models::WorkEntity,
        _context: &Context,
    ) -> Box<Future<Item = WorkPostResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let edit: WorkEditRow =
            diesel::sql_query(
                "WITH rev AS ( INSERT INTO work_rev (work_type)
                        VALUES ($1)
                        RETURNING id ),
                ident AS ( INSERT INTO work_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO work_edit (editgroup_id, ident_id, rev_id) VALUES
                ($2, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
            ).bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.work_type)
                .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
                .get_result(&conn)
                .unwrap();
        let edit = &edit;

        let entity_edit = EntityEdit {
            editgroup_id: Some(edit.editgroup_id),
            revision: Some(edit.rev_id.unwrap()),
            ident: Some(edit.ident_id.to_string()),
            edit_id: Some(edit.id),
            extra: edit.extra_json.clone(),
        };
        Box::new(futures::done(Ok(WorkPostResponse::CreatedEntity(
            entity_edit,
        ))))
    }

    fn release_post(
        &self,
        body: models::ReleaseEntity,
        _context: &Context,
    ) -> Box<Future<Item = ReleasePostResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        let editor_id = 1; // TODO: auth
        let editgroup_id = match body.editgroup_id {
            None => get_or_create_editgroup(editor_id, &conn).expect("current editgroup"),
            Some(param) => param as i64,
        };

        let work_id = uuid::Uuid::parse_str(&body.work_id).expect("invalid UUID");
        let _container_id: Option<uuid::Uuid> = match body.container_id {
            Some(id) => Some(uuid::Uuid::parse_str(&id).expect("invalid UUID")),
            None => None,
        };

        let edit: ReleaseEditRow = diesel::sql_query(
            "WITH rev AS ( INSERT INTO release_rev (title, release_type, doi, volume, pages, issue, work_ident_id)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        RETURNING id ),
                ident AS ( INSERT INTO release_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO release_edit (editgroup_id, ident_id, rev_id) VALUES
                ($8, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(body.title)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.release_type)
            //.bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.date)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.doi)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.volume)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.pages)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.issue)
            .bind::<diesel::sql_types::Uuid, _>(work_id)
            //.bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(body.container_id)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .get_result(&conn)
            .unwrap();
        let edit = &edit;

        let entity_edit = EntityEdit {
            editgroup_id: Some(edit.editgroup_id),
            revision: Some(edit.rev_id.unwrap()),
            ident: Some(edit.ident_id.to_string()),
            edit_id: Some(edit.id),
            extra: edit.extra_json.clone(),
        };
        Box::new(futures::done(Ok(ReleasePostResponse::CreatedEntity(
            entity_edit,
        ))))
    }

    fn editgroup_id_accept_post(
        &self,
        id: i64,
        _context: &Context,
    ) -> Box<Future<Item = EditgroupIdAcceptPostResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");

        accept_editgroup(id as i64, &conn).expect("failed to accept editgroup");

        Box::new(futures::done(Ok(
            EditgroupIdAcceptPostResponse::MergedSuccessfully(Success {
                message: "horray!".to_string(),
            }),
        )))
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
        match self.editor_changelog_get_handler(username) {
            Ok(Some(entries)) =>
                Box::new(futures::done(Ok(EditorUsernameChangelogGetResponse::FoundMergedChanges(entries)))),
            Ok(None) =>
                Box::new(futures::done(Ok(EditorUsernameChangelogGetResponse::NotFound(
                    ErrorResponse { message: "No such entity".to_string() }),
                ))),
            Err(e) =>
                // TODO: dig in to error type here
                Box::new(futures::done(Ok(EditorUsernameChangelogGetResponse::GenericError(
                    ErrorResponse { message: e.to_string() },
                )))),
        }
    }

    fn editor_username_get(
        &self,
        username: String,
        _context: &Context,
    ) -> Box<Future<Item = EditorUsernameGetResponse, Error = ApiError> + Send> {
        match self.editor_get_handler(username) {
            Ok(Some(entity)) =>
                Box::new(futures::done(Ok(EditorUsernameGetResponse::FoundEditor(entity)))),
            Ok(None) =>
                Box::new(futures::done(Ok(EditorUsernameGetResponse::NotFound(
                    ErrorResponse { message: "No such entity".to_string() }),
                ))),
            Err(e) =>
                // TODO: dig in to error type here
                Box::new(futures::done(Ok(EditorUsernameGetResponse::GenericError(
                    ErrorResponse { message: e.to_string() },
                )))),
        }
    }
}
