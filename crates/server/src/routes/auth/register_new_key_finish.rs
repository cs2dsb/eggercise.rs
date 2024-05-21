use axum::Json;
use shared::{api::error::{Nothing, ServerError}, 
 unauthorized_error};
use webauthn_rs::prelude::RegisterPublicKeyCredential;

use crate::{db::DatabaseConnection, PasskeyRegistrationState, SessionValue, UserState, Webauthn};

pub async fn register_new_key_finish(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    user_state: UserState,
    Json(register_public_key_credential): Json<RegisterPublicKeyCredential>,
) -> Result<Json<()>, ServerError<Nothing>> {
    // Get the challenge from the session
    let PasskeyRegistrationState { passkey_registration, .. } = session
        .take_passkey_registration_state()
        .await?
        .ok_or(unauthorized_error!("Current session doesn't contain a PasskeyRegistrationState. Client error or replay attack?"))?;

    // Attempt to complete the passkey registration with the provided public key
    let passkey = webauthn.finish_passkey_registration(&register_public_key_credential, &passkey_registration)?;
    
    let result = {
        conn.interact(move |conn| {
            // Get the user first
            let user = user_state.id.fetch_full_user(conn)
                .map_err(|e| (true, e.into()))?;
            
            // Add the new passkey
            user.add_passkey(conn, passkey)
                .map_err(|e| (false, e))?;

            Ok::<_, (bool, ServerError<_>)>(())
        })
        .await?
    };

    if let Err((logout, err)) = result {
        if logout {
            // Log the user out because there was no User in the database for the given id
            let _ = session.take_user_state().await?;
        }
        Err(err)?;
    }
    
    Ok(Json(()))
}