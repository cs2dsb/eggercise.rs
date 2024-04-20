use wasm_bindgen::{
    prelude::wasm_bindgen,
    JsValue,
};
use web_sys::{
    console::log_1, js_sys::Promise, FetchEvent, ServiceWorkerGlobalScope
};

macro_rules! console_log {
    ($($t:tt)*) => (log_1(&JsValue::from(format_args!($($t)*).to_string())))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn worker_install(sw: ServiceWorkerGlobalScope) -> Result<Promise, JsValue> {
    console_log!("worker_install called");
    sw.skip_waiting()
}

#[wasm_bindgen]
pub fn worker_activate(sw: ServiceWorkerGlobalScope) -> Promise{
    console_log!("worker_activate called");
    sw.clients().claim()
}

#[wasm_bindgen]
pub fn worker_fetch(_sw: ServiceWorkerGlobalScope, event: FetchEvent) {
    let request = event.request();
    let method = request.method();
    let uri = request.url();

    console_log!("worker_fetch called: {}, {}", method, uri);

    // Not calling event.respond_with causes it to just send the req over the
    // network as normal
}