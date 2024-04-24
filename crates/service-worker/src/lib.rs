use wasm_bindgen::{
    prelude::wasm_bindgen, JsCast, JsValue
};
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::{
    console::{error_1, log_1}, js_sys::{Array, Object, Promise}, Cache, CacheStorage, FetchEvent, Request, Response, ResponseInit, ServiceWorkerGlobalScope
};
use console_error_panic_hook::set_once as set_panic_hook;

macro_rules! console_log {
    ($($t:tt)*) => (log_1(&JsValue::from(format_args!($($t)*).to_string())))
}

macro_rules! console_error {
    ($($t:tt)*) => (error_1(&JsValue::from(format_args!($($t)*).to_string())))
}

const INITIAL_FILES_TO_CACHE: [&str; 6] = [
    "/",
    "/index.html",
    "/favicon.ico",
    "/css/base.css",
    "/wasm/service_worker_bg.wasm",
    "/wasm/service_worker.js",
];

async fn get_cache(caches: &CacheStorage) -> Result<Cache, JsValue> {
    let cache: Cache = JsFuture::from(caches.open(env!("CARGO_PKG_VERSION")))
        .await?
        .into();
    Ok(cache)
}

async fn clear_cache(caches: CacheStorage) -> Result<JsValue, JsValue> {
    let cache = get_cache(&caches).await?;
    let keys: Array = JsFuture::from(cache.keys()).await?.into();
    
    for k in keys.into_iter().map(|x| <JsValue as Into<Request>>::into(x)) {
        console_log!("Clearing {}", k.url());
        JsFuture::from(cache.delete_with_request(&k)).await?;
    }
    Ok(JsValue::undefined())
}

async fn add_to_cache(caches: CacheStorage, resources: &[&str]) -> Result<JsValue, JsValue> {
    let cache = get_cache(&caches).await?;
    
    JsFuture::from(cache.add_all_with_str_sequence(&JsValue::from(resources.into_iter()
        .map(|x| JsValue::from_str(x))
        .collect::<Array>()))).await?;
    
    console_log!("add_to_cache OK");
    Ok(JsValue::undefined())
}

async fn cache_request(caches: &CacheStorage, request: &Request, response: &Response) -> Result<(), JsValue> {
    let cache = get_cache(caches).await?;
    
    let uri = request.url();
    // Need to clone before caching or the caller won't be able to use the original response
    let clone = response.clone()?;
    JsFuture::from(cache.put_with_request(request, &clone)).await?;
    
    console_log!("cache_request OK ({})", uri);
    Ok(())
}


async fn try_fetch_from_cache(sw: ServiceWorkerGlobalScope, request: Request) -> Result<JsValue, JsValue> {
    let caches = sw.caches()?;
    let cache = get_cache(&caches).await?;
    
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
                cache_request(&caches, &request, &response).await?;
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

async fn install(sw: ServiceWorkerGlobalScope) -> Result<JsValue, JsValue> {
    // Clearing is mainly for debugging   
    //clear_cache(sw.caches()?).await?;
    
    add_to_cache(sw.caches()?, &INITIAL_FILES_TO_CACHE).await?;
    
    JsFuture::from(sw.skip_waiting()?).await?;

    Ok(JsValue::undefined())
}

#[wasm_bindgen]
pub fn worker_install(sw: ServiceWorkerGlobalScope, version: String) -> Result<Promise, JsValue> {
    set_panic_hook();
    console_log!("worker_install called. Version: {}", version);

    Ok(future_to_promise(install(sw)))
}

#[wasm_bindgen]
pub fn worker_activate(sw: ServiceWorkerGlobalScope) -> Promise{
    set_panic_hook();
    console_log!("worker_activate called");
    sw.clients().claim()
}

#[wasm_bindgen]
pub fn worker_fetch(sw: ServiceWorkerGlobalScope, event: FetchEvent) -> Result<(), JsValue> {
    set_panic_hook();
    let request = event.request();
    let method = request.method();
    let uri = request.url();

    console_log!("worker_fetch called: {}, {}", method, uri);

    let fetch = future_to_promise(try_fetch_from_cache(sw, request));
    event.respond_with(&fetch)?;

    Ok(())
}