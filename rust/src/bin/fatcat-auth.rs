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

use clap::{App, Arg, SubCommand};
use dotenv::dotenv;
use std::env;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use fatcat::ConnectionPool;
use fatcat::errors::*;
use fatcat::api_helpers::FatCatId;
use std::str::FromStr;
//use uuid::Uuid;

//use error_chain::ChainedError;
//use std::io::{Stdout,StdoutLock};
//use std::io::prelude::*;
//use std::io::{BufReader, BufWriter};


/// Instantiate a new API server with a pooled database connection
// TODO: copypasta from fatcat-export
pub fn database_worker_pool() -> Result<ConnectionPool> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.");
    Ok(pool)
}

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
                     --env-format 'outputs in a format that shells can source'"
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

    let db_conn = database_worker_pool()?.get().expect("database pool");
    match m.subcommand() {
        ("list-editors", Some(_subm)) => {
            fatcat::auth::print_editors(&db_conn)?;
        },
        ("create-editor", Some(subm)) => {
            let editor = fatcat::auth::create_editor(
                &db_conn,
                subm.value_of("username").unwrap().to_string(),
                subm.is_present("admin"),
                subm.is_present("bot"))?;
            //println!("{:?}", editor);
            println!("{}", FatCatId::from_uuid(&editor.id).to_string());
        },
        ("create-token", Some(subm)) => {
            let editor_id = FatCatId::from_str(subm.value_of("editor-id").unwrap())?;
            println!("{}", fatcat::auth::create_token(&db_conn, editor_id, None)?);
        },
        ("inspect-token", Some(subm)) => {
            fatcat::auth::inspect_token(&db_conn, subm.value_of("token").unwrap())?;
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
