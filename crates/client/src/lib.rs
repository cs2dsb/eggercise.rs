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
pub mod db;

use utils::{
    sqlite3::SqlitePromiser,
    tracing::configure_tracing,
};
use db::migrations;

#[wasm_bindgen]
pub async fn start_client(sqlite_promiser: Function) {
    set_panic_hook();
    configure_tracing();

    let sqlite_promiser = SqlitePromiser::new(sqlite_promiser);
    debug!("SqlitePromiser!: {:?}", sqlite_promiser);

    sqlite_promiser.configure().await.unwrap();

    let opfs_tree = sqlite_promiser.opfs_tree().await.unwrap();

    let version = migrations::run_migrations(&sqlite_promiser).await.unwrap();

    mount_to_body(move || view! { 
        <App/>
        <p>{ format!("DB Version: {}", version) }</p>
        <p>{ format!("OPFS files: {:#?}", opfs_tree) }</p>
    });

}
