use axum::Json;
use chrono::Utc;
use shared::{
    api::error::{Nothing, ServerError},
    ensure_server,
    model::{Credential, User},
    unauthorized_error,
};
use webauthn_rs::prelude::PublicKeyCredential;

use crate::{db::DatabaseConnection, PasskeyAuthenticationState, SessionValue, Webauthn};

pub async fn login_finish(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    Json(public_key_credential): Json<PublicKeyCredential>,
) -> Result<Json<User>, ServerError<Nothing>> {
    // Get the challenge from the session
    let PasskeyAuthenticationState { passkey_authentication, .. } =
        session.take_passkey_authentication_state().await?.ok_or(unauthorized_error!(
            "Current session doesn't contain a PasskeyAuthenticationState. Client error or replay \
             attack?"
        ))?;

    // Attempt to complete the passkey authentication with the provided public key
    let authentication_result =
        webauthn.finish_passkey_authentication(&public_key_credential, &passkey_authentication)?;

    // At this point the autnetication has succeeded but there are a few more checks
    // and updates we need to make
    let user = conn
        .interact(move |conn| {
            // Need a transaction because we're updating the credential and user and want it
            // to rollback if either fail
            let tx = conn.transaction()?;

            let id = authentication_result.cred_id().clone().into();

            // Get the credential
            // If it was deleted between start & finish this might fail and we should not
            // proceed with the login
            let mut credential = Credential::fetch(&tx, &id)?;

            let mut dirty = false;

            // If the counter is non-zero, we have to check it
            let counter = authentication_result.counter();
            if counter > 0 {
                ensure_server!(
                    counter > credential.counter,
                    "Stored counter ({}) >= authentication result counter ({}). Possible \
                     credential clone or re-use.",
                    credential.counter,
                    counter
                );
                credential.counter = counter;
                dirty = true;
            }

            let backup_state = authentication_result.backup_state();
            if backup_state != credential.backup_state {
                credential.backup_state = backup_state;
                dirty = true;
            }

            let backup_eligible = authentication_result.backup_eligible();
            if backup_eligible != credential.backup_eligible {
                credential.backup_eligible = backup_eligible;
                dirty = true;
            }

            let now = Utc::now();
            credential.last_used_date = Some(now);

            if dirty {
                credential.last_updated_date = now;
            }

            credential.update(&tx)?;

            let mut user = User::fetch_by_id(&tx, &credential.user_id)?;
            user.last_updated_date = now;
            user.last_login_date = Some(now);
            user.update(&tx)?;

            tx.commit()?;

            Ok::<_, ServerError<Nothing>>(user)
        })
        .await??;

    // Update the user state in the session so the user is logged in on furture
    // requests
    session.set_user_state(&user).await?;

    Ok(Json(user))
}
