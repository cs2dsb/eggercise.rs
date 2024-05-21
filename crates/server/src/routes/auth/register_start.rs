use axum::Json;
use shared::{
    api::{error::ServerError, response_errors::RegisterError},
    model::{RegistrationUser, User},
};
use webauthn_rs::prelude::{CreationChallengeResponse, Uuid};

use crate::{db::DatabaseConnection, PasskeyRegistrationState, SessionValue, Webauthn};

pub async fn register_start(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    Json(reg_user): Json<RegistrationUser>,
) -> Result<Json<CreationChallengeResponse>, ServerError<RegisterError>> {
    // Remove the existing challenge
    // session.take_passkey_registration_state().await?;
    session.take_passkey_registration_state().await?;

    if reg_user.username.len() < 4 {
        Err(RegisterError::UsernameInvalid {
            message: "Username needs to be at least 4 characters".to_string(),
        })?;
    }

    let (existing, user_id) = {
        let username = reg_user.username.clone();
        conn.interact(move |conn| {
            // Get the uuid associated with the given username, if any
            let user_id = User::fetch_by_username(conn, username)?.map(|u| u.id);

            Ok::<_, ServerError<_>>(match user_id {
                None => (false, Uuid::new_v4().into()),
                Some(uuid) => (true, uuid),
            })
        })
        .await??
    };

    if existing {
        Err(RegisterError::UsernameUnavailable)?;
    }

    // Start the registration
    let (creation_challenge_response, passkey_registration) = webauthn.start_passkey_registration(
        *user_id,
        &reg_user.username,
        // TODO: display name
        &reg_user.username,
        None,
    )?;

    // Stash the registration
    session
        .set_passkey_registration_state(PasskeyRegistrationState::new(
            reg_user.username,
            user_id,
            passkey_registration,
        ))
        .await?;

    // Send the challenge back to the client
    Ok(Json(creation_challenge_response.into()))
}
