use gloo_net::http::Method;
use shared::api::{
    self,
    error::{FrontendError, NoValidation, ServerError},
    response_errors::RegisterError,
};
use tracing::debug;
use web_sys::CredentialCreationOptions;
use webauthn_rs_proto::{CreationChallengeResponse, RegisterPublicKeyCredential};

use super::json_request;
use crate::api::create_credentials;

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

    // Ask the browser to give us a credential
    debug!("add_key::create_credentials");
    let public_key_credential = create_credentials(credential_creation_options).await?;

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
