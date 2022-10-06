//! Database cleanup tool

use clap::{value_t_or_exit, App, SubCommand};

use fatcat::database_models::*;
use fatcat::database_schema::*;
use fatcat::errors::Result;
use fatcat::identifiers::FatcatId;
use fatcat::server;
use fatcat::server::DbConn;
use std::process;
use std::str::FromStr;

use diesel::prelude::*;

fn backfill_changelog_gap(conn: &DbConn, last_good: i64, max_index: i64) -> Result<()> {
    // sanity check arguments against database
    assert!(last_good > 0);
    assert!(max_index > 0);
    assert!(last_good < max_index);
    let highest_row: ChangelogRow = changelog::table.order(changelog::id.desc()).first(conn)?;
    assert!(highest_row.id >= max_index);

    // default values
    // 'root' editor_id is aaaaaaaaaaaabkvkaaaaaaaaae
    // 'admin' editor_id is aaaaaaaaaaaabkvkaaaaaaaaai
    let editor_id = FatcatId::from_str("aaaaaaaaaaaabkvkaaaaaaaaai").unwrap();
    let description = "Backfill of missing changelog entries due to database id gap";

    // fetch the last entry before the gap, to re-use the timestamp
    let existing_row: ChangelogRow = changelog::table.find(last_good).first(conn)?;

    for index in last_good + 1..max_index + 1 {
        // ensure this index is actually a gap
        let count: i64 = changelog::table
            .filter(changelog::id.eq(index))
            .count()
            .get_result(conn)?;
        if count != 0 {
            println!("Found existing changelog: {}", index);
            return Ok(());
        }

        // create dummy empty editgroup, then add a changelog entry
        let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
            .values((
                editgroup::editor_id.eq(editor_id.to_uuid()),
                editgroup::created.eq(existing_row.timestamp),
                editgroup::is_accepted.eq(true),
                editgroup::description.eq(Some(description)),
            ))
            .get_result(conn)?;
        let _entry_row: ChangelogRow = diesel::insert_into(changelog::table)
            .values((
                changelog::id.eq(index),
                changelog::editgroup_id.eq(eg_row.id),
                changelog::timestamp.eq(existing_row.timestamp),
            ))
            .get_result(conn)?;
        println!("Inserted changelog: {}", index);
    }
    Ok(())
}

fn main() -> Result<()> {
    let m = App::new("fatcat-doctor")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Bryan Newbold <bnewbold@archive.org>")
        .about("Database cleanup / fixup tool")
        .subcommand(
            SubCommand::with_name("backfill-changelog-gap")
                .about("Inserts dummy changelog entries and editgroups for gap")
                .args_from_usage(
                    "<start> 'changelog index of entry just before gap'
                    <end> 'highest changelog index to backfill'",
                ),
        )
        .get_matches();

    let db_conn = server::database_worker_pool()?
        .get()
        .expect("database pool");
    match m.subcommand() {
        ("backfill-changelog-gap", Some(subm)) => {
            let last_good: i64 = value_t_or_exit!(subm.value_of("start"), i64);
            let max_index: i64 = value_t_or_exit!(subm.value_of("end"), i64);
            backfill_changelog_gap(&db_conn, last_good, max_index)?;
        }
        _ => {
            println!("Missing or unimplemented command!");
            println!("{}", m.usage());
            process::exit(-1);
        }
    }
    Ok(())
}
