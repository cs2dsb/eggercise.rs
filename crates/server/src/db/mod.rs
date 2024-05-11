use std::{ffi::c_int, sync::Once, time::Duration};

use include_dir::{include_dir, Dir};
use rusqlite::{Connection, OpenFlags, TransactionBehavior};
use rusqlite_migration::{Migrations, SchemaVersion};
use tracing::{debug, error, info, instrument, span, trace, warn, Level};

mod database_connection;
pub use database_connection::*;

pub mod model;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

fn sqlite_connection_profiling_callback(query: &str, duration: Duration) {
    trace!(target: "sqlite_profiling", ?duration, query);
}

fn sqlite_log_callback(sqlite_code: c_int, msg: &str) {
    use rusqlite::ffi;
    let err_code = ffi::Error::new(sqlite_code);
    
    // See https://www.sqlite.org/rescode.html for description of result codes.
    match sqlite_code & 0xff {
        ffi::SQLITE_NOTICE => info!(target: "sqlite", msg, %err_code, "SQLITE NOTICE"),
        ffi::SQLITE_WARNING => warn!(target: "sqlite", msg, %err_code, "SQLITE WARNING"),
        _ => error!(target: "sqlite", msg, %err_code, "SQLITE ERROR"),
    };
}

pub fn get_migrations() -> Result<Migrations<'static>, anyhow::Error> {
    Ok(Migrations::from_directory(&MIGRATIONS_DIR)?)
}

pub fn configure_new_connection(conn: &mut Connection) -> Result<(), anyhow::Error> {
    run_pragmas(conn)?;

    // Hook up the profiling callback
    conn.profile(Some(sqlite_connection_profiling_callback));

    Ok(())
}

#[instrument]
pub fn run_pragmas(conn: &Connection) -> Result<(), anyhow::Error> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

#[instrument]
pub fn run_migrations(connection_string: &str) -> Result<usize, anyhow::Error> {
    // Configure the log callback before opening the database
    static CONFIG_LOG: Once = Once::new();
    let mut config_result = Ok(());
    CONFIG_LOG.call_once(|| {
        unsafe {
            config_result = rusqlite::trace::config_log(Some(sqlite_log_callback));
        }
    });
    config_result?;

    let open_flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_URI
        | OpenFlags::SQLITE_OPEN_NO_MUTEX
        | OpenFlags::SQLITE_OPEN_CREATE;

    let mut conn = Connection::open_with_flags(connection_string, open_flags)?;
    configure_new_connection(&mut conn)?;

    debug!("Checking DB is writable");
    conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;

    let migrations = get_migrations()?;
    let ran = {
        let _span = span!(Level::INFO, "Running migrations").entered();

        let initial_version: usize = match migrations.current_version(&conn)? {
            SchemaVersion::Inside(n) => Ok(n.into()),
            SchemaVersion::Outside(n) => Err(anyhow::anyhow!("Schema version {n} is outside of known schema migrations. Manual intervention required")),
            SchemaVersion::NoneSet => Ok(0),
        }?;

        migrations.to_latest(&mut conn)?;

        let final_version: usize = match migrations.current_version(&conn)? {
            SchemaVersion::Inside(n) => Ok(n.into()),
            SchemaVersion::Outside(n) => Err(anyhow::anyhow!("Schema version {n} is outside of known schema migrations. Manual intervention required")),
            SchemaVersion::NoneSet => Ok(0),
        }?;

        final_version - initial_version
    };

    Ok(ran)
}