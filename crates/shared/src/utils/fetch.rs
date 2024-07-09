//! TODO: DRY out these functions

use std::{
    any::type_name,
    error::Error,
    fmt::{Debug, Display},
};

use gloo::net::http::{Method, RequestBuilder, Response};
use headers::{CacheControl, Header};
use http::header::{self, ACCEPT, CACHE_CONTROL};
use mime::APPLICATION_JSON;
use serde::{de::DeserializeOwned, Serialize};
use tracing::debug;

use crate::{
    api::error::{FrontendError, Nothing, ResultContext, ServerError, WrongContentTypeError},
    model::ValidateModel,
    utils::csrf::Csrf,
};

/// How many retries to do on a failed fetch request
pub const FETCH_RETRIES: usize = 3;

pub trait ResponseContentType: Sized {
    fn content_type(&self) -> Option<String>;
}

impl ResponseContentType for Response {
    fn content_type(&self) -> Option<String> {
        self.headers().get(header::CONTENT_TYPE.as_str())
    }
}

fn no_cache(builder: RequestBuilder) -> RequestBuilder {
    let cc = CacheControl::new().with_no_store();
    let mut headers = Vec::with_capacity(1);
    cc.encode(&mut headers);

    let value = headers.pop().expect("CacheControl::encode should be infallible...");
    let str = value.to_str().expect("CacheControl::encode valid str");

    builder.header(CACHE_CONTROL.as_str(), str)
}

/// Perform a json request
///
/// If the body is provided it is validated using ValidateModel before
/// sending. If this isn't desired, wrap it in NoValidation
///
/// Applies CSRF token if the method requres it
///
/// Retries FETCH_RETRIES times if the status code != 200
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
    let method = &method;
    for retry in 0..=FETCH_RETRIES {
        // Check the body is valid
        debug!("json_request({method}, {url}, body type: {})", type_name::<B>());

        if let Some(body) = body {
            debug!("json_request::body::validate");
            body.validate()?;
            debug!("json_request::body::validate ok");
        }

        let r = async move {
            let mut builder = no_cache(RequestBuilder::new(url))
                .method(method.clone())
                .header(ACCEPT.as_str(), APPLICATION_JSON.essence_str());

            let csrf =
                if method == Method::POST || method == Method::PATCH || method == Method::DELETE {
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
                // TODO: It would be nice to be able to do this outside the retry loop
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

            if let Some(mut csrf) = csrf {
                // Update the token
                csrf.update_from(&response)?;
            } else {
                // Still update it on GETs but don't error if it fails
                let _ = Csrf::provide_from::<Nothing>(&response);
            };

            // Check the content-type is what we're expecting
            let content_type = response.content_type();
            let is_json =
                content_type.as_ref().map_or(false, |v| v == mime::APPLICATION_JSON.essence_str());
            debug!("json_request::response::is_json: {is_json}");

            // Handle non-json errors (this isn't to allow the api to return other things,
            // it's only to handle errors)
            if !is_json {
                let body =
                    response.text().await.map_err(FrontendError::from).with_context(|| {
                        format!("Extracting response body as text from {method} {url}")
                    })?;

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

                Err(FrontendError::Inner { inner: err })?;
            }

            // Deserialize the ok type
            debug!("json_request::deserialize");
            let payload =
                response.json::<R>().await.map_err(FrontendError::from).with_context(|| {
                    format!("Deserializing OK response ({}) from {method} {url}", type_name::<E>())
                })?;

            debug!("json_request::return Ok::<{}>", type_name::<R>());
            Ok(payload)
        }
        .await;

        if r.is_ok() || retry == FETCH_RETRIES {
            return r;
        }
        debug!("json_request::retrying ({})", retry + 1);
    }

    unreachable!()
}

/// Perform a request
///
/// Although the body is not json in this method it still sets the ACCEPT header
/// to application/json so the error returned can be parsed
///
/// Applies CSRF token if the method requres it
///
/// Retries FETCH_RETRIES times if the status code != 200
pub async fn simple_request<E, F>(
    method: Method,
    url: &str,
    modify_request: Option<F>,
) -> Result<Response, FrontendError<ServerError<E>>>
where
    E: Error + DeserializeOwned + Display,
    F: Fn(RequestBuilder) -> RequestBuilder,
{
    let method = &method;
    let modify_request = &modify_request;

    for retry in 0..=FETCH_RETRIES {
        let r = async move {
            let mut builder = no_cache(RequestBuilder::new(url))
                .method(method.clone())
                .header(ACCEPT.as_str(), APPLICATION_JSON.essence_str());

            let csrf =
                if method == Method::POST || method == Method::PATCH || method == Method::DELETE {
                    // Needs a csrf token
                    let csrf = Csrf::get().await?;
                    builder = csrf.add_to(builder);
                    Some(csrf)
                } else {
                    None
                };

            if let Some(f) = modify_request {
                builder = f(builder);
            }

            // Build the request
            debug!("simple_request::request::build");
            let request = builder
                .build()
                .map_err(FrontendError::from)
                .with_context(|| format!("Building request (for: {method} {url}"))?;

            // Send the request and handle the network and js errors
            debug!("simple_request::request::send");
            let response = request
                .send()
                .await
                .map_err(FrontendError::from)
                .with_context(|| format!("Sending to {method} {url}"))?;

            if let Some(mut csrf) = csrf {
                // Update the token
                csrf.update_from(&response)?;
            } else {
                // Still update it on GETs but don't error if it fails
                let _ = Csrf::provide_from::<Nothing>(&response);
            };

            // Deserialize the error type
            if !response.ok() {
                // Check the content-type is what we're expecting
                let content_type = response.content_type();
                let is_json = content_type
                    .as_ref()
                    .map_or(false, |v| v == mime::APPLICATION_JSON.essence_str());
                debug!("simple_request::response::is_json: {is_json}");

                // Handle non-json errors (this isn't to allow the api to return other things,
                // it's only to handle errors)
                if !is_json {
                    let body =
                        response.text().await.map_err(FrontendError::from).with_context(|| {
                            format!("Extracting response body as text from {method} {url}")
                        })?;

                    debug!("simple_request::return Err(WrongContentTypeError)");
                    Err(WrongContentTypeError {
                        expected: APPLICATION_JSON.to_string(),
                        got: content_type,
                        body,
                    })
                    .map_err(FrontendError::from)
                    .with_context(|| format!("Response from {method} {url}"))?;
                }

                debug!("simple_request::return Err(FrontendError)");
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

                Err(FrontendError::Inner { inner: err })?;
            }

            debug!("simple_request::return Ok::<Response>");
            Ok(response)
        }
        .await;

        if r.is_ok() || retry == FETCH_RETRIES {
            return r;
        }
        debug!("simple_request::retrying ({})", retry + 1);
    }

    unreachable!()
}
