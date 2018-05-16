//! API endpoint handlers

use ConnectionPool;
use database_models::*;
use database_schema::{changelog, container_edit, container_ident, container_rev, creator_edit,
                      creator_ident, creator_rev, editgroup, editor, file_edit, file_ident,
                      file_rev, release_edit, release_ident, release_rev, work_edit, work_ident,
                      work_rev};
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

#[derive(Clone)]
pub struct Server {
    pub db_pool: ConnectionPool,
}

impl Server {

    fn container_id_get_handler(&self, id: String) -> Result<Option<ContainerEntity>> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id)?;

        let res: ::std::result::Result<(ContainerIdentRow, ContainerRevRow), _> = container_ident::table
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
            parent: None, // TODO:
            name: rev.name,
            state: None, // TODO:
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id.map(|v| v as isize),
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup: None,
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
            state: None, // TODO:
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id.map(|v| v as isize),
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup: None,
        };
        Ok(Some(entity))
    }

    // TODO:
    fn file_id_get_handler(&self, id: String) -> Result<Option<FileEntity>> {
        Ok(None)
    }

    // TODO:
    fn work_id_get_handler(&self, id: String) -> Result<Option<WorkEntity>> {
        Ok(None)
    }

    // TODO:
    fn release_id_get_handler(&self, id: String) -> Result<Option<ReleaseEntity>> {
        Ok(None)
    }
}

impl Api for Server {

    fn container_id_get(
        &self,
        id: String,
        _context: &Context,
    ) -> Box<Future<Item = ContainerIdGetResponse, Error = ApiError> + Send> {
        match self.container_id_get_handler(id) {
            Ok(Some(entity)) =>
                Box::new(futures::done(Ok(ContainerIdGetResponse::FoundEntity(entity)))),
            Ok(None) =>
                Box::new(futures::done(Ok(ContainerIdGetResponse::NotFound(
                    ErrorResponse { message: "No such entity".to_string() }),
                ))),
            Err(e) =>
                // TODO: dig in to error type here
                Box::new(futures::done(Ok(ContainerIdGetResponse::BadRequest(
                    ErrorResponse { message: e.to_string() },
                )))),
        }
    }

    fn container_lookup_get(
        &self,
        issn: String,
        _context: &Context,
    ) -> Box<Future<Item = ContainerLookupGetResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");

        let res: ::std::result::Result<(ContainerIdentRow, ContainerRevRow), _> = container_ident::table
            .inner_join(container_rev::table)
            .first(&conn);
        // XXX: actually do a filter/lookup

        let (ident, rev) = match res {
            Ok(r) => r,
            Err(_) => {
                return Box::new(futures::done(Ok(
                    // TODO: UGH, need to add 404 responses everywhere, not 400
                    //ContainerIdGetResponse::NotFound(
                    ContainerLookupGetResponse::BadRequest(ErrorResponse {
                        message: "No such container".to_string(),
                    }),
                )));
            }
        };

        let entity = ContainerEntity {
            issn: rev.issn,
            publisher: rev.publisher,
            parent: None, // TODO:
            name: rev.name,
            state: None, // TODO:
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id.map(|v| v as isize),
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup: None,
        };
        Box::new(futures::done(Ok(ContainerLookupGetResponse::FoundEntity(
            entity,
        ))))
    }

    fn container_post(
        &self,
        body: models::ContainerEntity,
        _context: &Context,
    ) -> Box<Future<Item = ContainerPostResponse, Error = ApiError> + Send> {
        println!("{:?}", body);
        //let editgroup_id: i64 = body.editgroup.expect("need editgroup_id") as i64;
        // TODO: or find/create
        let editgroup_id = 1;
        let conn = self.db_pool.get().expect("db_pool error");

        let name = body.name;
        let issn = body.issn;
        println!("name={} issn={:?}", name, issn);

        let edit: Vec<ContainerEditRow> = diesel::sql_query(
            "WITH rev AS ( INSERT INTO container_rev (name, issn)
                        VALUES ($1, $2)
                        RETURNING id ),
                ident AS ( INSERT INTO container_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO container_edit (editgroup_id, ident_id, rev_id) VALUES
                ($3, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *",
        ).bind::<diesel::sql_types::Text, _>(name)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(issn)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .load(&conn)
            .unwrap();
        let edit = &edit[0];

        let entity_edit = EntityEdit {
            editgroup_id: Some(edit.editgroup_id as isize),
            revision: Some(edit.rev_id.unwrap() as isize),
            ident: Some(edit.ident_id.to_string()),
            edit_id: Some(edit.id as isize),
        };
        Box::new(futures::done(Ok(ContainerPostResponse::CreatedEntity(
            entity_edit,
        ))))
    }

    fn creator_id_get(
        &self,
        id: String,
        _context: &Context,
    ) -> Box<Future<Item = CreatorIdGetResponse, Error = ApiError> + Send> {
        match self.creator_id_get_handler(id) {
            Ok(Some(entity)) =>
                Box::new(futures::done(Ok(CreatorIdGetResponse::FoundEntity(entity)))),
            Ok(None) =>
                Box::new(futures::done(Ok(CreatorIdGetResponse::NotFound(
                    ErrorResponse { message: "No such entity".to_string() }),
                ))),
            Err(e) =>
                // TODO: dig in to error type here
                Box::new(futures::done(Ok(CreatorIdGetResponse::BadRequest(
                    ErrorResponse { message: e.to_string() },
                )))),
        }
    }

    fn creator_lookup_get(
        &self,
        orcid: String,
        context: &Context,
    ) -> Box<Future<Item = CreatorLookupGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "creator_lookup_get(\"{}\") - X-Span-ID: {:?}",
            orcid,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn creator_post(
        &self,
        body: models::CreatorEntity,
        context: &Context,
    ) -> Box<Future<Item = CreatorPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "creator_post({:?}) - X-Span-ID: {:?}",
            body,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn file_id_get(
        &self,
        id: String,
        _context: &Context,
    ) -> Box<Future<Item = FileIdGetResponse, Error = ApiError> + Send> {
        match self.file_id_get_handler(id) {
            Ok(Some(entity)) =>
                Box::new(futures::done(Ok(FileIdGetResponse::FoundEntity(entity)))),
            Ok(None) =>
                Box::new(futures::done(Ok(FileIdGetResponse::NotFound(
                    ErrorResponse { message: "No such entity".to_string() }),
                ))),
            Err(e) =>
                // TODO: dig in to error type here
                Box::new(futures::done(Ok(FileIdGetResponse::BadRequest(
                    ErrorResponse { message: e.to_string() },
                )))),
        }
    }

    fn file_lookup_get(
        &self,
        sha1: String,
        context: &Context,
    ) -> Box<Future<Item = FileLookupGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "file_lookup_get(\"{}\") - X-Span-ID: {:?}",
            sha1,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn file_post(
        &self,
        body: models::FileEntity,
        context: &Context,
    ) -> Box<Future<Item = FilePostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "file_post({:?}) - X-Span-ID: {:?}",
            body,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn work_id_get(
        &self,
        id: String,
        _context: &Context,
    ) -> Box<Future<Item = WorkIdGetResponse, Error = ApiError> + Send> {
        match self.work_id_get_handler(id) {
            Ok(Some(entity)) =>
                Box::new(futures::done(Ok(WorkIdGetResponse::FoundEntity(entity)))),
            Ok(None) =>
                Box::new(futures::done(Ok(WorkIdGetResponse::NotFound(
                    ErrorResponse { message: "No such entity".to_string() }),
                ))),
            Err(e) =>
                // TODO: dig in to error type here
                Box::new(futures::done(Ok(WorkIdGetResponse::BadRequest(
                    ErrorResponse { message: e.to_string() },
                )))),
        }
    }

    fn work_post(
        &self,
        body: models::WorkEntity,
        context: &Context,
    ) -> Box<Future<Item = WorkPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "work_post({:?}) - X-Span-ID: {:?}",
            body,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn release_id_get(
        &self,
        id: String,
        _context: &Context,
    ) -> Box<Future<Item = ReleaseIdGetResponse, Error = ApiError> + Send> {
        match self.release_id_get_handler(id) {
            Ok(Some(entity)) =>
                Box::new(futures::done(Ok(ReleaseIdGetResponse::FoundEntity(entity)))),
            Ok(None) =>
                Box::new(futures::done(Ok(ReleaseIdGetResponse::NotFound(
                    ErrorResponse { message: "No such entity".to_string() }),
                ))),
            Err(e) =>
                // TODO: dig in to error type here
                Box::new(futures::done(Ok(ReleaseIdGetResponse::BadRequest(
                    ErrorResponse { message: e.to_string() },
                )))),
        }
    }

    fn release_lookup_get(
        &self,
        doi: String,
        context: &Context,
    ) -> Box<Future<Item = ReleaseLookupGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "release_lookup_get(\"{}\") - X-Span-ID: {:?}",
            doi,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn release_post(
        &self,
        body: models::ReleaseEntity,
        context: &Context,
    ) -> Box<Future<Item = ReleasePostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "release_post({:?}) - X-Span-ID: {:?}",
            body,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editgroup_id_accept_post(
        &self,
        id: i32,
        context: &Context,
    ) -> Box<Future<Item = EditgroupIdAcceptPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "editgroup_id_accept_post({}) - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editgroup_id_get(
        &self,
        id: i32,
        context: &Context,
    ) -> Box<Future<Item = EditgroupIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "editgroup_id_get({}) - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editgroup_post(
        &self,
        context: &Context,
    ) -> Box<Future<Item = EditgroupPostResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "editgroup_post() - X-Span-ID: {:?}",
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editor_username_changelog_get(
        &self,
        username: String,
        context: &Context,
    ) -> Box<Future<Item = EditorUsernameChangelogGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "editor_username_changelog_get(\"{}\") - X-Span-ID: {:?}",
            username,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn editor_username_get(
        &self,
        username: String,
        context: &Context,
    ) -> Box<Future<Item = EditorUsernameGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "editor_username_get(\"{}\") - X-Span-ID: {:?}",
            username,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }
}
