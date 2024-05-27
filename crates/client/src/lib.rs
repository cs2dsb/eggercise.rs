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

use utils::sqlite3::SqlitePromiser;

#[wasm_bindgen]
pub async fn start_client(sqlite_promiser: Function) {
    set_panic_hook();

    let sqlite_promiser = SqlitePromiser::new(sqlite_promiser);

    leptos::logging::log!("SqlitePromiser: {:?}", sqlite_promiser);

    let result = sqlite_promiser.get_config().await
        .unwrap();

    mount_to_body(move || view! { 
        <App/>
        { format!("{:#?}", result) }
    });

}
