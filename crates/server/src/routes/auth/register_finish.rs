use axum::Json;
use shared::{
    api::error::{Nothing, ServerError},
    model::NewUserWithPasskey,
    unauthorized_error,
};
use tracing::error;
use webauthn_rs::prelude::RegisterPublicKeyCredential;

use crate::{
    db::DatabaseConnection, ClientControlMessage, PasskeyRegistrationState, SessionClients,
    SessionValue, Webauthn,
};

pub async fn register_finish(
    DatabaseConnection(conn): DatabaseConnection,
    webauthn: Webauthn,
    mut session: SessionValue,
    clients: Option<SessionClients>,
    Json(register_public_key_credential): Json<RegisterPublicKeyCredential>,
) -> Result<Json<()>, ServerError<Nothing>> {
    // Get the challenge from the session
    let PasskeyRegistrationState { username, id, passkey_registration } =
        session.take_passkey_registration_state().await?.ok_or(unauthorized_error!(
            "Current session doesn't contain a PasskeyRegistrationState. Client error or replay \
             attack?"
        ))?;

    // Attempt to complete the passkey registration with the provided public key
    let passkey = webauthn
        .finish_passkey_registration(&register_public_key_credential, &passkey_registration)?;

    // Create the new user with their passkey
    let new_user = NewUserWithPasskey::new(id, username, passkey);
    let (user, _) =
        conn.interact(move |conn| Ok::<_, ServerError<_>>(new_user.create(conn)?)).await??;

    if let Some(clients) = clients {
        for client in clients.clients {
            if let Err(e) = client.send(ClientControlMessage::Login((&user).into())).await {
                error!("Error sending ClientControlMessage for user {user:?}: {e:?}");
            }
        }
    }

    Ok(Json(()))
}
