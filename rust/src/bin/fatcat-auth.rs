//! JSON Export Helper

use clap::{App, SubCommand};

use fatcat::editing_crud::EditorCrud;
use fatcat::errors::Result;
use fatcat::identifiers::FatcatId;
use fatcat::{auth, server};
use fatcat_api_spec::models::Editor;
use std::process;
use std::str::FromStr;

fn main() -> Result<()> {
    let m = App::new("fatcat-auth")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Bryan Newbold <bnewbold@archive.org>")
        .about("Editor authentication admin tool")
        .subcommand(
            SubCommand::with_name("list-editors").about("Prints all currently registered editors"),
        )
        .subcommand(
            SubCommand::with_name("create-editor")
                .about("Creates a new auth token (macaroon) for the given editor")
                .args_from_usage(
                    "<username> 'username for editor'
                     --admin 'creates editor with admin privs'
                     --bot 'this editor is a bot'",
                ),
        )
        .subcommand(
            SubCommand::with_name("create-token")
                .about("Creates a new auth token (macaroon) for the given editor")
                .args_from_usage(
                    "<editor-id> 'id of the editor (fatcatid, not username)'
                     --env-format 'outputs in a format that shells can source'", // TODO
                ),
        )
        .subcommand(
            SubCommand::with_name("inspect-token")
                .about("Dumps token metadata (and whether it is valid)")
                .args_from_usage("<token> 'base64-encoded token (macaroon)'"),
        )
        .subcommand(
            SubCommand::with_name("create-key")
                .about("Creates a new auth secret key (aka, root/signing key for tokens)")
                .args_from_usage(
                    "--env-format 'outputs in a format that shells can source'", // TODO
                ),
        )
        .subcommand(
            SubCommand::with_name("revoke-tokens")
                .about("Resets auth_epoch for a single editor (invalidating all existing tokens)")
                .args_from_usage("<editor-id> 'identifier (fcid) of editor'"),
        )
        .subcommand(
            SubCommand::with_name("revoke-tokens-everyone")
                .about("Resets auth_epoch for all editors (invalidating tokens for all users!)"),
        )
        .get_matches();

    // First, the commands with no db or confectionary needed
    match m.subcommand() {
        ("create-key", Some(_subm)) => {
            println!("{}", fatcat::auth::create_key());
            return Ok(());
        }
        _ => (),
    }

    // Then the ones that do
    let db_conn = server::database_worker_pool()?
        .get()
        .expect("database pool");
    let confectionary = auth::env_confectionary()?;
    match m.subcommand() {
        ("list-editors", Some(_subm)) => {
            fatcat::auth::print_editors(&db_conn)?;
        }
        ("create-editor", Some(subm)) => {
            let editor = Editor {
                editor_id: None,
                username: subm.value_of("username").unwrap().to_string(),
                is_admin: Some(subm.is_present("admin")),
                is_bot: Some(subm.is_present("bot")),
                is_active: Some(true),
            };
            let editor_row = editor.db_create(&db_conn)?;
            //println!("{:?}", editor);
            println!("{}", FatcatId::from_uuid(&editor_row.id).to_string());
        }
        ("create-token", Some(subm)) => {
            let editor_id = FatcatId::from_str(subm.value_of("editor-id").unwrap())?;
            println!("{}", confectionary.create_token(editor_id, None)?);
        }
        ("inspect-token", Some(subm)) => {
            confectionary.inspect_token(&db_conn, subm.value_of("token").unwrap())?;
        }
        ("revoke-tokens", Some(subm)) => {
            let editor_id = FatcatId::from_str(subm.value_of("editor-id").unwrap())?;
            fatcat::auth::revoke_tokens(&db_conn, editor_id)?;
            println!("success!");
        }
        ("revoke-tokens-everyone", Some(_subm)) => {
            fatcat::auth::revoke_tokens_everyone(&db_conn)?;
            println!("success!");
        }
        _ => {
            println!("Missing or unimplemented command!");
            println!("{}", m.usage());
            process::exit(-1);
        }
    }
    Ok(())
}
