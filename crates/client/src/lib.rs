#![feature(error_generic_member_access)]

use console_error_panic_hook::set_once as set_panic_hook;
use leptos::{mount_to_body, view };
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

#[wasm_bindgen]
pub async fn start_client(sqlite_promiser: Function) {
    set_panic_hook();
    configure_tracing();

    SqlitePromiser::new(sqlite_promiser).provide_context();
    mount_to_body(move || view! { 
        <App/>
    });
}
