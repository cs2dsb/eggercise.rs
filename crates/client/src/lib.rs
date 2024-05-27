#![feature(error_generic_member_access)]

use console_error_panic_hook::set_once as set_panic_hook;
use leptos::{mount_to_body, view };
use tracing::debug;
use wasm_bindgen::prelude::wasm_bindgen;

mod components;
use components::App;

mod routes;
pub use routes::*;
use web_sys::js_sys::Function;

pub mod api;
pub mod utils;

use utils::{
    sqlite3::SqlitePromiser,
    tracing::configure_tracing,
};

#[wasm_bindgen]
pub async fn start_client(sqlite_promiser: Function) {
    set_panic_hook();
    configure_tracing();

    let sqlite_promiser = SqlitePromiser::new(sqlite_promiser);
    debug!("SqlitePromiser!: {:?}", sqlite_promiser);

    let result = sqlite_promiser.get_config().await
        .unwrap();

    let pragma = [
        "PRAGMA journal_mode = WAL",
        "PRAGMA synchronous = NORMAL",
        "PRAGMA foreign_keys = ON",
    ];
    for p in pragma.into_iter() {
        sqlite_promiser.exec(p).await.unwrap();
    }

    let exercise_table = r#"
        CREATE TABLE IF NOT EXISTS exercise (
            id                  TEXT PRIMARY KEY NOT NULL,
            name                TEXT NOT NULL UNIQUE,
            creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );"#;
    let session_table = r#"
        CREATE TABLE IF NOT EXISTS session (
            id                  TEXT PRIMARY KEY NOT NULL,
            creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );"#;
    let session_exercise_table = r#"
        CREATE TABLE IF NOT EXISTS session_exercise (
            id                  TEXT PRIMARY KEY NOT NULL,
            exercise_id         TEXT NOT NULL,
            session_id          TEXT NOT NULL,
            creation_date       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_updated_date   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

            FOREIGN KEY (exercise_id) REFERENCES exercise(id),
            FOREIGN KEY (session_id) REFERENCES session(id) ON DELETE CASCADE
        );"#;

    let tables = [exercise_table, session_table, session_exercise_table];
    for t in tables.into_iter() {
        sqlite_promiser.exec(t).await.unwrap();
    }

    let r2 = sqlite_promiser.exec("SELECT * FROM sqlite_schema").await
        .unwrap();

    let opfs_tree = sqlite_promiser.opfs_tree().await.unwrap();

    mount_to_body(move || view! { 
        <App/>
        <p>{ format!("{:#?}", result) }</p>
        <p>{ format!("{:#?}", r2) }</p>
        <p>{ format!("{:#?}", opfs_tree) }</p>
    });

}
