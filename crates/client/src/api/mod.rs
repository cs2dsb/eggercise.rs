use std::{any::type_name, fmt::{Debug, Display}};

use mime::APPLICATION_JSON;
use http::header::{self, ACCEPT};
use gloo_net::http::{ RequestBuilder, Response, Method};
use serde::{de::DeserializeOwned, Serialize};

use shared::{
    api::error::{ FrontendError, ResultContext, WrongContentTypeError },
    model::ValidateModel,
};

mod register;
pub use register::*;

mod login;
pub use login::*;

mod fetch_user;
pub use fetch_user::*;


pub trait ResponseContentType: Sized {
    fn content_type(&self) -> Option<String>;
}

impl ResponseContentType for Response {
    fn content_type(&self) -> Option<String> {
        self.headers().get(header::CONTENT_TYPE.as_str())
    }
}

pub async fn json_request<B, R, E>(method: Method, url: &str, body: Option<&B>) -> Result<R, FrontendError<E>> 
where
    B: Serialize + Debug + ValidateModel, 
    R: DeserializeOwned, 
    E: DeserializeOwned + Display
{
    // Check the body is valid
    if let Some(body) = body {
        body.validate()?;
    }

    let builder = RequestBuilder::new(url)
        .method(method.clone())
        .header(ACCEPT.as_str(), APPLICATION_JSON.essence_str());

    // Add the json body or set the releavant headers
    let request = match body {
            Some(body) => builder.json(body),
            None => builder.build(),
        }
        .map_err(FrontendError::from)
        .with_context(|| format!("Converting {:?} to json body (for: {method} {url}", body))?;

    // Send the request and handle the network and js errors
    let response = request
        .send()
        .await
        .map_err(FrontendError::from)
        .with_context(|| format!("Sending {:?} to {method} {url}", body))?;

    // Check the content-type is what we're expecting   
    let content_type = response.content_type();
    let is_json = content_type.as_ref()
        .map_or(false, |v| v == mime::APPLICATION_JSON.essence_str());

    // Handle non-json errors (this isn't to allow the api to return other things, it's only to handle errors)
    if !is_json {
        let body = response
            .text()
            .await
            .map_err(FrontendError::from)
            .with_context(|| format!("Extracting response body as text from {method} {url}"))?;

        Err(WrongContentTypeError {
            expected: APPLICATION_JSON.to_string(),
            got: content_type,
            body })
        .map_err(FrontendError::from)
        .with_context(|| format!("Response from {method} {url}"))?;
    }

    // Deserialize the error type 
    if !response.ok() {
        let err = response.json::<E>()
            .await
            .map_err(FrontendError::<E>::from)
            .with_context(|| format!("Deserializing error response ({}) from {method} {url}", type_name::<E>()))?;
        Err(FrontendError::Inner { inner: err })?;
    }

    // Deserialize the ok type
    let payload = response.json::<R>()
        .await
        .map_err(FrontendError::<E>::from)
        .with_context(|| format!("Deserializing OK response ({}) from {method} {url}", type_name::<E>()))?;

    Ok(payload)
} 