use wasm_bindgen::prelude::wasm_bindgen;
use console_error_panic_hook::set_once as set_panic_hook;
use leptos::*;

#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
    mount_to_body(|| view! {
        <h1>"Eggercise"</h1>
        <p>"Hello world!"</p>
        <div id="update_container"></div>
    });
}