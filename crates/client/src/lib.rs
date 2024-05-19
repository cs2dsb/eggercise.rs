#![feature(error_generic_member_access)]
mod components;
use components::App;

mod routes;
pub use routes::*;

pub mod api;
use console_error_panic_hook::set_once as set_panic_hook;
use leptos::{mount_to_body, view};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
    mount_to_body(|| view! { <App/> });
}
