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
use wasm_bindgen_futures::JsFuture;
use web_sys::{CredentialRequestOptions, PublicKeyCredential};
use webauthn_rs_proto::{PublicKeyCredential as WebauthnPublicKey, RequestChallengeResponse};

use super::json_request;

pub async fn login(login_user: &LoginUser) -> Result<User, FrontendError<ServerError<LoginError>>> {
    // Ask the server to start the login process and return a challenge
    let request_challenge_response: RequestChallengeResponse =
        json_request(Method::POST, api::Auth::LoginStart.path(), Some(login_user)).await?;

    // Convert to the browser type
    let credential_request_options: CredentialRequestOptions = request_challenge_response.into();

    // Get a promise that returns the credentials
    let get_fut = window()
        .navigator()
        .credentials()
        .get_with_options(&credential_request_options)
        .map_err(FrontendError::from)
        .context("Getting credential get request (window.navigator.credentials.get)")?;

    // Get the credentials
    let public_key_credential: PublicKeyCredential = JsFuture::from(get_fut)
        .await
        .map_err(FrontendError::from)
        .context("Awaiting credential get request (window.navigator.credentials.await)")?
        .into();

    // Convert to the rust type
    let public_key_credentials: WebauthnPublicKey = public_key_credential.into();

    // Complete the login with the server
    let user = json_request(
        Method::POST,
        api::Auth::LoginFinish.path(),
        Some(&NoValidation(public_key_credentials)),
    )
    .await?;

    Ok(user)
}
