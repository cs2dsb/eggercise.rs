use std::fmt::Display;

use leptos::window;
use shared::api::error::FrontendError;
use wasm_bindgen_futures::JsFuture;
use web_sys::{PushManager, ServiceWorkerRegistration};

pub async fn get_service_worker_registration<T: Display>(
) -> Result<ServiceWorkerRegistration, FrontendError<T>> {
    let sw = window().navigator().service_worker();
    Ok(JsFuture::from(sw.ready()?).await?.into())
}

pub async fn get_web_push_manger<T: Display>() -> Result<PushManager, FrontendError<T>> {
    Ok(get_service_worker_registration().await?.push_manager()?)
}
