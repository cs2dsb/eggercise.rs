#![feature(error_generic_member_access, let_chains)]

use console_error_panic_hook::set_once as set_panic_hook;
use leptos::{mount_to_body, view};
use wasm_bindgen::prelude::wasm_bindgen;

mod components;
use components::{App, Online};

mod routes;
pub use routes::*;
use web_sys::js_sys::Function;

pub mod actions;
pub mod api;
pub mod db;
pub mod utils;

use db::sqlite3::SqlitePromiser;
use shared::{server_trace, utils::tracing::configure_tracing};
use utils::{rtc::Rtc, websocket::Websocket};

#[wasm_bindgen]
pub async fn start_client(sqlite_promiser: Function) {
    set_panic_hook();
    configure_tracing();

    server_trace!();

    SqlitePromiser::new(sqlite_promiser).provide_context();

    Online::provide_context();

    Websocket::provide_context().unwrap();
    let source = Websocket::take_rtc_source().expect("RtcSource missing");
    let sender = Websocket::get_sender();

    Rtc::provide_context(source, sender).await.unwrap();

    mount_to_body(move || {
        view! {
            <App/>
        }
    });
}
