/*
use console_error_panic_hook::set_once as set_panic_hook;
use gloo_utils::format::JsValueSerdeExt;
use serde::{de::DeserializeOwned, Serialize};
use shared::{api::API_BASE_PATH, ServiceWorkerPackage, SERVICE_WORKER_PACKAGE_URL};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::{
    console::{error_1, log_1},
    js_sys::{Array, Object, Promise},
    Cache, CacheStorage, FetchEvent, MessageEvent, Request, RequestInit, Response, ResponseInit,
    ServiceWorkerGlobalScope, Url,
};

const SKIP_WAITING: &str = "SKIP_WAITING";

macro_rules! console_log {
    ($($t:tt)*) => (log_1(&JsValue::from(format_args!($($t)*).to_string())))
}

macro_rules! console_error {
    ($($t:tt)*) => (error_1(&JsValue::from(format_args!($($t)*).to_string())))
}

#[wasm_bindgen]
pub fn worker_message(sw: ServiceWorkerGlobalScope, event: MessageEvent) -> Result<(), JsValue> {
    if let Ok(value) = <JsValue as TryInto<String>>::try_into(event.data()) {
        if value == SKIP_WAITING {
            console_log!("worker_message got SKIP_WAITING");

            // MDN states the promise returned can be safely ignored
            let _ = sw.skip_waiting()?;

            return Ok(());
        }
    }

    console_log!("worker_message got unexpected message: {:?}", event.data());

    Ok(())
}
*/