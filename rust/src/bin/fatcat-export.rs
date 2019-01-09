//! JSON Export Helper

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;

use clap::{App, Arg};

use fatcat::entity_crud::*;
use fatcat::errors::*;
use fatcat::identifiers::*;
use fatcat::server::*;
use fatcat_api_spec::models::*;
use std::str::FromStr;
use uuid::Uuid;

use error_chain::ChainedError;
use std::thread;
//use std::io::{Stdout,StdoutLock};
use crossbeam_channel as channel;
//use num_cpus; TODO:
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

const CHANNEL_BUFFER_LEN: usize = 200;

arg_enum! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub enum ExportEntityType {
        Creator,
        Container,
        File,
        Fileset,
        Webcapture,
        Release,
        Work
    }
}

struct IdentRow {
    ident_id: FatCatId,
    rev_id: Option<Uuid>,
    redirect_id: Option<FatCatId>,
}

macro_rules! generic_loop_work {
    ($fn_name:ident, $entity_model:ident) => {
        fn $fn_name(
            row_receiver: channel::Receiver<IdentRow>,
            output_sender: channel::Sender<String>,
            db_conn: &DbConn,
            expand: Option<ExpandFlags>,
        ) {
            let result: Result<()> = (|| {
                for row in row_receiver {
                    let mut entity = $entity_model::db_get_rev(
                        db_conn,
                        row.rev_id.expect("valid, non-deleted row"),
                        HideFlags::none(),
                    )
                    .chain_err(|| "reading entity from database")?;
                    //let mut entity = ReleaseEntity::db_get_rev(db_conn, row.rev_id.expect("valid, non-deleted row"))?;
                    entity.state = Some("active".to_string()); // XXX
                    entity.ident = Some(row.ident_id.to_string());
                    if let Some(expand) = expand {
                        entity
                            .db_expand(db_conn, expand)
                            .chain_err(|| "expanding sub-entities from database")?;
                    }
                    output_sender.send(serde_json::to_string(&entity)?);
                }
                Ok(())
            })();
            if let Err(ref e) = result {
                error!("{}", e.display_chain())
            }
            result.unwrap()
        }
    };
}

generic_loop_work!(loop_work_container, ContainerEntity);
generic_loop_work!(loop_work_creator, CreatorEntity);
generic_loop_work!(loop_work_file, FileEntity);
generic_loop_work!(loop_work_fileset, FilesetEntity);
generic_loop_work!(loop_work_webcapture, WebcaptureEntity);
generic_loop_work!(loop_work_release, ReleaseEntity);
generic_loop_work!(loop_work_work, WorkEntity);

fn loop_printer(
    output_receiver: channel::Receiver<String>,
    done_sender: channel::Sender<()>,
) -> Result<()> {
    let output = std::io::stdout();
    // XXX should log...
    // let mut buf_output = BufWriter::new(output.lock());
    let mut buf_output = BufWriter::new(output);
    for line in output_receiver {
        buf_output.write_all(&line.into_bytes())?;
        buf_output.write(b"\n")?;
        buf_output.flush()?;
    }
    drop(done_sender);
    Ok(())
}

fn parse_line(s: &str) -> Result<IdentRow> {
    let fields: Vec<String> = s.split("\t").map(|v| v.to_string()).collect();
    if fields.len() != 3 {
        bail!("Invalid input line");
    }
    Ok(IdentRow {
        ident_id: FatCatId::from_uuid(&Uuid::from_str(&fields[0])?),
        rev_id: match fields[1].as_ref() {
            "" => None,
            val => Some(Uuid::from_str(&val)?),
        },
        redirect_id: match fields[2].as_ref() {
            "" => None,
            val => Some(FatCatId::from_uuid(&Uuid::from_str(&val)?)),
        },
    })
}

#[test]
fn test_parse_line() {
    assert!(parse_line(
        "00000000-0000-0000-3333-000000000001\t00000000-0000-0000-3333-fff000000001\t"
    )
    .is_ok());
    assert!(parse_line(
        "00000-0000-0000-3333-000000000001\t00000000-0000-0000-3333-fff000000001\t"
    )
    .is_err());
    assert!(
        parse_line("00000-0000-0000-3333-000000000001\t00000000-0000-0000-3333-fff000000001")
            .is_err()
    );
    assert!(parse_line("00000000-0000-0000-3333-000000000002\t00000000-0000-0000-3333-fff000000001\t00000000-0000-0000-3333-000000000001").is_ok());
}

// Use num_cpus/2, or CLI arg for worker count
//
// 1. open buffered reader, buffered writer, and database pool. create channels
// 2. spawn workers:
//      - get a database connection from database pool
//      - iterate over row channel, pushing Strings to output channel
//      - exit when end of rows
// 3. spawn output printer
// 4. read rows, pushing to row channel
//      => every N lines, log to stderr
// 5. wait for all channels to finish, and quit
pub fn do_export(
    num_workers: usize,
    expand: Option<ExpandFlags>,
    entity_type: ExportEntityType,
    redirects: bool,
) -> Result<()> {
    let db_pool = database_worker_pool()?;
    let buf_input = BufReader::new(std::io::stdin());
    let (row_sender, row_receiver) = channel::bounded(CHANNEL_BUFFER_LEN);
    let (output_sender, output_receiver) = channel::bounded(CHANNEL_BUFFER_LEN);
    let (done_sender, done_receiver) = channel::bounded(0);

    info!("Starting an export of {} entities", entity_type);

    // Start row worker threads
    assert!(num_workers > 0);
    for _ in 0..num_workers {
        let db_conn = db_pool.get().expect("database pool");
        let row_receiver = row_receiver.clone();
        let output_sender = output_sender.clone();
        match entity_type {
            ExportEntityType::Container => thread::spawn(move || {
                loop_work_container(row_receiver, output_sender, &db_conn, expand)
            }),
            ExportEntityType::Creator => thread::spawn(move || {
                loop_work_creator(row_receiver, output_sender, &db_conn, expand)
            }),
            ExportEntityType::File => {
                thread::spawn(move || loop_work_file(row_receiver, output_sender, &db_conn, expand))
            }
            ExportEntityType::Fileset => thread::spawn(move || {
                loop_work_fileset(row_receiver, output_sender, &db_conn, expand)
            }),
            ExportEntityType::Webcapture => thread::spawn(move || {
                loop_work_webcapture(row_receiver, output_sender, &db_conn, expand)
            }),
            ExportEntityType::Release => thread::spawn(move || {
                loop_work_release(row_receiver, output_sender, &db_conn, expand)
            }),
            ExportEntityType::Work => {
                thread::spawn(move || loop_work_work(row_receiver, output_sender, &db_conn, expand))
            }
        };
    }
    drop(output_sender);
    // Start printer thread
    thread::spawn(move || loop_printer(output_receiver, done_sender).expect("printing to stdout"));

    let mut count = 0;
    for line in buf_input.lines() {
        let line = line?;
        let row = parse_line(&line)?;
        match (row.rev_id, row.redirect_id, redirects) {
            (None, _, _) => (),
            (Some(_), Some(_), false) => (),
            _ => row_sender.send(row),
        }
        count += 1;
        if count % 1000 == 0 {
            info!("processed {} lines...", count);
        }
    }
    drop(row_sender);
    done_receiver.recv();
    info!(
        "Done reading ({} lines), waiting for workers to exit...",
        count
    );
    Ok(())
}

fn main() -> Result<()> {
    let m = App::new("fatcat-export")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Bryan Newbold <bnewbold@archive.org>")
        .about("Fast exports of database entities from an id list")
        .arg(Arg::from_usage("<entity_type> 'what entity type the idents correspond to'")
            .possible_values(&ExportEntityType::variants())
            .case_insensitive(true))
        .args_from_usage(
            "-j --workers=[workers] 'number of threads (database connections) to use'
             -q --quiet 'less status output to stderr'
             --include-redirects 'include redirected idents (normally skipped)'
             --expand=[expand] 'sub-entities to include in dump'")
        .after_help("Reads a ident table TSV dump from stdin (aka, ident_id, rev_id, redirect_id), \
            and outputs JSON (one entity per line). Database connection info read from environment \
            (DATABASE_URL, same as fatcatd).")
        .get_matches();

    let num_workers: usize = match m.value_of("workers") {
        Some(_) => value_t_or_exit!(m.value_of("workers"), usize),
        None => std::cmp::min(1, num_cpus::get() / 2) as usize,
    };
    let expand = match m.value_of("expand") {
        Some(s) => Some(ExpandFlags::from_str(&s)?),
        None => None,
    };
    let log_level = if m.is_present("quiet") {
        "warn"
    } else {
        "info"
    };
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, log_level);
    env_logger::Builder::from_env(env).init();

    do_export(
        num_workers,
        expand,
        value_t_or_exit!(m.value_of("entity_type"), ExportEntityType),
        m.is_present("include_redirects"),
    )
}
