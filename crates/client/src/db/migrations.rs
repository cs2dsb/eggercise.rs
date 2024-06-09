//! TODO: replace
//! This is a simple clone of the features I use from rusqlite_migrations but
//! operating on the sqlite wasm connection we have access to here. The long
//! term goal is to get rusqlite working on wasm and use the same crates as the
//! server but this is sufficient to get working on some client features

use std::fmt;

use include_dir::{include_dir, Dir};
use leptos::{view, IntoView};
use thiserror::Error;
use tracing::debug;

use crate::utils::sqlite3::{SqlitePromiser, SqlitePromiserError};

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

#[derive(Debug, Clone, Copy)]
pub struct DatabaseVersion(usize);

impl IntoView for DatabaseVersion {
    fn into_view(self) -> leptos::View {
        view! {
            <span>
                { self.0.to_string() }
            </span>
        }
        .into_view()
    }
}

impl From<usize> for DatabaseVersion {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<DatabaseVersion> for usize {
    fn from(value: DatabaseVersion) -> Self {
        value.0
    }
}

impl fmt::Display for DatabaseVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Error)]
pub enum MigrationError {
    #[error("Migrations dir error: {0}")]
    Dir(String),
    #[error("Sqlite Promiser error: {0}")]
    Sql(String),
}

impl From<SqlitePromiserError> for MigrationError {
    fn from(value: SqlitePromiserError) -> Self {
        Self::Sql(value.to_string())
    }
}

async fn get_version(conn: &SqlitePromiser) -> Result<DatabaseVersion, SqlitePromiserError> {
    let result: Option<usize> = conn.get_value("PRAGMA user_version").await?;

    Ok(result.unwrap_or(0).into())
}

async fn set_version(conn: &SqlitePromiser, version: usize) -> Result<(), SqlitePromiserError> {
    conn.exec(format!("PRAGMA user_version={version}")).await?;
    Ok(())
}

pub async fn run_migrations(conn: &SqlitePromiser) -> Result<DatabaseVersion, MigrationError> {
    // To reset the db:
    // set_version(conn, 0).await?;
    // conn.exec("DROP TABLE session_exercise; DROP TABLE exercise; DROP TABLE
    // session;").await?;
    let mut version = get_version(conn).await?.into();
    debug!("Version: {version}");

    for (i, m) in MIGRATIONS_DIR.dirs().enumerate().skip(version) {
        let new_version = i + 1;
        debug!("Migration: {:?}, Version: {new_version}", m.path());
        let up = m
            .files()
            .find(|f| f.path().ends_with("up.sql"))
            .ok_or(MigrationError::Dir(format!(
                "Migration directory {:?} doesn't contain up.sql",
                m.path()
            )))?
            .contents_utf8()
            .ok_or(MigrationError::Dir(format!(
                "up.sql in migration directory {:?} could not be read as a utf8 string",
                m.path()
            )))?;

        conn.exec(up).await?;
        set_version(conn, new_version).await?;

        version = new_version;
    }

    Ok(version.into())
}
