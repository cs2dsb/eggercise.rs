use axum::Json;
use shared::{
    api::{error::ServerError, response_errors::LoginError},
    model::{Credential, LoginUser, Model, User, UserIden},
};
use webauthn_rs::prelude::RequestChallengeResponse;

use crate::{db::DatabaseConnection, PasskeyAuthenticationState, SessionValue, Webauthn};

pub async fn login_start(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    Json(login_user): Json<LoginUser>,
) -> Result<Json<RequestChallengeResponse>, ServerError<LoginError>> {
    // Remove the existing challenge
    session.take_passkey_authentication_state().await?;

    // Remove the existing user
    session.take_user_state().await?;

    if login_user.username.len() < 4 {
        Err(LoginError::UsernameInvalid {
            message: "Username needs to be at least 4 characters".to_string(),
        })?;
    }

    let (user, existing_passkeys) = {
        let username = login_user.username.clone();
        conn.interact(move |conn| {
            // First get the user associated with the given username, if any
            let user = User::fetch_by_column_maybe(conn, &username, UserIden::Username)?;

            // Then fetch the existing passkeys if the user exists
            Ok::<_, ServerError<_>>(match user {
                None => (None, None),
                Some(user) => {
                    let passkeys =
                        Credential::fetch_passkeys(conn, &user.id)?.into_iter().collect::<Vec<_>>();
                    (Some(user), Some(passkeys))
                },
            })
        })
        .await??
    };

    if user.is_none() {
        Err(LoginError::UsernameDoesntExist)?;
    }
    let user = user.unwrap();

    if existing_passkeys.as_ref().map_or(0, |v| v.len()) == 0 {
        Err(LoginError::UserHasNoCredentials)?;
    }
    let existing_passkeys = existing_passkeys.unwrap();

    // Start the authentication attempt
    let (request_challenge_response, passkey_authentication) =
        webauthn.start_passkey_authentication(&existing_passkeys)?;

    // Stash the authentication
    session
        .set_passkey_authentication_state(PasskeyAuthenticationState::new(
            user.id,
            passkey_authentication,
        ))
        .await?;

    // Send the challenge back to the client
    Ok(Json(request_challenge_response.into()))
}
