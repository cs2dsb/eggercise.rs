use gloo_net::http::Method;
use shared::{
    api::{
        self,
        error::{FrontendError, NoValidation, ServerError},
        response_errors::RegisterError,
    },
    model::RegistrationUser,
    utils::fetch::json_request,
};
use tracing::debug;
use web_sys::CredentialCreationOptions;
use webauthn_rs_proto::{CreationChallengeResponse, RegisterPublicKeyCredential};

use crate::{api::create_credentials, utils::JsValueIntoOk};

pub async fn register(
    reg_user: &RegistrationUser,
) -> Result<(), FrontendError<ServerError<RegisterError>>> {
    // Ask the server to start the registration process and return a challenge
    debug!("register::json_request::register_start");
    let creation_challenge_response: CreationChallengeResponse = json_request(
        Method::POST,
        api::Auth::RegisterStart.path(),
        Some(reg_user),
    )
    .await?;

    // Convert to the browser type
    debug!("register::CreationChallengeResponse => CredentialCreationOptions");
    let credential_creation_options: CredentialCreationOptions = creation_challenge_response.into();

    // Ask the browser to give us a credential
    debug!("register::create_credentials");
    let public_key_credential = create_credentials(credential_creation_options).await?;

    // Convert to the rust type
    debug!("register::PublicKeyCredentials => RegisterPublicKeyCredential");
    let register_public_key_credentials: RegisterPublicKeyCredential =
        public_key_credential.ok()?;

    // Complete the registration with the server
    debug!("register::json_request::register_finish");
    json_request::<_, (), _>(
        Method::POST,
        api::Auth::RegisterFinish.path(),
        Some(&NoValidation(register_public_key_credentials)),
    )
    .await?;

    Ok(())
}
