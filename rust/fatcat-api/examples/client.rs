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
use fatcat::{AcceptEditgroupResponse, ApiError, ApiNoContext, ContextWrapperExt, CreateContainerBatchResponse, CreateContainerResponse, CreateCreatorBatchResponse, CreateCreatorResponse,
             CreateEditgroupResponse, CreateFileBatchResponse, CreateFileResponse, CreateReleaseBatchResponse, CreateReleaseResponse, CreateWorkBatchResponse, CreateWorkResponse,
             GetContainerResponse, GetCreatorReleasesResponse, GetCreatorResponse, GetEditgroupResponse, GetEditorChangelogResponse, GetEditorResponse, GetFileResponse, GetReleaseFilesResponse,
             GetReleaseResponse, GetWorkReleasesResponse, GetWorkResponse, LookupContainerResponse, LookupCreatorResponse, LookupFileResponse, LookupReleaseResponse};
#[allow(unused_imports)]
use futures::{future, stream, Future, Stream};

fn main() {
    let matches = App::new("client")
        .arg(
            Arg::with_name("operation")
                .help("Sets the operation to run")
                .possible_values(&[
                    "AcceptEditgroup",
                    "CreateContainerBatch",
                    "CreateCreatorBatch",
                    "CreateFileBatch",
                    "CreateReleaseBatch",
                    "CreateWorkBatch",
                    "GetContainer",
                    "GetCreator",
                    "GetCreatorReleases",
                    "GetEditgroup",
                    "GetEditor",
                    "GetEditorChangelog",
                    "GetFile",
                    "GetRelease",
                    "GetReleaseFiles",
                    "GetWork",
                    "GetWorkReleases",
                    "LookupContainer",
                    "LookupCreator",
                    "LookupFile",
                    "LookupRelease",
                ])
                .required(true)
                .index(1),
        )
        .arg(Arg::with_name("https").long("https").help("Whether to use HTTPS or not"))
        .arg(Arg::with_name("host").long("host").takes_value(true).default_value("localhost").help("Hostname to contact"))
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
        Some("AcceptEditgroup") => {
            let result = client.accept_editgroup(789).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("CreateContainer") => {
        //     let result = client.create_container(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("CreateContainerBatch") => {
            let result = client.create_container_batch(&Vec::new()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("CreateCreator") => {
        //     let result = client.create_creator(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("CreateCreatorBatch") => {
            let result = client.create_creator_batch(&Vec::new()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("CreateEditgroup") => {
        //     let result = client.create_editgroup(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },

        // Disabled because there's no example.
        // Some("CreateFile") => {
        //     let result = client.create_file(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("CreateFileBatch") => {
            let result = client.create_file_batch(&Vec::new()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("CreateRelease") => {
        //     let result = client.create_release(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("CreateReleaseBatch") => {
            let result = client.create_release_batch(&Vec::new()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        // Disabled because there's no example.
        // Some("CreateWork") => {
        //     let result = client.create_work(???).wait();
        //     println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        //  },
        Some("CreateWorkBatch") => {
            let result = client.create_work_batch(&Vec::new()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetContainer") => {
            let result = client.get_container("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetCreator") => {
            let result = client.get_creator("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetCreatorReleases") => {
            let result = client.get_creator_releases("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetEditgroup") => {
            let result = client.get_editgroup(789).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetEditor") => {
            let result = client.get_editor("username_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetEditorChangelog") => {
            let result = client.get_editor_changelog("username_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetFile") => {
            let result = client.get_file("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetRelease") => {
            let result = client.get_release("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetReleaseFiles") => {
            let result = client.get_release_files("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWork") => {
            let result = client.get_work("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("GetWorkReleases") => {
            let result = client.get_work_releases("id_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("LookupContainer") => {
            let result = client.lookup_container("issnl_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("LookupCreator") => {
            let result = client.lookup_creator("orcid_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("LookupFile") => {
            let result = client.lookup_file("sha1_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        Some("LookupRelease") => {
            let result = client.lookup_release("doi_example".to_string()).wait();
            println!("{:?} (X-Span-ID: {:?})", result, client.context().x_span_id.clone().unwrap_or(String::from("<none>")));
        }

        _ => panic!("Invalid operation provided"),
    }
}
