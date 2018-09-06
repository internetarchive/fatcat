//! API endpoint handlers

use api_helpers::fcid2uuid;
use api_server::Server;
use diesel::Connection;
use errors::*;
use fatcat_api::models;
use fatcat_api::models::*;
use fatcat_api::*;
use futures::{self, Future};

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
            $post_batch_resp:ident, $update_fn:ident, $update_handler:ident, $update_resp:ident,
            $delete_fn:ident, $delete_handler:ident, $delete_resp:ident, $get_history_fn:ident,
            $get_history_handler:ident, $get_history_resp:ident, $model:ident) => {

        fn $get_fn(
            &self,
            id: String,
            expand: Option<String>,
            _context: &Context,
        ) -> Box<Future<Item = $get_resp, Error = ApiError> + Send> {
            let id = if let Ok(parsed) = fcid2uuid(&id) { parsed } else {
                return Box::new(futures::done(Ok($get_resp::BadRequest(ErrorResponse {
                    message: ErrorKind::InvalidFatcatId(id).to_string() }))));
            };
            let conn = self.db_pool.get().expect("db_pool error");
            // No transaction for GET
            let ret = match self.$get_handler(&id, expand, &conn) {
                Ok(entity) =>
                    $get_resp::FoundEntity(entity),
                Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) =>
                    $get_resp::NotFound(ErrorResponse { message: format!("No such entity {}: {}", stringify!($model), id) }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $get_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::InvalidFatcatId(e), _)) =>
                    $get_resp::BadRequest(ErrorResponse {
                        message: ErrorKind::InvalidFatcatId(e).to_string() }),
                Err(Error(ErrorKind::MalformedExternalId(e), _)) =>
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
            let conn = self.db_pool.get().expect("db_pool error");
            let ret = match conn.transaction(|| self.$post_handler(entity, &conn)) {
                Ok(edit) =>
                    $post_resp::CreatedEntity(edit),
                Err(Error(ErrorKind::Diesel(e), _)) =>
                    $post_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $post_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::InvalidFatcatId(e), _)) =>
                    $post_resp::BadRequest(ErrorResponse {
                        message: ErrorKind::InvalidFatcatId(e).to_string() }),
                Err(Error(ErrorKind::MalformedExternalId(e), _)) =>
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
            let conn = self.db_pool.get().expect("db_pool error");
            let ret = match conn.transaction(|| self.$post_batch_handler(entity_list, &conn)) {
                Ok(edit) =>
                    $post_batch_resp::CreatedEntities(edit),
                Err(Error(ErrorKind::Diesel(e), _)) =>
                    $post_batch_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $post_batch_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::InvalidFatcatId(e), _)) =>
                    $post_batch_resp::BadRequest(ErrorResponse {
                        message: ErrorKind::InvalidFatcatId(e).to_string() }),
                Err(Error(ErrorKind::MalformedExternalId(e), _)) =>
                    $post_batch_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(e) => {
                    error!("{}", e);
                    $post_batch_resp::GenericError(ErrorResponse { message: e.to_string() })
                },
            };
            Box::new(futures::done(Ok(ret)))
        }

        fn $update_fn(
            &self,
            id: String,
            entity: models::$model,
            _context: &Context,
        ) -> Box<Future<Item = $update_resp, Error = ApiError> + Send> {
            let id = if let Ok(parsed) = fcid2uuid(&id) { parsed } else {
                return Box::new(futures::done(Ok($update_resp::BadRequest(ErrorResponse {
                    message: ErrorKind::InvalidFatcatId(id).to_string() }))));
            };
            let conn = self.db_pool.get().expect("db_pool error");
            let ret = match conn.transaction(|| self.$update_handler(&id, entity, &conn)) {
                Ok(edit) =>
                    $update_resp::UpdatedEntity(edit),
                Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) =>
                    $update_resp::NotFound(ErrorResponse { message: format!("No such entity {}: {}", stringify!($model), id) }),
                Err(Error(ErrorKind::Diesel(e), _)) =>
                    $update_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $update_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::InvalidFatcatId(e), _)) =>
                    $update_resp::BadRequest(ErrorResponse {
                        message: ErrorKind::InvalidFatcatId(e).to_string() }),
                Err(Error(ErrorKind::MalformedExternalId(e), _)) =>
                    $update_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(e) => {
                    error!("{}", e);
                    $update_resp::GenericError(ErrorResponse { message: e.to_string() })
                },
            };
            Box::new(futures::done(Ok(ret)))
        }

        fn $delete_fn(
            &self,
            id: String,
            editgroup_id: Option<String>,
            _context: &Context,
        ) -> Box<Future<Item = $delete_resp, Error = ApiError> + Send> {
            let id = if let Ok(parsed) = fcid2uuid(&id) { parsed } else {
                return Box::new(futures::done(Ok($delete_resp::BadRequest(ErrorResponse {
                    message: ErrorKind::InvalidFatcatId(id).to_string() }))));
            };
            let editgroup_id = match editgroup_id {
                Some(raw) => if let Ok(parsed) = fcid2uuid(&raw) { Some(parsed) } else {
                    return Box::new(futures::done(Ok($delete_resp::BadRequest(ErrorResponse {
                        message: ErrorKind::InvalidFatcatId(raw).to_string() }))))
                }
                None => None
            };
            let conn = self.db_pool.get().expect("db_pool error");
            let ret = match conn.transaction(|| self.$delete_handler(&id, editgroup_id, &conn)) {
                Ok(edit) =>
                    $delete_resp::DeletedEntity(edit),
                Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) =>
                    $delete_resp::NotFound(ErrorResponse { message: format!("No such entity {}: {}", stringify!($model), id) }),
                Err(Error(ErrorKind::Diesel(e), _)) =>
                    $delete_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $delete_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::InvalidFatcatId(e), _)) =>
                    $delete_resp::BadRequest(ErrorResponse {
                        message: ErrorKind::InvalidFatcatId(e).to_string() }),
                Err(Error(ErrorKind::MalformedExternalId(e), _)) =>
                    $delete_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(e) => {
                    error!("{}", e);
                    $delete_resp::GenericError(ErrorResponse { message: e.to_string() })
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
            let id = if let Ok(parsed) = fcid2uuid(&id) { parsed } else {
                return Box::new(futures::done(Ok($get_history_resp::BadRequest(ErrorResponse {
                    message: ErrorKind::InvalidFatcatId(id).to_string() }))));
            };
            let conn = self.db_pool.get().expect("db_pool error");
            // No transaction for GET
            let ret = match self.$get_history_handler(&id, limit, &conn) {
                Ok(history) =>
                    $get_history_resp::FoundEntityHistory(history),
                Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) =>
                    $get_history_resp::NotFound(ErrorResponse { message: format!("No such entity {}: {}", stringify!($model), id) }),
                Err(Error(ErrorKind::Uuid(e), _)) =>
                    $get_history_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(Error(ErrorKind::InvalidFatcatId(e), _)) =>
                    $get_history_resp::BadRequest(ErrorResponse {
                        message: ErrorKind::InvalidFatcatId(e).to_string() }),
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
            let conn = self.db_pool.get().expect("db_pool error");
            // No transaction for GET
            let ret = match self.$get_handler(&$idname, &conn) {
                Ok(entity) =>
                    $get_resp::FoundEntity(entity),
                Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) =>
                    $get_resp::NotFound(ErrorResponse { message: format!("Not found: {}", $idname) }),
                Err(Error(ErrorKind::MalformedExternalId(e), _)) =>
                    $get_resp::BadRequest(ErrorResponse { message: e.to_string() }),
                Err(e) => {
                    error!("{}", e);
                    $get_resp::BadRequest(ErrorResponse { message: e.to_string() })
                },
            };
            Box::new(futures::done(Ok(ret)))
        }
    }
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
        update_container,
        update_container_handler,
        UpdateContainerResponse,
        delete_container,
        delete_container_handler,
        DeleteContainerResponse,
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
        update_creator,
        update_creator_handler,
        UpdateCreatorResponse,
        delete_creator,
        delete_creator_handler,
        DeleteCreatorResponse,
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
        update_file,
        update_file_handler,
        UpdateFileResponse,
        delete_file,
        delete_file_handler,
        DeleteFileResponse,
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
        update_release,
        update_release_handler,
        UpdateReleaseResponse,
        delete_release,
        delete_release_handler,
        DeleteReleaseResponse,
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
        update_work,
        update_work_handler,
        UpdateWorkResponse,
        delete_work,
        delete_work_handler,
        DeleteWorkResponse,
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
        id: String,
        _context: &Context,
    ) -> Box<Future<Item = AcceptEditgroupResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        let ret = match conn.transaction(|| self.accept_editgroup_handler(&id, &conn)) {
            Ok(()) => AcceptEditgroupResponse::MergedSuccessfully(Success {
                message: "horray!".to_string(),
            }),
            Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) => {
                AcceptEditgroupResponse::NotFound(ErrorResponse {
                    message: format!("No such editgroup: {}", id),
                })
            }
            Err(Error(ErrorKind::EditgroupAlreadyAccepted(e), _)) => {
                AcceptEditgroupResponse::BadRequest(ErrorResponse {
                    message: ErrorKind::EditgroupAlreadyAccepted(e).to_string(),
                })
            }
            Err(e) => AcceptEditgroupResponse::GenericError(ErrorResponse {
                message: e.to_string(),
            }),
        };
        Box::new(futures::done(Ok(ret)))
    }

    fn get_editgroup(
        &self,
        id: String,
        _context: &Context,
    ) -> Box<Future<Item = GetEditgroupResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        let ret = match conn.transaction(|| self.get_editgroup_handler(&id, &conn)) {
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
        let ret = match conn.transaction(|| self.create_editgroup_handler(entity, &conn)) {
            Ok(eg) =>
                CreateEditgroupResponse::SuccessfullyCreated(eg),
            Err(e) =>
                // TODO: dig in to error type here
                CreateEditgroupResponse::GenericError(
                    ErrorResponse { message: e.to_string() }),
        };
        Box::new(futures::done(Ok(ret)))
    }

    fn get_editor_changelog(
        &self,
        username: String,
        _context: &Context,
    ) -> Box<Future<Item = GetEditorChangelogResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        // No transaction for GET
        let ret = match self.editor_changelog_get_handler(&username, &conn) {
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
        let conn = self.db_pool.get().expect("db_pool error");
        // No transaction for GET
        let ret = match self.get_editor_handler(&username, &conn) {
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

    fn get_changelog(
        &self,
        limit: Option<i64>,
        _context: &Context,
    ) -> Box<Future<Item = GetChangelogResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        // No transaction for GET
        let ret = match self.get_changelog_handler(limit, &conn) {
            Ok(changelog) => GetChangelogResponse::Success(changelog),
            Err(e) => {
                error!("{}", e);
                GetChangelogResponse::GenericError(ErrorResponse {
                    message: e.to_string(),
                })
            }
        };
        Box::new(futures::done(Ok(ret)))
    }

    fn get_changelog_entry(
        &self,
        id: i64,
        _context: &Context,
    ) -> Box<Future<Item = GetChangelogEntryResponse, Error = ApiError> + Send> {
        let conn = self.db_pool.get().expect("db_pool error");
        // No transaction for GET
        let ret = match self.get_changelog_entry_handler(id, &conn) {
            Ok(entry) => GetChangelogEntryResponse::FoundChangelogEntry(entry),
            Err(Error(ErrorKind::Diesel(::diesel::result::Error::NotFound), _)) => {
                GetChangelogEntryResponse::NotFound(ErrorResponse {
                    message: format!("No such changelog entry: {}", id),
                })
            }
            Err(e) => {
                error!("{}", e);
                GetChangelogEntryResponse::GenericError(ErrorResponse {
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
        let conn = self.db_pool.get().expect("db_pool error");
        // No transaction for GET
        let ret = match self.get_stats_handler(&more, &conn) {
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
