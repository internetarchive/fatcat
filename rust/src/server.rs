//! API endpoint handlers

use crate::auth::*;
use crate::errors::*;
use chrono;
use diesel;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use rand::Rng;
use std::process::Command;
use std::{env, thread, time};

#[cfg(feature = "postgres")]
embed_migrations!("../migrations/");

pub type ConnectionPool = diesel::r2d2::Pool<ConnectionManager<diesel::pg::PgConnection>>;

pub type DbConn =
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

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

#[derive(Clone)]
pub struct Server {
    pub db_pool: ConnectionPool,
    pub auth_confectionary: AuthConfectionary,
}

/// Instantiate a new API server with a pooled database connection
pub fn create_server() -> Result<Server> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.");
    let confectionary = env_confectionary()?;
    Ok(Server {
        db_pool: pool,
        auth_confectionary: confectionary,
    })
}

/// Generates a server for testing. Calls an external bash script to generate a random postgres
/// database, which will be unique to this process but common across threads and connections. The
/// database will automagically get cleaned up (deleted) after 60 seconds.
/// Currently, start times are staggered by up to 200ms to prevent internal postgres concurrency
/// errors; if this fails run the tests serially (one at a time), which is slower but more robust.
/// CI should run tests serially.
pub fn create_test_server() -> Result<Server> {
    dotenv().ok();
    // sleep a bit so we don't have thundering herd collisions, resuliting in
    // "pg_extension_name_index" or "pg_proc_proname_args_nsp_index" or "pg_type_typname_nsp_index"
    // duplicate key violations.
    thread::sleep(time::Duration::from_millis(
        rand::thread_rng().gen_range(0, 200),
    ));
    let pg_tmp = Command::new("./tests/pg_tmp.sh")
        .output()
        .expect("run ./tests/pg_tmp.sh to get temporary postgres DB");
    let database_url = String::from_utf8_lossy(&pg_tmp.stdout).to_string();
    env::set_var("DATABASE_URL", database_url);

    let mut server = create_server()?;
    server.auth_confectionary = AuthConfectionary::new_dummy();
    let conn = server.db_pool.get().expect("db_pool error");

    // run migrations; this is a fresh/bare database
    diesel_migrations::run_pending_migrations(&conn).unwrap();
    Ok(server)
}
