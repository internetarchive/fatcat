//! API endpoint handlers

use ConnectionPool;
use database_models::*;
use database_schema::{container_rev, container_ident, container_edit,
                      creator_rev, creator_ident, creator_edit,
                      file_rev, file_ident, file_edit,
                      release_rev, release_ident, release_edit,
                      work_rev, work_ident, work_edit,
                      editor, editgroup, changelog
};
use uuid;
use diesel::prelude::*;
use diesel::{self, insert_into};
use futures::{self, Future};
use fatcat_api::models;
use fatcat_api::models::*;
use fatcat_api::{Api, ApiError, ContainerIdGetResponse, ContainerLookupGetResponse,
                 ContainerPostResponse, Context, CreatorIdGetResponse, CreatorLookupGetResponse,
                 CreatorPostResponse, EditgroupIdAcceptPostResponse, EditgroupIdGetResponse,
                 EditgroupPostResponse, EditorUsernameChangelogGetResponse,
                 EditorUsernameGetResponse, FileIdGetResponse, FileLookupGetResponse,
                 FilePostResponse, ReleaseIdGetResponse, ReleaseLookupGetResponse,
                 ReleasePostResponse, WorkIdGetResponse, WorkPostResponse};

#[derive(Clone)]
pub struct Server {
    pub db_pool: ConnectionPool,
}

impl Api for Server {
    fn container_id_get(
        &self,
        id: String,
        _context: &Context,
    ) -> Box<Future<Item = ContainerIdGetResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        let id = uuid::Uuid::parse_str(&id).unwrap();
        let (ident, rev): (ContainerIdentRow, ContainerRevRow) = container_ident::table
            .find(id)
            .inner_join(container_rev::table)
            .first(&conn)
            .expect("error loading container");

        let entity = ContainerEntity {
            issn: rev.issn,
            publisher: rev.publisher,
            parent: None, // TODO
            name: Some(rev.name), // TODO: not optional
            state: None, // TODO:
            ident: Some(ident.id.to_string()),
            revision: ident.rev_id.map(|v| v as isize),
            redirect: ident.redirect_id.map(|u| u.to_string()),
            editgroup: None,
        };
        Box::new(futures::done(Ok(
            ContainerIdGetResponse::FetchASingleContainerById(entity),
        )))
    }

    fn container_lookup_get(
        &self,
        issn: String,
        context: &Context,
    ) -> Box<Future<Item = ContainerLookupGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "container_lookup_get(\"{}\") - X-Span-ID: {:?}",
            issn,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn container_post(
        &self,
        body: Option<models::ContainerEntity>,
        context: &Context,
    ) -> Box<Future<Item = ContainerPostResponse, Error = ApiError> + Send> {
        println!("{:?}", body);
        let body = body.expect("missing body"); // TODO: required parameter
        //let editgroup_id: i64 = body.editgroup.expect("need editgroup_id") as i64; // TODO: or find/create
        let editgroup_id = 1;
        let conn = self.db_pool.get().expect("db_pool error");

        let name = body.name.unwrap();
        let issn = body.issn.unwrap();
        println!("name={} issn={}", name, issn);

        let edit: Vec<ContainerEditRow> = diesel::sql_query(
            "WITH rev AS ( INSERT INTO container_rev (name, issn)
                        VALUES ($1, $2)
                        RETURNING id ),
                ident AS ( INSERT INTO container_ident (rev_id)
                            VALUES ((SELECT rev.id FROM rev))
                            RETURNING id )
            INSERT INTO container_edit (editgroup_id, ident_id, rev_id) VALUES
                ($3, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
            RETURNING *")
            .bind::<diesel::sql_types::Text, _>(name)
            .bind::<diesel::sql_types::Text, _>(issn)
            .bind::<diesel::sql_types::BigInt, _>(editgroup_id)
            .load(&conn)
            .unwrap();
        let edit = &edit[0];

        let entity_edit = EntityEdit {
            editgroup_id: Some(edit.editgroup_id as isize),
            revision: Some(edit.rev_id.unwrap() as isize),
            ident: Some(edit.ident_id.to_string()),
            id: Some(edit.id as isize),
        };
        Box::new(futures::done(Ok(
            ContainerPostResponse::Created(entity_edit),
        )))
    }

    fn creator_id_get(
        &self,
        id: String,
        _context: &Context,
    ) -> Box<Future<Item = CreatorIdGetResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        /*
        let first_thing: (Uuid, bool, Option<i64>, Option<Uuid>) = creator_ident::table
            .first(&conn)
            .unwrap();
        let entity_table = creator_ident::table.left_join(creator_rev::table);
        let thing: (creator_ident::SqlType, creator_rev::SqlType) = creator_ident::table
            .inner_join(creator_rev::table)
            .first(&conn)
            .expect("Error loading creator");
        */
        let ce = CreatorEntity {
            orcid: None,
            name: None,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            editgroup: None,
        };
        Box::new(futures::done(Ok(
            CreatorIdGetResponse::FetchASingleCreatorById(ce),
        )))
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
        body: Option<models::CreatorEntity>,
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

    fn file_id_get(
        &self,
        id: String,
        context: &Context,
    ) -> Box<Future<Item = FileIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "file_id_get(\"{}\") - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
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
        body: Option<models::FileEntity>,
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

    fn release_id_get(
        &self,
        id: String,
        context: &Context,
    ) -> Box<Future<Item = ReleaseIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "release_id_get(\"{}\") - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
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
        body: Option<models::ReleaseEntity>,
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

    fn work_id_get(
        &self,
        id: String,
        context: &Context,
    ) -> Box<Future<Item = WorkIdGetResponse, Error = ApiError> + Send> {
        let context = context.clone();
        println!(
            "work_id_get(\"{}\") - X-Span-ID: {:?}",
            id,
            context.x_span_id.unwrap_or(String::from("<none>")).clone()
        );
        Box::new(futures::failed("Generic failure".into()))
    }

    fn work_post(
        &self,
        body: Option<models::WorkEntity>,
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
}
