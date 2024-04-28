use serde::de::DeserializeOwned;
use shared::{ServiceWorkerPackage, SERVICE_WORKER_PACKAGE_URL};
use wasm_bindgen::{
    prelude::wasm_bindgen, JsCast, JsValue
};
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::{
    console::{error_1, log_1}, js_sys::{Array, Object, Promise}, Cache, CacheStorage, FetchEvent, MessageEvent, Request, RequestInit, Response, ResponseInit, ServiceWorkerGlobalScope
};
use gloo_utils::format::JsValueSerdeExt;
use console_error_panic_hook::set_once as set_panic_hook;

const SKIP_WAITING: &str = "SKIP_WAITING";

macro_rules! console_log {
    ($($t:tt)*) => (log_1(&JsValue::from(format_args!($($t)*).to_string())))
}

macro_rules! console_error {
    ($($t:tt)*) => (error_1(&JsValue::from(format_args!($($t)*).to_string())))
}

async fn get_cache(caches: &CacheStorage, version: &str) -> Result<Cache, JsValue> {
    let cache: Cache = JsFuture::from(caches.open(version))
        .await?
        .into();
    Ok(cache)
}

#[allow(dead_code)]
async fn clear_cache(caches: CacheStorage, version: &str) -> Result<JsValue, JsValue> {
    let cache = get_cache(&caches, version).await?;
    let keys: Array = JsFuture::from(cache.keys()).await?.into();
    
    for k in keys.into_iter().map(|x| <JsValue as Into<Request>>::into(x)) {
        console_log!("Clearing {}", k.url());
        JsFuture::from(cache.delete_with_request(&k)).await?;
    }
    Ok(JsValue::undefined())
}

async fn add_to_cache(caches: CacheStorage, version: &str, resources: &[Request]) -> Result<JsValue, JsValue> {
    let cache = get_cache(&caches, version).await?;
    
    JsFuture::from(cache.add_all_with_request_sequence(&JsValue::from(resources.into_iter()
        .collect::<Array>()))).await?;
    
    console_log!("add_to_cache OK");
    Ok(JsValue::undefined())
}

async fn cache_request(caches: &CacheStorage, version: &str, request: &Request, response: &Response) -> Result<(), JsValue> {
    let cache = get_cache(caches, version).await?;
    
    let uri = request.url();
    // Need to clone before caching or the caller won't be able to use the original response
    let clone = response.clone()?;
    JsFuture::from(cache.put_with_request(request, &clone)).await?;
    
    console_log!("cache_request OK ({})", uri);
    Ok(())
}


async fn try_fetch_from_cache(sw: ServiceWorkerGlobalScope, version: String, request: Request) -> Result<JsValue, JsValue> {
    let caches = sw.caches()?;
    let cache = get_cache(&caches, &version).await?;
    
    // Check the cache first
    let cached = JsFuture::from(cache.match_with_request(&request)).await?;
    if cached.is_instance_of::<Response>() {
        console_log!("HIT: {}", request.url());
        return Ok(cached);
    } else {
        console_log!("MISS: {}", request.url());
    }

    // Try and fetch it
    match JsFuture::from(sw.fetch_with_request(&request)).await {
        Ok(response) => {
            if response.is_instance_of::<Response>() {
                let response: Response = response.into();
                cache_request(&caches, &version, &request, &response).await?;
                Ok(JsValue::from(&response))
            } else {
                let e = format!("Fetch returned something other than a Response: {:?}", response);
                console_error!("{}", e);
                
                // We have to construct some kind of response
                let headers = Object::new();
                js_sys::Reflect::set(
                    &headers, 
                    &JsValue::from_str("Content-Type"),
                    &JsValue::from_str("text/plain"))?;

                let mut r_init = ResponseInit::new();
                r_init
                    .status(500)
                    .headers(&headers);
                let response = Response::new_with_opt_str_and_init(
                    Some(&e), &r_init)?;
                Ok(JsValue::from(&response))
            }
        },
        Err(e) => {
            console_error!("Fetch error: {:?}", e);
            Err(e)
        }
    }
}

fn log_and_err<T>(msg: &str) -> Result<T, JsValue> {
    console_error!("{}", msg);
    Err(JsValue::from(msg))
}

async fn fetch_response(sw: &ServiceWorkerGlobalScope, request: Request) -> Result<Response, JsValue> {
    let response = JsFuture::from(sw.fetch_with_request(&request)).await?;

    if response.is_instance_of::<Response>() {
        Ok(response.into())
    } else {
        log_and_err(&format!("Fetch of ({:?}) returned something other than a Response: {:?}", request.url(), response))
    }
}

async fn fetch_json<T: DeserializeOwned>(sw: &ServiceWorkerGlobalScope, request: Request) -> Result<T, JsValue> {
    let response = fetch_response(sw, request).await?;
    let json = JsFuture::from(response.json()?).await?;
    
    json.into_serde()
        .map_err(|e| log_and_err::<()>(&format!("Error deserializing json: {}", e)).unwrap_err())
}

async fn install(sw: ServiceWorkerGlobalScope, version: String) -> Result<JsValue, JsValue> {
    let package: ServiceWorkerPackage = fetch_json(&sw, Request::new_with_str(SERVICE_WORKER_PACKAGE_URL)?).await?;
    let requests = package.files
        .iter()
        .map(|f| Request::new_with_str_and_init(
            &f.path,
            &RequestInit::new()
                .integrity(&f.hash)))
        .collect::<Result<Vec<_>, _>>()?;

    add_to_cache(sw.caches()?, &version, &requests).await?;
    
    Ok(JsValue::undefined())
}

#[wasm_bindgen]
pub fn worker_install(sw: ServiceWorkerGlobalScope, version: String) -> Result<Promise, JsValue> {
    set_panic_hook();
    console_log!("worker_install called. Version: {}", version);

    Ok(future_to_promise(install(sw, version)))
}

#[wasm_bindgen]
pub fn worker_activate(_sw: ServiceWorkerGlobalScope) -> Promise {
    set_panic_hook();
    console_log!("worker_activate called");

    Promise::resolve(&JsValue::undefined())
}

#[wasm_bindgen]
pub fn worker_fetch(sw: ServiceWorkerGlobalScope, version: String, event: FetchEvent) -> Result<(), JsValue> {
    let request = event.request();
    let method = request.method();
    let uri = request.url();

    console_log!("worker_fetch called: {}, {}", method, uri);

    let fetch = future_to_promise(try_fetch_from_cache(sw, version, request));
    event.respond_with(&fetch)?;

    Ok(())
}

#[wasm_bindgen]
pub fn worker_message(sw: ServiceWorkerGlobalScope, event: MessageEvent) -> Result<(), JsValue> {
    if let Ok(value) = <JsValue as TryInto<String>>::try_into(event.data()) {
        if value == SKIP_WAITING {
            console_log!("worker_message got SKIP_WAITING");

            // MDN states the promise returned can be safely ignored
            let _ = sw.skip_waiting()?;

            return Ok(())
        }    
    }

    console_log!("worker_message got unexpected message: {:?}", event.data());

    Ok(())
}