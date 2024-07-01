use std::{fmt::Display, time::Duration};

use leptos::window;
use shared::api::error::{FrontendError, JsError, ResultContext};
use tracing::{debug, error};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{Array, Function, Promise},
    CredentialCreationOptions, CredentialRequestOptions, CredentialsContainer, PublicKeyCredential,
};

mod register;
pub use register::*;

mod login;
pub use login::*;

mod fetch_user;
pub use fetch_user::*;

mod add_key;
pub use add_key::*;

mod create_temporary_login;
pub use create_temporary_login::*;

mod ping;
pub use ping::*;

pub async fn run_promise_with_timeout(
    promise: Promise,
    timeout: Duration,
    timeout_message: &str,
) -> Result<JsValue, JsValue> {
    let mut cb = |_resolve: Function, reject: Function| {
        if let Err(e) = window()
            .set_timeout_with_callback_and_timeout_and_arguments_1(
                &reject,
                timeout.as_millis() as i32,
                &JsValue::from_str(timeout_message),
            )
            .map_err(JsError::from)
        {
            error!("Error from set_timeout in run_promise_with_timeout: {e}");
        }
    };

    let timeout = Promise::new(&mut cb);

    JsFuture::from(Promise::race(&Array::of2(&timeout, &promise))).await
}

async fn fetch_browser_credentials<T, F>(f: F) -> Result<PublicKeyCredential, FrontendError<T>>
where
    T: Display,
    F: FnOnce(CredentialsContainer) -> Result<Promise, JsValue>,
{
    // Get a promise that returns the credentials
    debug!("create/get_credentials::window.credentials.create/get");
    let create_fut = f(window().navigator().credentials())
        .map_err(FrontendError::from)
        .context(
            "Creating credential create/get request (window.navigator.credentials.create/get)",
        )?;

    // Get the credentials
    debug!("create/get_credentials::window.credentials.create/get.await");
    // The timeout is to handle the case where the browser never resolves or rejects
    // the promise. This happenes when it times out wating for the user, no key
    // is available or all keys are excluded already
    let public_key_credential: PublicKeyCredential = run_promise_with_timeout(
        create_fut,
        Duration::from_secs(30),
        "Timeout awaiting credential request",
    )
    .await
    .map_err(FrontendError::from)
    .context("Awaiting credential create/get request (window.navigator.credentials.await)")?
    .into();

    Ok(public_key_credential)
}

pub async fn create_credentials<T: Display>(
    options: CredentialCreationOptions,
) -> Result<PublicKeyCredential, FrontendError<T>> {
    fetch_browser_credentials(|c| c.create_with_options(&options))
        .await
        .with_context(|| "create_credentials")
}

pub async fn get_credentials<T: Display>(
    options: CredentialRequestOptions,
) -> Result<PublicKeyCredential, FrontendError<T>> {
    fetch_browser_credentials(|c| c.get_with_options(&options))
        .await
        .with_context(|| "get_credentials")
}
