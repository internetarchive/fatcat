//! JSON Export Helper

extern crate clap;
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate error_chain;
extern crate fatcat;
extern crate fatcat_api_spec;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate uuid;
extern crate crossbeam_channel;
extern crate serde_json;

use clap::{App, Arg};
use slog::{Drain, Logger};
use dotenv::dotenv;
use std::env;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use fatcat_api_spec::models::*;
use std::str::FromStr;
use uuid::Uuid;
use fatcat::ConnectionPool;
use fatcat::api_helpers::*;
use fatcat::api_entity_crud::*;
use fatcat::errors::*;

use std::thread;
//use std::io::{Stdout,StdoutLock};
use crossbeam_channel as channel;
//use num_cpus; TODO:
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

const CHANNEL_BUFFER_LEN: usize = 200;

struct IdentRow {
    ident_id: FatCatId,
    rev_id: Option<Uuid>,
    redirect_id: Option<FatCatId>,
}

/// Instantiate a new API server with a pooled database connection
pub fn database_worker_pool() -> Result<ConnectionPool> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.");
    Ok(pool)
}

fn loop_work(row_receiver: channel::Receiver<IdentRow>, output_sender: channel::Sender<String>, db_conn: &DbConn, expand: Option<ExpandFlags>) -> Result<()> {
    for row in row_receiver {
        // TODO: based on types
        let mut entity = ReleaseEntity::db_get_rev(db_conn, row.rev_id.expect("valid, non-deleted row"))?;
        entity.state = Some("active".to_string()); // TODO
        entity.ident = Some(row.ident_id.to_string());
        if let Some(expand) = expand {
            entity.db_expand(db_conn, expand)?;
        }
        output_sender.send(serde_json::to_string(&entity)?);
    }
    Ok(())
}

fn loop_printer(output_receiver: channel::Receiver<String>, done_sender: channel::Sender<()>) -> Result<()> {
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

fn parse_line(s: String) -> Result<IdentRow> {
    let fields: Vec<String> = s.split("\t").map(|v| v.to_string()).collect();
    if fields.len() != 3 {
        bail!("Invalid input line");
    }
    Ok(IdentRow {
        ident_id: FatCatId::from_uuid(&Uuid::from_str(&fields[0])?),
        rev_id: Some(Uuid::from_str(&fields[1])?),
        redirect_id: None,
    })
}

#[test]
fn test_parse_line() {
    assert!(false)
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
pub fn run() -> Result<()> {

    let db_pool = database_worker_pool()?;
    let buf_input = BufReader::new(std::io::stdin());
    let worker_count = 4;
    let (row_sender, row_receiver) = channel::bounded(CHANNEL_BUFFER_LEN);
    let (output_sender, output_receiver) = channel::bounded(CHANNEL_BUFFER_LEN);
    let (done_sender, done_receiver) = channel::bounded(0);

    // Start row worker threads
    for _ in 0..worker_count {
        let db_conn = db_pool.get().expect("database pool");
        let row_receiver = row_receiver.clone();
        let output_sender = output_sender.clone();
        thread::spawn(move || loop_work(row_receiver, output_sender, &db_conn, None));
    }
    drop(output_sender);
    // Start printer thread
    thread::spawn(move || loop_printer(output_receiver, done_sender));

    for line in buf_input.lines() {
        let line = line?;
        let row = parse_line(line)?;
        row_sender.send(row);
    }
    drop(row_sender);
    done_receiver.recv();
    Ok(())
}

fn main() {
    /*
    let matches = App::new("server")
        .arg(
            Arg::with_name("https")
                .long("https")
                .help("Whether to use HTTPS or not"),
        )
        .get_matches();

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = Logger::root(drain, o!());
    */
    run().expect("success")
}
