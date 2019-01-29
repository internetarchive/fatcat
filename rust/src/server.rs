//! API endpoint handlers

use crate::auth::{self, AuthConfectionary};
use crate::errors::Result;
use cadence::{NopMetricSink, StatsdClient};
use diesel;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use std::env;

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
    pub metrics: StatsdClient,
}

/// Instantiate a new API server with a pooled database connection
pub fn create_server() -> Result<Server> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create database pool.");
    let auth_confectionary = auth::env_confectionary()?;
    let metrics = StatsdClient::from_sink("blackhole", NopMetricSink);
    Ok(Server {
        db_pool,
        auth_confectionary,
        metrics,
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
    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    env::set_var("DATABASE_URL", database_url);

    let mut server = create_server()?;
    server.auth_confectionary = AuthConfectionary::new_dummy();
    let conn = server.db_pool.get().expect("db_pool error");

    // create fresh database
    diesel_migrations::run_pending_migrations(&conn).unwrap();
    diesel_migrations::revert_latest_migration(&conn).unwrap();
    diesel_migrations::run_pending_migrations(&conn).unwrap();
    Ok(server)
}
