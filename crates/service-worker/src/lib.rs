use wasm_bindgen::prelude::wasm_bindgen;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
fn run() {
    console_log!("Log from {}", env!("CARGO_PKG_NAME"));
}