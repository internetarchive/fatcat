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
use fatcat::{ApiError, ApiNoContext, ContainerIdGetResponse, ContainerLookupGetResponse, ContainerPostResponse, ContextWrapperExt, CreatorIdGetResponse, CreatorLookupGetResponse,
             CreatorPostResponse, EditgroupIdAcceptPostResponse, EditgroupIdGetResponse, EditgroupPostResponse, EditorUsernameChangelogGetResponse, EditorUsernameGetResponse, FileIdGetResponse,
             FileLookupGetResponse, FilePostResponse, ReleaseIdGetResponse, ReleaseLookupGetResponse, ReleasePostResponse, WorkIdGetResponse, WorkPostResponse};
#[allow(unused_imports)]
use futures::{future, stream, Future, Stream};

fn main() {
    let matches = App::new("client")
        .arg(
            Arg::with_name("operation")
                .help("Sets the operation to run")
                .possible_values(&[
                    "ContainerIdGet",
                    "ContainerLookupGet",
                    "ContainerPost",
                    "CreatorIdGet",
                    "CreatorLookupGet",
                    "CreatorPost",
                    "EditgroupIdAcceptPost",
                    "EditgroupIdGet",
                    "EditgroupPost",
                    "EditorUsernameChangelogGet",
                    "EditorUsernameGet",
                    "FileIdGet",
                    "FileLookupGet",
                    "FilePost",
                    "ReleaseIdGet",
                    "ReleaseLookupGet",
                    "ReleasePost",
                    "WorkIdGet",
                    "WorkPost",
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
        Some("ContainerIdGet") => {
            let result = client.container_id_get("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("ContainerLookupGet") => {
            let result = client.container_lookup_get("issn_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("ContainerPost") => {
            let result = client.container_post(None).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("CreatorIdGet") => {
            let result = client.creator_id_get("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("CreatorLookupGet") => {
            let result = client.creator_lookup_get("orcid_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("CreatorPost") => {
            let result = client.creator_post(None).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("EditgroupIdAcceptPost") => {
            let result = client.editgroup_id_accept_post(56).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("EditgroupIdGet") => {
            let result = client.editgroup_id_get(56).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("EditgroupPost") => {
            let result = client.editgroup_post().wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("EditorUsernameChangelogGet") => {
            let result = client.editor_username_changelog_get("username_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("EditorUsernameGet") => {
            let result = client.editor_username_get("username_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("FileIdGet") => {
            let result = client.file_id_get("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("FileLookupGet") => {
            let result = client.file_lookup_get("sha1_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("FilePost") => {
            let result = client.file_post(None).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("ReleaseIdGet") => {
            let result = client.release_id_get("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("ReleaseLookupGet") => {
            let result = client.release_lookup_get("doi_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("ReleasePost") => {
            let result = client.release_post(None).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("WorkIdGet") => {
            let result = client.work_id_get("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("WorkPost") => {
            let result = client.work_post(None).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        _ => panic!("Invalid operation provided"),
    }
}
