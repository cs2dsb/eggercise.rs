use leptos::window;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CredentialCreationOptions, PublicKeyCredential};
use gloo_net::http::Method;
use webauthn_rs_proto::{CreationChallengeResponse, RegisterPublicKeyCredential};

use shared::api::{self, error::{ FrontendError, NoValidation, ResultContext, ServerError }, response_errors::RegisterError};
use super::json_request;

pub async fn add_key() -> Result<(), FrontendError<ServerError<RegisterError>>> {
    // Ask the server to start the registration process and return a challenge
    let creation_challenge_response: CreationChallengeResponse = json_request(
        Method::POST, 
        api::Auth::RegisterNewKeyStart.path(),
        None::<&()>)
        .await?;

    // Convert to the browser type
    let credential_creation_options: CredentialCreationOptions = creation_challenge_response.into();

    // Get a promise that returns the credentials
    let create_fut = window()
        .navigator()
        .credentials()
        .create_with_options(&credential_creation_options)
        .map_err(FrontendError::from)
        .context("Creating credential create request (window.navigator.credentials.create)")?;
    
    // Get the credentials
    let public_key_credential: PublicKeyCredential = JsFuture::from(create_fut)
        .await
        .map_err(FrontendError::from)
        .context("Awaiting credential create request (window.navigator.credentials.await)")?
        .into();

    // Convert to the rust type
    let register_public_key_credentials: RegisterPublicKeyCredential = public_key_credential.into();

    // Complete the registration with the server
    json_request(
        Method::POST, 
        api::Auth::RegisterNewKeyFinish.path(),
        Some(&NoValidation(register_public_key_credentials)))
        .await?;

    Ok(())
}