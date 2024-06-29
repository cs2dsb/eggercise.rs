use gloo_net::http::Method;
use leptos::window;
use shared::api::{
    self,
    error::{FrontendError, NoValidation, ResultContext, ServerError},
    response_errors::RegisterError,
};
use tracing::debug;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CredentialCreationOptions, PublicKeyCredential};
use webauthn_rs_proto::{CreationChallengeResponse, RegisterPublicKeyCredential};

use super::json_request;

pub async fn add_key() -> Result<(), FrontendError<ServerError<RegisterError>>> {
    // Ask the server to start the registration process and return a challenge
    debug!("add_key::json_request::register_new_key_start");
    let creation_challenge_response: CreationChallengeResponse = json_request(
        Method::POST,
        api::Auth::RegisterNewKeyStart.path(),
        None::<&()>,
    )
    .await?;

    // Convert to the browser type
    debug!("add_key::CreationChallengeResponse => CredentialCreationOptions");
    let credential_creation_options: CredentialCreationOptions = creation_challenge_response.into();

    // Get a promise that returns the credentials

    debug!("add_key::window.credentials.create");
    let create_fut = window()
        .navigator()
        .credentials()
        .create_with_options(&credential_creation_options)
        .map_err(FrontendError::from)
        .context("Creating credential create request (window.navigator.credentials.create)")?;

    // Get the credentials
    debug!("add_key::window.credentials.create.await");
    // TODO: calls to this can error but not return here. One example is when it
    // times out or no key is available
    let public_key_credential: PublicKeyCredential = JsFuture::from(create_fut)
        .await
        .map_err(FrontendError::from)
        .context("Awaiting credential create request (window.navigator.credentials.await)")?
        .into();

    // Convert to the rust type
    debug!("add_key::PublicKeyCredentials => RegisterPublicKeyCredential");
    let register_public_key_credentials: RegisterPublicKeyCredential = public_key_credential.into();

    // Complete the registration with the server
    debug!("add_key::json_request::register_new_key_finish");
    json_request(
        Method::POST,
        api::Auth::RegisterNewKeyFinish.path(),
        Some(&NoValidation(register_public_key_credentials)),
    )
    .await?;

    Ok(())
}
