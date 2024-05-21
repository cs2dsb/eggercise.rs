use axum::Json;
use shared::{api::{error::ServerError, response_errors::RegisterError}, 
 model::Credential, unauthorized_error};
use webauthn_rs::prelude::CreationChallengeResponse;

use crate::{db::DatabaseConnection, PasskeyRegistrationState, SessionValue, UserState, Webauthn};

pub async fn register_new_key_start(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    user_state: UserState,
) -> Result<Json<CreationChallengeResponse>, ServerError<RegisterError>> {
    // Remove the existing challenge
    // session.take_passkey_registration_state().await?;
    session.take_passkey_registration_state().await?;
    
    let user_id = user_state.id;

    let (user, existing_key_ids) = {
        conn.interact(move |conn| {
            // We need the username for the challenge so fetch the full user
            let user = user_id.fetch_full_user(conn)?;
            // Fetch the existing passkeys for this user
            let passkeys = Credential::fetch_passkeys(conn, &*user_id)?
                .into_iter()
                // We only want the ID for this step
                .map(|p| p.cred_id().to_owned())
                .collect::<Vec<_>>();

            Ok::<_, ServerError<_>>((user, passkeys))
        }).await??
    };

    if existing_key_ids.is_empty() {
        // Log the user out
        let _ = session.take_user_state().await?;
        Err(unauthorized_error!("No existing keys found. It is now impossible to log in to this account"))?;
    }

    // Start the registration challenge
    let (creation_challenge_response, passkey_registration) = webauthn.start_passkey_registration(
        *user.id,
        &user.username,
        // TODO: display name
        &user.username,
        Some(existing_key_ids),
    )?;

    // Stash the registration
    session.set_passkey_registration_state(
        PasskeyRegistrationState::new(user.username, user.id, passkey_registration)).await?;

    // Send the challenge back to the client
    Ok(Json(creation_challenge_response.into()))
}