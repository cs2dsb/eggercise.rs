use gloo_net::http::Method;
use leptos::window;
use shared::{
    api::{
        self,
        error::{FrontendError, NoValidation, ResultContext, ServerError},
        response_errors::LoginError,
    },
    model::{LoginUser, User},
};
use tracing::debug;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CredentialRequestOptions, PublicKeyCredential};
use webauthn_rs_proto::{PublicKeyCredential as WebauthnPublicKey, RequestChallengeResponse};

use super::json_request;
use crate::utils::JsValueIntoOk;

pub async fn login(login_user: &LoginUser) -> Result<User, FrontendError<ServerError<LoginError>>> {
    // Ask the server to start the login process and return a challenge
    debug!("login::json_request::login_start");
    let request_challenge_response: RequestChallengeResponse =
        json_request(Method::POST, api::Auth::LoginStart.path(), Some(login_user)).await?;

    // Convert to the browser type
    debug!("login::RequestChallengeResponse => CredentialRequestOptions");
    let credential_request_options: CredentialRequestOptions = request_challenge_response.into();

    // Get a promise that returns the credentials
    debug!("login::window.credentials.create");
    let get_fut = window()
        .navigator()
        .credentials()
        .get_with_options(&credential_request_options)
        .map_err(FrontendError::from)
        .context("Getting credential get request (window.navigator.credentials.get)")?;

    // Get the credentials
    debug!("login::window.credentials.create.await");
    let public_key_credential: PublicKeyCredential = JsFuture::from(get_fut)
        .await
        .map_err(FrontendError::from)
        .context("Awaiting credential get request (window.navigator.credentials.await)")?
        .into();

    // Convert to the rust type
    debug!("login::PublicKeyCredentials => WebauthnPublicKey");
    let public_key_credentials: WebauthnPublicKey = public_key_credential.ok()?;

    // Complete the login with the server
    debug!("login::json_request::login_finish");
    let user = json_request(
        Method::POST,
        api::Auth::LoginFinish.path(),
        Some(&NoValidation(public_key_credentials)),
    )
    .await?;

    Ok(user)
}
