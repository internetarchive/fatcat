//! JSON Export Helper

#[macro_use]
extern crate clap;
extern crate dotenv;
#[macro_use]
extern crate error_chain;
extern crate fatcat;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde_json;
extern crate uuid;

use clap::{App, Arg, SubCommand};
use dotenv::dotenv;
use std::env;

use fatcat::errors::*;
use fatcat::api_helpers::FatCatId;
use std::str::FromStr;
use uuid::Uuid;

use error_chain::ChainedError;
//use std::io::{Stdout,StdoutLock};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};


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
        )
        .subcommand(
            SubCommand::with_name("revoke-tokens")
                .about("Resets auth_epoch for a single editor (invalidating all existing tokens)")
        )
        .subcommand(
            SubCommand::with_name("revoke-tokens-all")
                .about("Resets auth_epoch for all editors (invalidating tokens for all users!)")
        )
        .get_matches();

/*
        value_t_or_exit!(subm, "magic", u32)
        .after_help("Reads a ident table TSV dump from stdin (aka, ident_id, rev_id, redirect_id), \
            and outputs JSON (one entity per line). Database connection info read from environment \
            (DATABASE_URL, same as fatcatd).")
*/
    match m.subcommand() {
        ("list-editors", Some(_subm)) => {
            fatcat::auth::print_editors()?;
        },
        ("create-editor", Some(subm)) => {
            fatcat::auth::create_editor(
                subm.value_of("username").unwrap().to_string(),
                subm.is_present("admin"),
                subm.is_present("bot"))?;
        },
        ("create-token", Some(subm)) => {
            let editor_id = FatCatId::from_str(subm.value_of("editor").unwrap())?;
            fatcat::auth::create_token(editor_id, None)?;
        },
        ("inspect-token", Some(subm)) => {
            fatcat::auth::inspect_token(subm.value_of("token").unwrap())?;
        },
        ("revoke-tokens", Some(subm)) => {
            let editor_id = FatCatId::from_str(subm.value_of("editor").unwrap())?;
            fatcat::auth::revoke_tokens(editor_id)?;
        },
        ("revoke-tokens-everyone", Some(_subm)) => {
            fatcat::auth::revoke_tokens_everyone()?;
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
