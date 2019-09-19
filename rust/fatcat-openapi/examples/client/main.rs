#![allow(missing_docs, unused_variables, trivial_casts)]

use clap::{App, Arg};
#[allow(unused_imports)]
use fatcat_openapi::{
    models, AcceptEditgroupResponse, Api, ApiError, ApiNoContext, AuthCheckResponse,
    AuthOidcResponse, Client, ContextWrapperExt, CreateAuthTokenResponse,
    CreateContainerAutoBatchResponse, CreateContainerResponse, CreateCreatorAutoBatchResponse,
    CreateCreatorResponse, CreateEditgroupAnnotationResponse, CreateEditgroupResponse,
    CreateFileAutoBatchResponse, CreateFileResponse, CreateFilesetAutoBatchResponse,
    CreateFilesetResponse, CreateReleaseAutoBatchResponse, CreateReleaseResponse,
    CreateWebcaptureAutoBatchResponse, CreateWebcaptureResponse, CreateWorkAutoBatchResponse,
    CreateWorkResponse, DeleteContainerEditResponse, DeleteContainerResponse,
    DeleteCreatorEditResponse, DeleteCreatorResponse, DeleteFileEditResponse, DeleteFileResponse,
    DeleteFilesetEditResponse, DeleteFilesetResponse, DeleteReleaseEditResponse,
    DeleteReleaseResponse, DeleteWebcaptureEditResponse, DeleteWebcaptureResponse,
    DeleteWorkEditResponse, DeleteWorkResponse, GetChangelogEntryResponse, GetChangelogResponse,
    GetContainerEditResponse, GetContainerHistoryResponse, GetContainerRedirectsResponse,
    GetContainerResponse, GetContainerRevisionResponse, GetCreatorEditResponse,
    GetCreatorHistoryResponse, GetCreatorRedirectsResponse, GetCreatorReleasesResponse,
    GetCreatorResponse, GetCreatorRevisionResponse, GetEditgroupAnnotationsResponse,
    GetEditgroupResponse, GetEditgroupsReviewableResponse, GetEditorAnnotationsResponse,
    GetEditorEditgroupsResponse, GetEditorResponse, GetFileEditResponse, GetFileHistoryResponse,
    GetFileRedirectsResponse, GetFileResponse, GetFileRevisionResponse, GetFilesetEditResponse,
    GetFilesetHistoryResponse, GetFilesetRedirectsResponse, GetFilesetResponse,
    GetFilesetRevisionResponse, GetReleaseEditResponse, GetReleaseFilesResponse,
    GetReleaseFilesetsResponse, GetReleaseHistoryResponse, GetReleaseRedirectsResponse,
    GetReleaseResponse, GetReleaseRevisionResponse, GetReleaseWebcapturesResponse,
    GetWebcaptureEditResponse, GetWebcaptureHistoryResponse, GetWebcaptureRedirectsResponse,
    GetWebcaptureResponse, GetWebcaptureRevisionResponse, GetWorkEditResponse,
    GetWorkHistoryResponse, GetWorkRedirectsResponse, GetWorkReleasesResponse, GetWorkResponse,
    GetWorkRevisionResponse, LookupContainerResponse, LookupCreatorResponse, LookupFileResponse,
    LookupReleaseResponse, UpdateContainerResponse, UpdateCreatorResponse, UpdateEditgroupResponse,
    UpdateEditorResponse, UpdateFileResponse, UpdateFilesetResponse, UpdateReleaseResponse,
    UpdateWebcaptureResponse, UpdateWorkResponse,
};
#[allow(unused_imports)]
use futures::{future, stream, Future, Stream};

#[allow(unused_imports)]
use log::info;

// swagger::Has may be unused if there are no examples
#[allow(unused_imports)]
use swagger::{AuthData, ContextBuilder, EmptyContext, Has, Push, XSpanIdString};

// rt may be unused if there are no examples
#[allow(unused_mut)]
fn main() {
    env_logger::init();

    let matches = App::new("client")
        .arg(
            Arg::with_name("operation")
                .help("Sets the operation to run")
                .possible_values(&[
                    "AcceptEditgroup",
                    "AuthCheck",
                    "CreateAuthToken",
                    "DeleteContainer",
                    "DeleteContainerEdit",
                    "DeleteCreator",
                    "DeleteCreatorEdit",
                    "DeleteFile",
                    "DeleteFileEdit",
                    "DeleteFileset",
                    "DeleteFilesetEdit",
                    "DeleteRelease",
                    "DeleteReleaseEdit",
                    "DeleteWebcapture",
                    "DeleteWebcaptureEdit",
                    "DeleteWork",
                    "DeleteWorkEdit",
                    "GetChangelog",
                    "GetChangelogEntry",
                    "GetContainer",
                    "GetContainerEdit",
                    "GetContainerHistory",
                    "GetContainerRedirects",
                    "GetContainerRevision",
                    "GetCreator",
                    "GetCreatorEdit",
                    "GetCreatorHistory",
                    "GetCreatorRedirects",
                    "GetCreatorReleases",
                    "GetCreatorRevision",
                    "GetEditgroup",
                    "GetEditgroupAnnotations",
                    "GetEditgroupsReviewable",
                    "GetEditor",
                    "GetEditorAnnotations",
                    "GetEditorEditgroups",
                    "GetFile",
                    "GetFileEdit",
                    "GetFileHistory",
                    "GetFileRedirects",
                    "GetFileRevision",
                    "GetFileset",
                    "GetFilesetEdit",
                    "GetFilesetHistory",
                    "GetFilesetRedirects",
                    "GetFilesetRevision",
                    "GetRelease",
                    "GetReleaseEdit",
                    "GetReleaseFiles",
                    "GetReleaseFilesets",
                    "GetReleaseHistory",
                    "GetReleaseRedirects",
                    "GetReleaseRevision",
                    "GetReleaseWebcaptures",
                    "GetWebcapture",
                    "GetWebcaptureEdit",
                    "GetWebcaptureHistory",
                    "GetWebcaptureRedirects",
                    "GetWebcaptureRevision",
                    "GetWork",
                    "GetWorkEdit",
                    "GetWorkHistory",
                    "GetWorkRedirects",
                    "GetWorkReleases",
                    "GetWorkRevision",
                    "LookupContainer",
                    "LookupCreator",
                    "LookupFile",
                    "LookupRelease",
                ])
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("https")
                .long("https")
                .help("Whether to use HTTPS or not"),
        )
        .arg(
            Arg::with_name("host")
                .long("host")
                .takes_value(true)
                .default_value("api.fatcat.wiki")
                .help("Hostname to contact"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .takes_value(true)
                .default_value("8080")
                .help("Port to contact"),
        )
        .get_matches();

    let is_https = matches.is_present("https");
    let base_url = format!(
        "{}://{}:{}",
        if is_https { "https" } else { "http" },
        matches.value_of("host").unwrap(),
        matches.value_of("port").unwrap()
    );

    let client = if matches.is_present("https") {
        // Using Simple HTTPS
        Client::try_new_https(&base_url).expect("Failed to create HTTPS client")
    } else {
        // Using HTTP
        Client::try_new_http(&base_url).expect("Failed to create HTTP client")
    };

    let context: swagger::make_context_ty!(
        ContextBuilder,
        EmptyContext,
        Option<AuthData>,
        XSpanIdString
    ) = swagger::make_context!(
        ContextBuilder,
        EmptyContext,
        None as Option<AuthData>,
        XSpanIdString::default()
    );

    let client = client.with_context(context);

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    match matches.value_of("operation") {
        Some("AcceptEditgroup") => {
            let result = rt.block_on(client.accept_editgroup("editgroup_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("AuthCheck") => {
            let result = rt.block_on(client.auth_check(Some("role_example".to_string())));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
        Some("AuthOidc") => {
            let result = rt.block_on(client.auth_oidc(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        Some("CreateAuthToken") => {
            let result =
                rt.block_on(client.create_auth_token("editor_id_example".to_string(), Some(56)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
        Some("CreateContainer") => {
            let result = rt.block_on(client.create_container(
                  "editgroup_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateContainerAutoBatch") => {
            let result = rt.block_on(client.create_container_auto_batch(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateCreator") => {
            let result = rt.block_on(client.create_creator(
                  "editgroup_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateCreatorAutoBatch") => {
            let result = rt.block_on(client.create_creator_auto_batch(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateEditgroup") => {
            let result = rt.block_on(client.create_editgroup(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateEditgroupAnnotation") => {
            let result = rt.block_on(client.create_editgroup_annotation(
                  "editgroup_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateFile") => {
            let result = rt.block_on(client.create_file(
                  "editgroup_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateFileAutoBatch") => {
            let result = rt.block_on(client.create_file_auto_batch(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateFileset") => {
            let result = rt.block_on(client.create_fileset(
                  "editgroup_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateFilesetAutoBatch") => {
            let result = rt.block_on(client.create_fileset_auto_batch(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateRelease") => {
            let result = rt.block_on(client.create_release(
                  "editgroup_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateReleaseAutoBatch") => {
            let result = rt.block_on(client.create_release_auto_batch(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateWebcapture") => {
            let result = rt.block_on(client.create_webcapture(
                  "editgroup_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateWebcaptureAutoBatch") => {
            let result = rt.block_on(client.create_webcapture_auto_batch(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateWork") => {
            let result = rt.block_on(client.create_work(
                  "editgroup_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("CreateWorkAutoBatch") => {
            let result = rt.block_on(client.create_work_auto_batch(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        Some("DeleteContainer") => {
            let result = rt.block_on(client.delete_container(
                "editgroup_id_example".to_string(),
                "ident_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteContainerEdit") => {
            let result = rt.block_on(client.delete_container_edit(
                "editgroup_id_example".to_string(),
                "edit_id_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteCreator") => {
            let result = rt.block_on(client.delete_creator(
                "editgroup_id_example".to_string(),
                "ident_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteCreatorEdit") => {
            let result = rt.block_on(client.delete_creator_edit(
                "editgroup_id_example".to_string(),
                "edit_id_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteFile") => {
            let result = rt.block_on(client.delete_file(
                "editgroup_id_example".to_string(),
                "ident_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteFileEdit") => {
            let result = rt.block_on(client.delete_file_edit(
                "editgroup_id_example".to_string(),
                "edit_id_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteFileset") => {
            let result = rt.block_on(client.delete_fileset(
                "editgroup_id_example".to_string(),
                "ident_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteFilesetEdit") => {
            let result = rt.block_on(client.delete_fileset_edit(
                "editgroup_id_example".to_string(),
                "edit_id_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteRelease") => {
            let result = rt.block_on(client.delete_release(
                "editgroup_id_example".to_string(),
                "ident_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteReleaseEdit") => {
            let result = rt.block_on(client.delete_release_edit(
                "editgroup_id_example".to_string(),
                "edit_id_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteWebcapture") => {
            let result = rt.block_on(client.delete_webcapture(
                "editgroup_id_example".to_string(),
                "ident_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteWebcaptureEdit") => {
            let result = rt.block_on(client.delete_webcapture_edit(
                "editgroup_id_example".to_string(),
                "edit_id_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteWork") => {
            let result = rt.block_on(client.delete_work(
                "editgroup_id_example".to_string(),
                "ident_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("DeleteWorkEdit") => {
            let result = rt.block_on(client.delete_work_edit(
                "editgroup_id_example".to_string(),
                "edit_id_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetChangelog") => {
            let result = rt.block_on(client.get_changelog(Some(789)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetChangelogEntry") => {
            let result = rt.block_on(client.get_changelog_entry(789));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetContainer") => {
            let result = rt.block_on(client.get_container(
                "ident_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetContainerEdit") => {
            let result = rt.block_on(client.get_container_edit("edit_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetContainerHistory") => {
            let result =
                rt.block_on(client.get_container_history("ident_example".to_string(), Some(789)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetContainerRedirects") => {
            let result = rt.block_on(client.get_container_redirects("ident_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetContainerRevision") => {
            let result = rt.block_on(client.get_container_revision(
                "rev_id_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetCreator") => {
            let result = rt.block_on(client.get_creator(
                "ident_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetCreatorEdit") => {
            let result = rt.block_on(client.get_creator_edit("edit_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetCreatorHistory") => {
            let result =
                rt.block_on(client.get_creator_history("ident_example".to_string(), Some(789)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetCreatorRedirects") => {
            let result = rt.block_on(client.get_creator_redirects("ident_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetCreatorReleases") => {
            let result = rt.block_on(client.get_creator_releases(
                "ident_example".to_string(),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetCreatorRevision") => {
            let result = rt.block_on(client.get_creator_revision(
                "rev_id_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetEditgroup") => {
            let result = rt.block_on(client.get_editgroup("editgroup_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetEditgroupAnnotations") => {
            let result = rt.block_on(client.get_editgroup_annotations(
                "editgroup_id_example".to_string(),
                Some("expand_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetEditgroupsReviewable") => {
            let result = rt.block_on(client.get_editgroups_reviewable(
                Some("expand_example".to_string()),
                Some(789),
                None,
                None,
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetEditor") => {
            let result = rt.block_on(client.get_editor("editor_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetEditorAnnotations") => {
            let result = rt.block_on(client.get_editor_annotations(
                "editor_id_example".to_string(),
                Some(789),
                None,
                None,
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetEditorEditgroups") => {
            let result = rt.block_on(client.get_editor_editgroups(
                "editor_id_example".to_string(),
                Some(789),
                None,
                None,
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFile") => {
            let result = rt.block_on(client.get_file(
                "ident_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFileEdit") => {
            let result = rt.block_on(client.get_file_edit("edit_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFileHistory") => {
            let result =
                rt.block_on(client.get_file_history("ident_example".to_string(), Some(789)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFileRedirects") => {
            let result = rt.block_on(client.get_file_redirects("ident_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFileRevision") => {
            let result = rt.block_on(client.get_file_revision(
                "rev_id_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFileset") => {
            let result = rt.block_on(client.get_fileset(
                "ident_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFilesetEdit") => {
            let result = rt.block_on(client.get_fileset_edit("edit_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFilesetHistory") => {
            let result =
                rt.block_on(client.get_fileset_history("ident_example".to_string(), Some(789)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFilesetRedirects") => {
            let result = rt.block_on(client.get_fileset_redirects("ident_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetFilesetRevision") => {
            let result = rt.block_on(client.get_fileset_revision(
                "rev_id_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetRelease") => {
            let result = rt.block_on(client.get_release(
                "ident_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetReleaseEdit") => {
            let result = rt.block_on(client.get_release_edit("edit_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetReleaseFiles") => {
            let result = rt.block_on(client.get_release_files(
                "ident_example".to_string(),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetReleaseFilesets") => {
            let result = rt.block_on(client.get_release_filesets(
                "ident_example".to_string(),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetReleaseHistory") => {
            let result =
                rt.block_on(client.get_release_history("ident_example".to_string(), Some(789)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetReleaseRedirects") => {
            let result = rt.block_on(client.get_release_redirects("ident_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetReleaseRevision") => {
            let result = rt.block_on(client.get_release_revision(
                "rev_id_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetReleaseWebcaptures") => {
            let result = rt.block_on(client.get_release_webcaptures(
                "ident_example".to_string(),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWebcapture") => {
            let result = rt.block_on(client.get_webcapture(
                "ident_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWebcaptureEdit") => {
            let result = rt.block_on(client.get_webcapture_edit("edit_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWebcaptureHistory") => {
            let result =
                rt.block_on(client.get_webcapture_history("ident_example".to_string(), Some(789)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWebcaptureRedirects") => {
            let result = rt.block_on(client.get_webcapture_redirects("ident_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWebcaptureRevision") => {
            let result = rt.block_on(client.get_webcapture_revision(
                "rev_id_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWork") => {
            let result = rt.block_on(client.get_work(
                "ident_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWorkEdit") => {
            let result = rt.block_on(client.get_work_edit("edit_id_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWorkHistory") => {
            let result =
                rt.block_on(client.get_work_history("ident_example".to_string(), Some(789)));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWorkRedirects") => {
            let result = rt.block_on(client.get_work_redirects("ident_example".to_string()));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWorkReleases") => {
            let result = rt.block_on(client.get_work_releases(
                "ident_example".to_string(),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("GetWorkRevision") => {
            let result = rt.block_on(client.get_work_revision(
                "rev_id_example".to_string(),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("LookupContainer") => {
            let result = rt.block_on(client.lookup_container(
                Some("issnl_example".to_string()),
                Some("wikidata_qid_example".to_string()),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("LookupCreator") => {
            let result = rt.block_on(client.lookup_creator(
                Some("orcid_example".to_string()),
                Some("wikidata_qid_example".to_string()),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("LookupFile") => {
            let result = rt.block_on(client.lookup_file(
                Some("md5_example".to_string()),
                Some("sha1_example".to_string()),
                Some("sha256_example".to_string()),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Some("LookupRelease") => {
            let result = rt.block_on(client.lookup_release(
                Some("doi_example".to_string()),
                Some("wikidata_qid_example".to_string()),
                Some("isbn13_example".to_string()),
                Some("pmid_example".to_string()),
                Some("pmcid_example".to_string()),
                Some("core_example".to_string()),
                Some("arxiv_example".to_string()),
                Some("jstor_example".to_string()),
                Some("ark_example".to_string()),
                Some("mag_example".to_string()),
                Some("expand_example".to_string()),
                Some("hide_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
        Some("UpdateContainer") => {
            let result = rt.block_on(client.update_container(
                  "editgroup_id_example".to_string(),
                  "ident_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("UpdateCreator") => {
            let result = rt.block_on(client.update_creator(
                  "editgroup_id_example".to_string(),
                  "ident_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("UpdateEditgroup") => {
            let result = rt.block_on(client.update_editgroup(
                  "editgroup_id_example".to_string(),
                  ???,
                  Some(true)
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("UpdateEditor") => {
            let result = rt.block_on(client.update_editor(
                  "editor_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("UpdateFile") => {
            let result = rt.block_on(client.update_file(
                  "editgroup_id_example".to_string(),
                  "ident_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("UpdateFileset") => {
            let result = rt.block_on(client.update_fileset(
                  "editgroup_id_example".to_string(),
                  "ident_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("UpdateRelease") => {
            let result = rt.block_on(client.update_release(
                  "editgroup_id_example".to_string(),
                  "ident_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("UpdateWebcapture") => {
            let result = rt.block_on(client.update_webcapture(
                  "editgroup_id_example".to_string(),
                  "ident_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("UpdateWork") => {
            let result = rt.block_on(client.update_work(
                  "editgroup_id_example".to_string(),
                  "ident_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        _ => panic!("Invalid operation provided"),
    }
}
