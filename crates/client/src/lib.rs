mod components;
use components::App;

use leptos::{mount_to_body, view};
use wasm_bindgen::prelude::wasm_bindgen;
use console_error_panic_hook::set_once as set_panic_hook;


#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
    mount_to_body(|| view! { <App/> });
}