#![allow(missing_docs, unused_variables, trivial_casts)]

extern crate clap;
extern crate fatcat;
#[allow(unused_extern_crates)]
extern crate futures;
#[allow(unused_extern_crates)]
extern crate swagger;
#[allow(unused_extern_crates)]
extern crate uuid;

use clap::{App, Arg};
#[allow(unused_imports)]
use fatcat::{
    AcceptEditgroupResponse, ApiError, ApiNoContext, AuthCheckResponse, AuthOidcResponse, ContextWrapperExt, CreateContainerAutoBatchResponse, CreateContainerResponse, CreateCreatorAutoBatchResponse,
    CreateCreatorResponse, CreateEditgroupAnnotationResponse, CreateEditgroupResponse, CreateFileAutoBatchResponse, CreateFileResponse, CreateFilesetAutoBatchResponse, CreateFilesetResponse,
    CreateReleaseAutoBatchResponse, CreateReleaseResponse, CreateWebcaptureAutoBatchResponse, CreateWebcaptureResponse, CreateWorkAutoBatchResponse, CreateWorkResponse, DeleteContainerEditResponse,
    DeleteContainerResponse, DeleteCreatorEditResponse, DeleteCreatorResponse, DeleteFileEditResponse, DeleteFileResponse, DeleteFilesetEditResponse, DeleteFilesetResponse, DeleteReleaseEditResponse,
    DeleteReleaseResponse, DeleteWebcaptureEditResponse, DeleteWebcaptureResponse, DeleteWorkEditResponse, DeleteWorkResponse, GetChangelogEntryResponse, GetChangelogResponse,
    GetContainerEditResponse, GetContainerHistoryResponse, GetContainerRedirectsResponse, GetContainerResponse, GetContainerRevisionResponse, GetCreatorEditResponse, GetCreatorHistoryResponse,
    GetCreatorRedirectsResponse, GetCreatorReleasesResponse, GetCreatorResponse, GetCreatorRevisionResponse, GetEditgroupAnnotationsResponse, GetEditgroupResponse, GetEditgroupsReviewableResponse,
    GetEditorAnnotationsResponse, GetEditorEditgroupsResponse, GetEditorResponse, GetFileEditResponse, GetFileHistoryResponse, GetFileRedirectsResponse, GetFileResponse, GetFileRevisionResponse,
    GetFilesetEditResponse, GetFilesetHistoryResponse, GetFilesetRedirectsResponse, GetFilesetResponse, GetFilesetRevisionResponse, GetReleaseEditResponse, GetReleaseFilesResponse,
    GetReleaseFilesetsResponse, GetReleaseHistoryResponse, GetReleaseRedirectsResponse, GetReleaseResponse, GetReleaseRevisionResponse, GetReleaseWebcapturesResponse, GetWebcaptureEditResponse,
    GetWebcaptureHistoryResponse, GetWebcaptureRedirectsResponse, GetWebcaptureResponse, GetWebcaptureRevisionResponse, GetWorkEditResponse, GetWorkHistoryResponse, GetWorkRedirectsResponse,
    GetWorkReleasesResponse, GetWorkResponse, GetWorkRevisionResponse, LookupContainerResponse, LookupCreatorResponse, LookupFileResponse, LookupReleaseResponse, UpdateContainerResponse,
    UpdateCreatorResponse, UpdateEditgroupResponse, UpdateEditorResponse, UpdateFileResponse, UpdateFilesetResponse, UpdateReleaseResponse, UpdateWebcaptureResponse, UpdateWorkResponse,
};
#[allow(unused_imports)]
use futures::{future, stream, Future, Stream};

fn main() {
    let matches = App::new("client")
        .arg(
            Arg::with_name("operation")
                .help("Sets the operation to run")
                .possible_values(&[
                    "DeleteContainer",
                    "DeleteContainerEdit",
                    "GetContainer",
                    "GetContainerEdit",
                    "GetContainerHistory",
                    "GetContainerRedirects",
                    "GetContainerRevision",
                    "LookupContainer",
                    "DeleteCreator",
                    "DeleteCreatorEdit",
                    "GetCreator",
                    "GetCreatorEdit",
                    "GetCreatorHistory",
                    "GetCreatorRedirects",
                    "GetCreatorReleases",
                    "GetCreatorRevision",
                    "LookupCreator",
                    "AuthCheck",
                    "GetEditgroupsReviewable",
                    "GetEditor",
                    "GetEditorEditgroups",
                    "AcceptEditgroup",
                    "GetChangelog",
                    "GetChangelogEntry",
                    "GetEditgroup",
                    "GetEditgroupAnnotations",
                    "GetEditorAnnotations",
                    "DeleteFile",
                    "DeleteFileEdit",
                    "GetFile",
                    "GetFileEdit",
                    "GetFileHistory",
                    "GetFileRedirects",
                    "GetFileRevision",
                    "LookupFile",
                    "DeleteFileset",
                    "DeleteFilesetEdit",
                    "GetFileset",
                    "GetFilesetEdit",
                    "GetFilesetHistory",
                    "GetFilesetRedirects",
                    "GetFilesetRevision",
                    "DeleteRelease",
                    "DeleteReleaseEdit",
                    "GetRelease",
                    "GetReleaseEdit",
                    "GetReleaseFiles",
                    "GetReleaseFilesets",
                    "GetReleaseHistory",
                    "GetReleaseRedirects",
                    "GetReleaseRevision",
                    "GetReleaseWebcaptures",
                    "LookupRelease",
                    "DeleteWebcapture",
                    "DeleteWebcaptureEdit",
                    "GetWebcapture",
                    "GetWebcaptureEdit",
                    "GetWebcaptureHistory",
                    "GetWebcaptureRedirects",
                    "GetWebcaptureRevision",
                    "DeleteWork",
                    "DeleteWorkEdit",
                    "GetWork",
                    "GetWorkEdit",
                    "GetWorkHistory",
                    "GetWorkRedirects",
                    "GetWorkReleases",
                    "GetWorkRevision",
                ])
                .required(true)
                .index(1),
        )
        .arg(Arg::with_name("https").long("https").help("Whether to use HTTPS or not"))
        .arg(Arg::with_name("host").long("host").takes_value(true).default_value("api.fatcat.wiki").help("Hostname to contact"))
        .arg(Arg::with_name("port").long("port").takes_value(true).default_value("8080").help("Port to contact"))
        .get_matches();

    let is_https = matches.is_present("https");
    let base_url = format!(
        "{}://{}:{}",
        if is_https { "https" } else { "http" },
        matches.value_of("host").unwrap(),
        matches.value_of("port").unwrap()
    );
    let client = if is_https {
        // Using Simple HTTPS
        fatcat::Client::try_new_https(&base_url, "examples/ca.pem").expect("Failed to create HTTPS client")
    } else {
        // Using HTTP
        fatcat::Client::try_new_http(&base_url).expect("Failed to create HTTP client")
    };

    // Using a non-default `Context` is not required; this is just an example!
    let client = client.with_context(fatcat::Context::new_with_span_id(self::uuid::Uuid::new_v4().to_string()));

    match matches.value_of("operation") {
        // Disabled because there's no example.
        // Some("CreateContainer") => {
        //     let result = client.create_container("editgroup_id_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateContainerAutoBatch") => {
        //     let result = client.create_container_auto_batch(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("DeleteContainer") => {
            let result = client.delete_container("editgroup_id_example".to_string(), "ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("DeleteContainerEdit") => {
            let result = client.delete_container_edit("editgroup_id_example".to_string(), "edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetContainer") => {
            let result = client
                .get_container("ident_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetContainerEdit") => {
            let result = client.get_container_edit("edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetContainerHistory") => {
            let result = client.get_container_history("ident_example".to_string(), Some(789)).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetContainerRedirects") => {
            let result = client.get_container_redirects("ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetContainerRevision") => {
            let result = client
                .get_container_revision("rev_id_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("LookupContainer") => {
            let result = client
                .lookup_container(
                    Some("issnl_example".to_string()),
                    Some("wikidata_qid_example".to_string()),
                    Some("expand_example".to_string()),
                    Some("hide_example".to_string()),
                )
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("UpdateContainer") => {
        //     let result = client.update_container("editgroup_id_example".to_string(), "ident_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateCreator") => {
        //     let result = client.create_creator("editgroup_id_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateCreatorAutoBatch") => {
        //     let result = client.create_creator_auto_batch(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("DeleteCreator") => {
            let result = client.delete_creator("editgroup_id_example".to_string(), "ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("DeleteCreatorEdit") => {
            let result = client.delete_creator_edit("editgroup_id_example".to_string(), "edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetCreator") => {
            let result = client
                .get_creator("ident_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetCreatorEdit") => {
            let result = client.get_creator_edit("edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetCreatorHistory") => {
            let result = client.get_creator_history("ident_example".to_string(), Some(789)).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetCreatorRedirects") => {
            let result = client.get_creator_redirects("ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetCreatorReleases") => {
            let result = client.get_creator_releases("ident_example".to_string(), Some("hide_example".to_string())).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetCreatorRevision") => {
            let result = client
                .get_creator_revision("rev_id_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("LookupCreator") => {
            let result = client
                .lookup_creator(
                    Some("orcid_example".to_string()),
                    Some("wikidata_qid_example".to_string()),
                    Some("expand_example".to_string()),
                    Some("hide_example".to_string()),
                )
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("UpdateCreator") => {
        //     let result = client.update_creator("editgroup_id_example".to_string(), "ident_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("AuthCheck") => {
            let result = client.auth_check(Some("role_example".to_string())).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("AuthOidc") => {
        //     let result = client.auth_oidc(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("GetEditgroupsReviewable") => {
            let result = client.get_editgroups_reviewable(Some("expand_example".to_string()), Some(789), None, None).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetEditor") => {
            let result = client.get_editor("editor_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetEditorEditgroups") => {
            let result = client.get_editor_editgroups("editor_id_example".to_string(), Some(789), None, None).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("UpdateEditgroup") => {
        //     let result = client.update_editgroup("editgroup_id_example".to_string(), ???, Some(true)).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("UpdateEditor") => {
        //     let result = client.update_editor("editor_id_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("AcceptEditgroup") => {
            let result = client.accept_editgroup("editgroup_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("CreateEditgroup") => {
        //     let result = client.create_editgroup(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateEditgroupAnnotation") => {
        //     let result = client.create_editgroup_annotation("editgroup_id_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("GetChangelog") => {
            let result = client.get_changelog(Some(789)).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetChangelogEntry") => {
            let result = client.get_changelog_entry(789).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetEditgroup") => {
            let result = client.get_editgroup("editgroup_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetEditgroupAnnotations") => {
            let result = client.get_editgroup_annotations("editgroup_id_example".to_string(), Some("expand_example".to_string())).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetEditorAnnotations") => {
            let result = client.get_editor_annotations("editor_id_example".to_string(), Some(789), None, None).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("CreateFile") => {
        //     let result = client.create_file("editgroup_id_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateFileAutoBatch") => {
        //     let result = client.create_file_auto_batch(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("DeleteFile") => {
            let result = client.delete_file("editgroup_id_example".to_string(), "ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("DeleteFileEdit") => {
            let result = client.delete_file_edit("editgroup_id_example".to_string(), "edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFile") => {
            let result = client
                .get_file("ident_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFileEdit") => {
            let result = client.get_file_edit("edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFileHistory") => {
            let result = client.get_file_history("ident_example".to_string(), Some(789)).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFileRedirects") => {
            let result = client.get_file_redirects("ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFileRevision") => {
            let result = client
                .get_file_revision("rev_id_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("LookupFile") => {
            let result = client
                .lookup_file(
                    Some("md5_example".to_string()),
                    Some("sha1_example".to_string()),
                    Some("sha256_example".to_string()),
                    Some("expand_example".to_string()),
                    Some("hide_example".to_string()),
                )
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("UpdateFile") => {
        //     let result = client.update_file("editgroup_id_example".to_string(), "ident_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateFileset") => {
        //     let result = client.create_fileset("editgroup_id_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateFilesetAutoBatch") => {
        //     let result = client.create_fileset_auto_batch(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("DeleteFileset") => {
            let result = client.delete_fileset("editgroup_id_example".to_string(), "ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("DeleteFilesetEdit") => {
            let result = client.delete_fileset_edit("editgroup_id_example".to_string(), "edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFileset") => {
            let result = client
                .get_fileset("ident_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFilesetEdit") => {
            let result = client.get_fileset_edit("edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFilesetHistory") => {
            let result = client.get_fileset_history("ident_example".to_string(), Some(789)).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFilesetRedirects") => {
            let result = client.get_fileset_redirects("ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFilesetRevision") => {
            let result = client
                .get_fileset_revision("rev_id_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("UpdateFileset") => {
        //     let result = client.update_fileset("editgroup_id_example".to_string(), "ident_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateRelease") => {
        //     let result = client.create_release("editgroup_id_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateReleaseAutoBatch") => {
        //     let result = client.create_release_auto_batch(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateWork") => {
        //     let result = client.create_work("editgroup_id_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("DeleteRelease") => {
            let result = client.delete_release("editgroup_id_example".to_string(), "ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("DeleteReleaseEdit") => {
            let result = client.delete_release_edit("editgroup_id_example".to_string(), "edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetRelease") => {
            let result = client
                .get_release("ident_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetReleaseEdit") => {
            let result = client.get_release_edit("edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetReleaseFiles") => {
            let result = client.get_release_files("ident_example".to_string(), Some("hide_example".to_string())).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetReleaseFilesets") => {
            let result = client.get_release_filesets("ident_example".to_string(), Some("hide_example".to_string())).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetReleaseHistory") => {
            let result = client.get_release_history("ident_example".to_string(), Some(789)).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetReleaseRedirects") => {
            let result = client.get_release_redirects("ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetReleaseRevision") => {
            let result = client
                .get_release_revision("rev_id_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetReleaseWebcaptures") => {
            let result = client.get_release_webcaptures("ident_example".to_string(), Some("hide_example".to_string())).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("LookupRelease") => {
            let result = client
                .lookup_release(
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
                )
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("UpdateRelease") => {
        //     let result = client.update_release("editgroup_id_example".to_string(), "ident_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateWebcapture") => {
        //     let result = client.create_webcapture("editgroup_id_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateWebcaptureAutoBatch") => {
        //     let result = client.create_webcapture_auto_batch(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("DeleteWebcapture") => {
            let result = client.delete_webcapture("editgroup_id_example".to_string(), "ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("DeleteWebcaptureEdit") => {
            let result = client.delete_webcapture_edit("editgroup_id_example".to_string(), "edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWebcapture") => {
            let result = client
                .get_webcapture("ident_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWebcaptureEdit") => {
            let result = client.get_webcapture_edit("edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWebcaptureHistory") => {
            let result = client.get_webcapture_history("ident_example".to_string(), Some(789)).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWebcaptureRedirects") => {
            let result = client.get_webcapture_redirects("ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWebcaptureRevision") => {
            let result = client
                .get_webcapture_revision("rev_id_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("UpdateWebcapture") => {
        //     let result = client.update_webcapture("editgroup_id_example".to_string(), "ident_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateWorkAutoBatch") => {
        //     let result = client.create_work_auto_batch(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("DeleteWork") => {
            let result = client.delete_work("editgroup_id_example".to_string(), "ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("DeleteWorkEdit") => {
            let result = client.delete_work_edit("editgroup_id_example".to_string(), "edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWork") => {
            let result = client
                .get_work("ident_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWorkEdit") => {
            let result = client.get_work_edit("edit_id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWorkHistory") => {
            let result = client.get_work_history("ident_example".to_string(), Some(789)).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWorkRedirects") => {
            let result = client.get_work_redirects("ident_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWorkReleases") => {
            let result = client.get_work_releases("ident_example".to_string(), Some("hide_example".to_string())).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWorkRevision") => {
            let result = client
                .get_work_revision("rev_id_example".to_string(), Some("expand_example".to_string()), Some("hide_example".to_string()))
                .wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("UpdateWork") => {
        //     let result = client.update_work("editgroup_id_example".to_string(), "ident_example".to_string(), ???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        _ => panic!("Invalid operation provided"),
    }
}
