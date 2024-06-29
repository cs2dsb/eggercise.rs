use gloo_net::http::Method;
use shared::{
    api::{
        self,
        error::{FrontendError, NoValidation, ServerError},
        response_errors::LoginError,
    },
    model::{LoginUser, User},
};
use tracing::debug;
use web_sys::CredentialRequestOptions;
use webauthn_rs_proto::{PublicKeyCredential as WebauthnPublicKey, RequestChallengeResponse};

use super::json_request;
use crate::{api::get_credentials, utils::JsValueIntoOk};

pub async fn login(login_user: &LoginUser) -> Result<User, FrontendError<ServerError<LoginError>>> {
    // Ask the server to start the login process and return a challenge
    debug!("login::json_request::login_start");
    let request_challenge_response: RequestChallengeResponse =
        json_request(Method::POST, api::Auth::LoginStart.path(), Some(login_user)).await?;

    // Convert to the browser type
    debug!("login::RequestChallengeResponse => CredentialRequestOptions");
    let credential_request_options: CredentialRequestOptions = request_challenge_response.into();

    // Ask the browser to give us a credential
    debug!("login::get_credentials");
    let public_key_credential = get_credentials(credential_request_options).await?;

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
