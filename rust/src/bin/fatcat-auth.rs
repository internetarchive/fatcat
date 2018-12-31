//! JSON Export Helper

//#[macro_use]
extern crate clap;
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate error_chain;
extern crate fatcat;
//#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde_json;
extern crate uuid;

use clap::{App, SubCommand};

use diesel::prelude::*;
use fatcat::errors::*;
use fatcat::api_helpers::FatCatId;
use std::str::FromStr;
//use uuid::Uuid;

//use error_chain::ChainedError;
//use std::io::{Stdout,StdoutLock};
//use std::io::prelude::*;
//use std::io::{BufReader, BufWriter};


fn run() -> Result<()> {
    let m = App::new("fatcat-auth")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Bryan Newbold <bnewbold@archive.org>")
        .about("Editor authentication admin tool")
        .subcommand(
            SubCommand::with_name("list-editors")
                .about("Prints all currently registered editors")
        )
        .subcommand(
            SubCommand::with_name("create-editor")
                .about("Creates a new auth token (macaroon) for the given editor")
                .args_from_usage(
                    "<username> 'username for editor'
                     --admin 'creates editor with admin privs'
                     --bot 'this editor is a bot'"
                )
        )
        .subcommand(
            SubCommand::with_name("create-token")
                .about("Creates a new auth token (macaroon) for the given editor")
                .args_from_usage(
                    "<editor-id> 'id of the editor (fatcatid, not username)'
                     --env-format 'outputs in a format that shells can source'" // TODO
                )
        )
        .subcommand(
            SubCommand::with_name("inspect-token")
                .about("Dumps token metadata (and whether it is valid)")
                .args_from_usage(
                    "<token> 'base64-encoded token (macaroon)'"
                )
        )
        .subcommand(
            SubCommand::with_name("create-key")
                .about("Creates a new auth secret key (aka, root/signing key for tokens)")
                .args_from_usage(
                    "--env-format 'outputs in a format that shells can source'" // TODO
                )
        )
        .subcommand(
            SubCommand::with_name("revoke-tokens")
                .about("Resets auth_epoch for a single editor (invalidating all existing tokens)")
                .args_from_usage(
                    "<editor-id> 'identifier (fcid) of editor'"
                )
        )
        .subcommand(
            SubCommand::with_name("revoke-tokens-everyone")
                .about("Resets auth_epoch for all editors (invalidating tokens for all users!)")
        )
        .get_matches();

    // First, the commands with no db or confectionary needed
    match m.subcommand() {
        ("create-key", Some(_subm)) => {
            println!("{}", fatcat::auth::create_key());
            return Ok(())
        },
        _ => (),
    }

    // Then the ones that do
    let db_conn = fatcat::database_worker_pool()?.get().expect("database pool");
    let confectionary = fatcat::env_confectionary()?;
    match m.subcommand() {
        ("list-editors", Some(_subm)) => {
            fatcat::auth::print_editors(&db_conn)?;
        },
        ("create-editor", Some(subm)) => {
            let editor = fatcat::api_helpers::create_editor(
                &db_conn,
                subm.value_of("username").unwrap().to_string(),
                subm.is_present("admin"),
                subm.is_present("bot"))?;
            //println!("{:?}", editor);
            println!("{}", FatCatId::from_uuid(&editor.id).to_string());
        },
        ("create-token", Some(subm)) => {
            let editor_id = FatCatId::from_str(subm.value_of("editor-id").unwrap())?;
            // check that editor exists
            let _ed: fatcat::database_models::EditorRow = fatcat::database_schema::editor::table
                .find(&editor_id.to_uuid())
                .get_result(&db_conn)?;
            println!("{}", confectionary.create_token(editor_id, None)?);
        },
        ("inspect-token", Some(subm)) => {
            confectionary.inspect_token(&db_conn, subm.value_of("token").unwrap())?;
        },
        ("revoke-tokens", Some(subm)) => {
            let editor_id = FatCatId::from_str(subm.value_of("editor-id").unwrap())?;
            fatcat::auth::revoke_tokens(&db_conn, editor_id)?;
            println!("success!");
        },
        ("revoke-tokens-everyone", Some(_subm)) => {
            fatcat::auth::revoke_tokens_everyone(&db_conn)?;
            println!("success!");
        },
        _ => {
            println!("Missing or unimplemented command!");
            println!("{}", m.usage());
            ::std::process::exit(-1);
        }
    }
    Ok(())
}

quick_main!(run);
