use std::{
    any::type_name,
    error::Error,
    fmt::{Debug, Display},
};

use gloo_net::http::{Method, RequestBuilder, Response};
use http::header::{self, ACCEPT};
use mime::APPLICATION_JSON;
use serde::{de::DeserializeOwned, Serialize};
use shared::{
    api::error::{FrontendError, ResultContext, ServerError, WrongContentTypeError},
    model::ValidateModel,
};
use leptos::logging::log;

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
    log!("json_request({method}, {url}, body type: {})", type_name::<B>());
    if let Some(body) = body {
        log!("json_request::body::validate");
        body.validate()?;
        log!("json_request::body::validate ok");
    }

    let builder = RequestBuilder::new(url)
        .method(method.clone())
        .header(ACCEPT.as_str(), APPLICATION_JSON.essence_str());

    // Add the json body or set the releavant headers
    log!("json_request::request::build");
    let request = match body {
        Some(body) => builder.json(body),
        None => builder.build(),
    }
    .map_err(FrontendError::from)
    .with_context(|| format!("Converting {:?} to json body (for: {method} {url}", body))?;

    // Send the request and handle the network and js errors
    log!("json_request::request::send");
    let response = request
        .send()
        .await
        .map_err(FrontendError::from)
        .with_context(|| format!("Sending {:?} to {method} {url}", body))?;

    // Check the content-type is what we're expecting
    let content_type = response.content_type();
    let is_json = content_type
    .as_ref()
    .map_or(false, |v| v == mime::APPLICATION_JSON.essence_str());
    log!("json_request::response::is_json: {is_json}");

    // Handle non-json errors (this isn't to allow the api to return other things,
    // it's only to handle errors)
    if !is_json {
        let body = response
            .text()
            .await
            .map_err(FrontendError::from)
            .with_context(|| format!("Extracting response body as text from {method} {url}"))?;

        log!("json_request::return Err(WrongContentTypeError)");
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
        log!("json_request::return Err(FrontendError)");
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
    log!("json_request::deserialize");
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


    log!("json_request::return Ok::<{}>", type_name::<R>());
    Ok(payload)
}
