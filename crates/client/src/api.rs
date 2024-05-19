#![allow(warnings)]
// #[derive(Clone, Copy)]
// pub struct UnauthorizedApi {
//     url: &'static str,
// }

// #[derive(Clone)]
// pub struct AuthorizedApi {
//     url: &'static str,
//     token: (),
// }

use std::{any::type_name, fmt::Debug};

use mime::APPLICATION_JSON;
use shared::{
    api::{self, error::{ ErrorContext, FrontendError, JsError, NoValidation, ResultContext, ValidationError, WrongContentTypeError }, response_errors::RegisterError},
    model::{RegistrationUser, ValidateModel},
};

use http::header;
use leptos::window;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys::{
    Error as GenericJsError,
    RangeError as JsRangeError,
    ReferenceError as JsReferenceError,
    SyntaxError as JsSyntaxError,
    // TryFromIntError as JsTryFromIntError,
    TypeError as JsTypeError,
    UriError as JsUriError,
}, CredentialCreationOptions, PublicKeyCredential};
use gloo_net::http::{ Request, RequestBuilder, Response, Method};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use webauthn_rs_proto::{CreationChallengeResponse, RegisterPublicKeyCredential};

trait ResponseExt: Sized {
    fn is_json(&self) -> bool;
    async fn json_map_err<T: DeserializeOwned, E: DeserializeOwned>(self) -> Result<T, E>; 
    async fn ok_result<E: DeserializeOwned>(self) -> Result<(), E>;
}

impl ResponseExt for Response {
    fn is_json(&self) -> bool {
        self.headers().get(header::CONTENT_TYPE.as_str())
            .map_or(false, |v| v == mime::APPLICATION_JSON.essence_str())
    }
    async fn json_map_err<T: DeserializeOwned, E: DeserializeOwned>(self) -> Result<T, E> {
        todo!()
        // if !self.ok() {
        //     Err(if self.is_json() {
        //         self.json::<ApiError>().await?
        //     } else {
        //         ApiError { message: self.text().await?}
        //     })?;
        // }    

        // Ok(self.json().await?)
    }

    async fn ok_result<E: DeserializeOwned>(self) -> Result<(), E> {
        todo!()
        // if !self.ok() {
        //     Err(ApiError { message: self.text().await? })?
        // }

        // Ok(())
    }
}

trait ResponseContentType: Sized {
    fn content_type(&self) -> Option<String>;
}

impl ResponseContentType for Response {
    fn content_type(&self) -> Option<String> {
        self.headers().get(header::CONTENT_TYPE.as_str())
    }
}

async fn json_request<B, R, E>(method: Method, url: &str, body: Option<&B>) -> Result<R, FrontendError<E>> 
where
    B: Serialize + Debug + ValidateModel, 
    R: DeserializeOwned, 
    E: DeserializeOwned
{
    // Check the body is valid
    if let Some(body) = body {
        body.validate()?;
    }

    let mut builder = RequestBuilder::new(url)
        .method(method.clone());

    // Add the json body. Use json(()) when there is no body so it still sets the other relevant headers
    let request = match body {
            Some(body) => builder.json(body),
            None => builder.json(&()),
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


pub async fn register(reg_user: &RegistrationUser) -> Result<(), FrontendError<RegisterError>> {
    let creation_challenge_response: CreationChallengeResponse = json_request(
        Method::POST, 
        api::Auth::RegisterStart.path(),
        Some(reg_user))
        .await?;

    // Convert to the browser type
    let credential_creation_options: CredentialCreationOptions = creation_challenge_response.into();

    // Get a promise that returns the credentials
    let cwo_fut = window()
        .navigator()
        .credentials()
        .create_with_options(&credential_creation_options)
        .map_err(FrontendError::from)
        .context("Creating credential request (window.navigator.credentials.create)")?;
    
    // Get the credentials
    let public_key_credential: PublicKeyCredential = JsFuture::from(cwo_fut)
        .await
        .map_err(FrontendError::from)
        .context("Awaiting credential request (window.navigator.credentials.await)")?
        .into();

    // Convert to the rust type
    let register_public_key_credentials: RegisterPublicKeyCredential = public_key_credential.into();

    // Complete the registration with the server
    json_request(
        Method::POST, 
        api::Auth::RegisterFinish.path(),
        Some(&NoValidation(register_public_key_credentials)))
        .await?;

    Ok(())
}