use std::{
    any::type_name,
    error::Error,
    fmt::{Debug, Display},
    time::Duration,
};

use gloo_net::http::{Method, RequestBuilder, Response};
use http::header::{self, ACCEPT};
use leptos::window;
use mime::APPLICATION_JSON;
use serde::{de::DeserializeOwned, Serialize};
use shared::{
    api::error::{
        FrontendError, JsError, Nothing, ResultContext, ServerError, WrongContentTypeError,
    },
    model::ValidateModel,
};
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

mod notifications;
pub use notifications::*;

mod ping;
pub use ping::*;

use crate::utils::csrf::Csrf;

pub trait ResponseContentType: Sized {
    fn content_type(&self) -> Option<String>;
}

impl ResponseContentType for Response {
    fn content_type(&self) -> Option<String> {
        self.headers().get(header::CONTENT_TYPE.as_str())
    }
}

pub async fn json_request<B, R, E>(
    method: Method,
    url: &str,
    body: Option<&B>,
) -> Result<R, FrontendError<ServerError<E>>>
where
    B: Serialize + Debug + ValidateModel,
    R: DeserializeOwned,
    E: Error + DeserializeOwned + Display,
{
    // Check the body is valid
    debug!(
        "json_request({method}, {url}, body type: {})",
        type_name::<B>()
    );
    if let Some(body) = body {
        debug!("json_request::body::validate");
        body.validate()?;
        debug!("json_request::body::validate ok");
    }

    let mut builder = RequestBuilder::new(url)
        .method(method.clone())
        .header(ACCEPT.as_str(), APPLICATION_JSON.essence_str());

    let csrf = if method == Method::POST || method == Method::PATCH || method == Method::DELETE {
        // Needs a csrf token
        let csrf = Csrf::get().await?;
        builder = csrf.add_to(builder);
        Some(csrf)
    } else {
        None
    };

    // Add the json body or set the releavant headers
    debug!("json_request::request::build");
    let request = match body {
        Some(body) => builder.json(body),
        None => builder.build(),
    }
    .map_err(FrontendError::from)
    .with_context(|| format!("Converting {:?} to json body (for: {method} {url}", body))?;

    // Send the request and handle the network and js errors
    debug!("json_request::request::send");
    let response = request
        .send()
        .await
        .map_err(FrontendError::from)
        .with_context(|| format!("Sending {:?} to {method} {url}", body))?;

    if response.ok() {
        if let Some(mut csrf) = csrf {
            // Update the token
            csrf.update_from(&response)?;
        } else {
            // Still update it on GETs but don't error if it fails
            let _ = Csrf::provide_from::<Nothing>(&response);
        }
    }

    // Check the content-type is what we're expecting
    let content_type = response.content_type();
    let is_json = content_type
        .as_ref()
        .map_or(false, |v| v == mime::APPLICATION_JSON.essence_str());
    debug!("json_request::response::is_json: {is_json}");

    // Handle non-json errors (this isn't to allow the api to return other things,
    // it's only to handle errors)
    if !is_json {
        let body = response
            .text()
            .await
            .map_err(FrontendError::from)
            .with_context(|| format!("Extracting response body as text from {method} {url}"))?;

        debug!("json_request::return Err(WrongContentTypeError)");
        Err(WrongContentTypeError {
            expected: APPLICATION_JSON.to_string(),
            got: content_type,
            body,
        })
        .map_err(FrontendError::from)
        .with_context(|| format!("Response from {method} {url}"))?;
    }

    // Deserialize the error type
    if !response.ok() {
        debug!("json_request::return Err(FrontendError)");
        let err = response
            .json::<ServerError<E>>()
            .await
            .map_err(FrontendError::from)
            .with_context(|| {
                format!(
                    "Deserializing error response ({}) from {method} {url}",
                    type_name::<E>()
                )
            })?;

        Err(FrontendError::Inner {
            inner: err,
        })?;
    }

    // Deserialize the ok type
    debug!("json_request::deserialize");
    let payload = response
        .json::<R>()
        .await
        .map_err(FrontendError::from)
        .with_context(|| {
            format!(
                "Deserializing OK response ({}) from {method} {url}",
                type_name::<E>()
            )
        })?;

    debug!("json_request::return Ok::<{}>", type_name::<R>());
    Ok(payload)
}

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
