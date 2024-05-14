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

async fn get_cache(caches: &CacheStorage, version: &str) -> Result<Cache, JsValue> {
    let cache: Cache = JsFuture::from(caches.open(version)).await?.into();
    Ok(cache)
}

#[allow(dead_code)]
async fn clear_cache(caches: CacheStorage, version: &str) -> Result<JsValue, JsValue> {
    let cache = get_cache(&caches, version).await?;
    let keys: Array = JsFuture::from(cache.keys()).await?.into();

    for k in keys
        .into_iter()
        .map(|x| <JsValue as Into<Request>>::into(x))
    {
        console_log!("Clearing {}", k.url());
        JsFuture::from(cache.delete_with_request(&k)).await?;
    }
    Ok(JsValue::undefined())
}

async fn add_to_cache(
    caches: CacheStorage,
    version: &str,
    resources: &[Request],
) -> Result<JsValue, JsValue> {
    let cache = get_cache(&caches, version).await?;

    JsFuture::from(
        cache.add_all_with_request_sequence(&JsValue::from(
            resources.into_iter().collect::<Array>(),
        )),
    )
    .await?;

    console_log!("add_to_cache OK");
    Ok(JsValue::undefined())
}

async fn fetch_from_cache(
    sw: &ServiceWorkerGlobalScope,
    version: &str,
    request: Request,
) -> Result<Response, JsValue> {
    let caches = sw.caches()?;
    let cache = get_cache(&caches, &version).await?;

    // Check the cache first
    let cached = JsFuture::from(cache.match_with_request(&request)).await?;
    let status_code = if cached.is_instance_of::<Response>() {
        console_log!("HIT: {}", request.url());
        return Ok(cached.into());
    } else if cached.is_undefined() {
        console_log!("MISS: {}", request.url());
        404
    } else {
        console_error!(
            "match_with_request returned something other than Result or undefined!: {:?}",
            cached
        );
        500
    };

    let headers = Object::new();
    js_sys::Reflect::set(
        &headers,
        &JsValue::from_str("Content-Type"),
        &JsValue::from_str("text/plain"),
    )?;

    let mut r_init = ResponseInit::new();
    r_init.status(status_code).headers(&headers);
    let response = Response::new_with_opt_str_and_init(
        Some(&format!(
            "Failed to retrieve {} from cache ({})",
            request.url(),
            status_code
        )),
        &r_init,
    )?;
    Ok(response)
}

fn log_and_err<T>(msg: &str) -> Result<T, JsValue> {
    console_error!("{}", msg);
    Err(JsValue::from(msg))
}

async fn fetch_json<T: DeserializeOwned>(
    sw: &ServiceWorkerGlobalScope,
    version: &str,
    request: Request,
) -> Result<T, JsValue> {
    let response = fetch_from_cache(sw, version, request).await?;
    let json = JsFuture::from(response.json()?).await?;

    JsValueSerdeExt::into_serde(&json)
        .map_err(|e| log_and_err::<()>(&format!("Error deserializing json: {}", e)).unwrap_err())
}

#[derive(Serialize)]
struct CacheHeader {
    cache: String,
}

impl CacheHeader {
    fn no_store() -> Self {
        Self {
            cache: "no-store".to_string(),
        }
    }
}

fn construct_request(url: &str, integrity: Option<&str>) -> Result<Request, JsValue> {
    let mut r_init = RequestInit::new();
    // Make sure we get the live file
    let value = <JsValue as JsValueSerdeExt>::from_serde(&CacheHeader::no_store()).unwrap();
    r_init.headers(&value);

    if let Some(hash) = integrity {
        r_init.integrity(hash);
    }

    Request::new_with_str_and_init(url, &r_init)
}

// Fetches the package. Fetches it strictly via the cache. If remote is true,
// fetches a new version and puts it in the cache first
async fn fetch_package(
    sw: &ServiceWorkerGlobalScope,
    version: &str,
    remote: bool,
) -> Result<ServiceWorkerPackage, JsValue> {
    let request = construct_request(SERVICE_WORKER_PACKAGE_URL, None)?;
    if remote {
        add_to_cache(sw.caches()?, version, &[request.clone()?]).await?;
    }

    Ok(fetch_json(sw, version, request).await?)
}

async fn install(sw: ServiceWorkerGlobalScope, version: String) -> Result<JsValue, JsValue> {
    let package = fetch_package(&sw, &version, true).await?;
    let requests = package
        .files
        .iter()
        .map(|f| construct_request(&f.path, Some(&f.hash)))
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

async fn fetch_cached(
    sw: ServiceWorkerGlobalScope,
    version: String,
    request: Request,
) -> Result<Response, JsValue> {
    let package = fetch_package(&sw, &version, false).await?;

    let uri = Url::new(&request.url())?;
    let path = uri.pathname();

    // Check if the request is a package file
    let response = if package.file(&path).is_some() {
        // If so, request it
        fetch_from_cache(&sw, &version, request).await?
    } else {
        // If not, return the index because the SPA contains multiple URLs the package
        // isn't aware of
        fetch_from_cache(&sw, &version, Request::new_with_str("/index.html")?).await?
    };

    Ok(response)
}

async fn fetch_direct(
    sw: ServiceWorkerGlobalScope,
    request: Request,
) -> Result<Response, JsValue> {
    let response = JsFuture::from(sw.fetch_with_request(&request)).await?;

    if response.is_instance_of::<Response>() {
        Ok(response.into())
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

        Ok(response)
    }
    
}

async fn fetch(
    sw: ServiceWorkerGlobalScope,
    version: String,
    request: Request,
) -> Result<JsValue, JsValue> {
    let method = request.method();
    let url = request.url();
    
    
    let uri = Url::new(&url)?;
    let path = uri.pathname();
    let cache = !path.starts_with(API_BASE_PATH);

    console_log!("worker_fetch called: {method}, {url}, cache: {cache}");

    let response = if cache {
        fetch_cached(sw, version, request).await?
    } else {
        fetch_direct(sw, request).await?
    };

    Ok(JsValue::from(&response))
}

#[wasm_bindgen]
pub fn worker_fetch(
    sw: ServiceWorkerGlobalScope,
    version: String,
    event: FetchEvent,
) -> Result<(), JsValue> {
    let fetch = future_to_promise(fetch(sw, version, event.request()));
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

            return Ok(());
        }
    }

    console_log!("worker_message got unexpected message: {:?}", event.data());

    Ok(())
}
