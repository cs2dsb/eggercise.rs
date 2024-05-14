// #[derive(Clone, Copy)]
// pub struct UnauthorizedApi {
//     url: &'static str,
// }

// #[derive(Clone)]
// pub struct AuthorizedApi {
//     url: &'static str,
//     token: (),
// }

use leptos::window;
use shared::{
    api,
    model::RegistrationUser,
};

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
use gloo_net::http::{Request, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use webauthn_rs_proto::{CreationChallengeResponse, RegisterPublicKeyCredential};

// TODO: move into shared?
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
}

pub trait ErrorContext<T, E>: Sized {
    /// Add helpful context to errors
    ///
    /// Backtrace will be captured  if nightly feature is enabled
    ///
    /// `context` is provided as a closure to avoid potential formatting cost if
    /// the result isn't an error
    #[allow(dead_code)]
    fn with_context<S: Into<String>, F: FnOnce() -> S>(self, context: F) -> Result<T, E>;
    /// Add helpful context to errors
    ///
    /// Backtrace will be captured  if nightly feature is enabled
    ///
    /// `context` is provided as a closure to avoid potential formatting cost if
    /// the result isn't an error
    fn context<S: Into<String>>(self, context: S) -> Result<T, E>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JS Error: {0:?}")]
    Fetch(#[from] gloo_net::Error),
    #[error("API Error: {0:?}")]
    Api(ApiError),
    
    #[error("GenericJs Error: {0:?}")]
    GenericJs(GenericJsError),
    #[error("JsRange Error: {0:?}")]
    JsRange(JsRangeError),
    #[error("JsReference Error: {0:?}")]
    JsReference(JsReferenceError),
    #[error("JsSyntax Error: {0:?}")]
    JsSyntax(JsSyntaxError),
    // #[error("JsTryFromInt Error: {0:?}")]
    // JsTryFromInt(JsTryFromIntError),
    #[error("JsType Error: {0:?}")]
    JsType(JsTypeError),
    #[error("JsUri Error: {0:?}")]
    JsUri(JsUriError),
    #[error("UnknownJsValue Error: {0:?}")]
    UnknownJsValue(String),

    #[error("WithContext [{0}]: {1}")]
    WithContext(String, Box<Self>),
}

impl<T, E: Into<Error>> ErrorContext<T, Error> for Result<T, E> {
    fn with_context<S: Into<String>, F: FnOnce() -> S>(self, context: F) -> Result<T, Error> {
        self.context(context())
    }
    fn context<S: Into<String>>(self, context: S) -> Result<T, Error> {
        self.map_err(|e| {
            Error::WithContext(
                context.into(),
                Box::new(e.into()),
            )
        })
    }
}

// Tries to get the specific errors first then the generic one
// Finally falls back to outputting a string
fn map_js_error(err: JsValue) -> Error {
    if err.is_instance_of::<JsRangeError>() {
        return Error::JsRange(err.into());
    }
    if err.is_instance_of::<JsReferenceError>() {
        return Error::JsReference(err.into());
    }
    if err.is_instance_of::<JsSyntaxError>() {
        return Error::JsSyntax(err.into());
    }
    // Not supported by JsCast
    // if err.is_instance_of::<JsTryFromIntError>() {
    //     return Error::JsTryFromInt(err.into());
    // }
    if err.is_instance_of::<JsTypeError>() {
        return Error::JsType(err.into());
    }
    if err.is_instance_of::<JsUriError>() {
        return Error::JsUri(err.into());
    }
    if err.is_instance_of::<GenericJsError>() {
        return Error::GenericJs(err.into());
    }
    Error::UnknownJsValue(format!("{:?}", err))
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Self::Api(err)
    }
}

trait ResponseExt: Sized {
    async fn json_map_err<T: DeserializeOwned>(self) -> Result<T, Error>; 
    async fn ok_result(self) -> Result<(), Error>;
}

impl ResponseExt for Response {
    async fn json_map_err<T: DeserializeOwned>(self) -> Result<T, Error> {
        if !self.ok() {
            // TODO: Need to test this actually works. I don't think errors are being returned as JSON
            Err(self.json::<ApiError>().await?)?
        }    

        Ok(self.json().await?)
    }
    async fn ok_result(self) -> Result<(), Error> {
        if !self.ok() {
            Err(ApiError { message: self.text().await? })?
        }

        Ok(())
    }
}

pub async fn register(reg_user: &RegistrationUser) -> Result<(), Error> {
    // TODO: username requirements

    // Get a challenge from the server
    let creation_challenge_response: CreationChallengeResponse = Request::post(api::Auth::RegisterStart.path())
        .json(reg_user).context("json(RegistrationUser)")?
        .send()
        .await.context("RegisterStart::send")?
        .json_map_err()
        .await.context("RegisterStart::json_map_err")?;

    // Convert to the browser type
    let credential_creation_options: CredentialCreationOptions = creation_challenge_response.into();

    // Get a promise that returns the credentials
    let cwo_fut = window()
        .navigator()
        .credentials()
        .create_with_options(&credential_creation_options)
        .map_err(map_js_error).context("window.navigator.credentials.create::json_map_err")?;
    
    // Get the credentials
    let public_key_credential: PublicKeyCredential = JsFuture::from(cwo_fut)
        .await
        .map_err(map_js_error).context("window.navigator.credentials.await")?
        .into();

    // Convert to the rust type
    let register_public_key_credentials: RegisterPublicKeyCredential = public_key_credential.into();

    // Complete the registration with the server
    Ok(Request::post(api::Auth::RegisterFinish.path())
        .json(&register_public_key_credentials).context("json(RegisterPublicKeyCredential)")?
        .send()
        .await.context("RegisterFinish::send")?
        .ok_result()
        .await.context("RegisterFinish::ok_result")?)
}