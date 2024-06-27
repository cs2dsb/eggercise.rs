use std::{
    cmp::Ordering,
    ffi::c_int,
    sync::Once,
    time::{Duration, Instant},
};

use include_dir::{include_dir, Dir};
use rusqlite::{Connection, OpenFlags, TransactionBehavior};
use rusqlite_migration::{Migrations, SchemaVersion};
use shared::{
    api::error::{Nothing, ServerError},
    model::{NewServiceVersion, ServiceVersion},
    other_error,
};
use tracing::{debug, error, info, instrument, span, trace, warn, Level};
mod database_connection;
pub use database_connection::*;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

fn sqlite_connection_profiling_callback(query: &str, duration: Duration) {
    trace!(target: "sqlite_profiling", ?duration, query);
}

fn sqlite_connection_trace_callback(query: &str) {
    trace!(target: "sqlite_tracing", query);
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

pub fn get_migrations() -> Result<Migrations<'static>, ServerError<Nothing>> {
    Ok(
        Migrations::from_directory(&MIGRATIONS_DIR).map_err(|e| ServerError::Other {
            message: format!("Migrations::from_directory: {:?}", e),
        })?,
    )
}

#[instrument(skip(conn))]
pub fn configure_new_connection(conn: &mut Connection) -> Result<(), ServerError<Nothing>> {
    run_pragmas(conn)?;

    if cfg!(debug_assertions) {
        conn.trace(Some(sqlite_connection_trace_callback));
    } else {
        // Hook up the profiling callback
        conn.profile(Some(sqlite_connection_profiling_callback));
    }

    Ok(())
}

#[instrument(skip(conn))]
pub fn run_pragmas(conn: &Connection) -> Result<(), ServerError<Nothing>> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

#[instrument]
pub fn run_migrations(
    connection_string: &str,
    current_version: &str,
) -> Result<(usize, Option<ServiceVersion>), ServerError<Nothing>> {
    // Configure the log callback before opening the database
    static CONFIG_LOG: Once = Once::new();
    let mut config_result = Ok(());
    CONFIG_LOG.call_once(|| unsafe {
        config_result = rusqlite::trace::config_log(Some(sqlite_log_callback));
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

        let initial_version: usize = match migrations.current_version(&conn)
            .map_err(|e| other_error!("Migrations::current_version: {:?}", e))? 
        {
            SchemaVersion::Inside(n) => Ok(n.into()),
            SchemaVersion::Outside(n) => Err(other_error!("Schema version {n} is outside of known schema migrations. Manual intervention required")),
            SchemaVersion::NoneSet => Ok(0),
        }?;

        migrations
            .to_latest(&mut conn)
            .map_err(|e| other_error!("Migrations::to_latest: {:?}", e))?;

        let final_version: usize = match migrations.current_version(&conn)
            .map_err(|e| other_error!("Migrations::current_version: {:?}", e))? 
        {
            SchemaVersion::Inside(n) => Ok(n.into()),
            SchemaVersion::Outside(n) => Err(other_error!("Schema version {n} is outside of known schema migrations. Manual intervention required")),
            SchemaVersion::NoneSet => Ok(0),
        }?;

        final_version - initial_version
    };

    let previous_version = ServiceVersion::fetch_latest(&conn)?;

    let newer = Ordering::Less
        == previous_version
            .as_ref()
            .map(|prev| prev.cmp(&current_version))
            // If this is the first version, it's automatically newer
            .unwrap_or(Ok(Ordering::Less))
            .map_err(|e| {
                other_error!(
                    "Comparing {:?} to {}: {:?}",
                    previous_version,
                    current_version,
                    e
                )
            })?;

    let new_version = if newer {
        let new_version = NewServiceVersion::new(current_version.to_owned())
            .map_err(|e| other_error!("NewServiceVersion::new({}): {:?}", current_version, e))?;
        let r = ServiceVersion::create(&mut conn, new_version)?;
        info!("New version: {r}");
        Some(r)
    } else {
        None
    };

    close_database(conn)?;

    Ok((ran, new_version))
}

/// Runs an optimize on the database. Should be run periodically to keep the
/// database running optimally. It should be very fast if run regularly
#[instrument(skip(conn))]
pub fn optimize_database(conn: &Connection) -> Result<Duration, ServerError<Nothing>> {
    let start = Instant::now();
    conn.pragma_update(None, "analysis_limit", "400")?;
    conn.pragma_update(None, "optimize", "")?;

    Ok(start.elapsed())
}

#[instrument(skip(conn))]
pub fn close_database(conn: Connection) -> Result<(), ServerError<Nothing>> {
    let d1 = optimize_database(&conn)?;
    let d2 = vacuum_database(&conn)?;

    info!(
        "Optimize db took: {:.3}, vacuum took: {:.3}, total: {:.3}",
        d1.as_secs_f32(),
        d2.as_secs_f32(),
        (d1 + d2).as_secs_f32()
    );
    if let Err((_conn, e)) = conn.close() {
        Err(e)?;
    }

    Ok(())
}

// Vacuums the database to free up space and improve fragmentation
#[instrument(skip(conn))]
pub fn vacuum_database(conn: &Connection) -> Result<Duration, ServerError<Nothing>> {
    let start = Instant::now();
    conn.execute("VACUUM", ())?;
    Ok(start.elapsed())
}
