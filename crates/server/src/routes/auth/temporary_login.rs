use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use client::ClientRoutes;
use shared::{
    api::error::{Nothing, ServerError},
    bad_request_error,
    model::{TemporaryLogin, User},
    status_code_error,
    types::Uuid,
};

use crate::{db::DatabaseConnection, SessionValue};

pub async fn temporary_login(
    DatabaseConnection(conn): DatabaseConnection,
    mut session: SessionValue,
    code: Path<String>,
) -> Result<impl IntoResponse, ServerError<Nothing>> {
    let id =
        Uuid::parse(&code).map_err(|_| bad_request_error!("Invalid code ({})", code.as_str()))?;

    let user = conn
        .interact(move |conn| {
            // Find the temporary login for the given code
            let temporary_login = if let Some(tl) = TemporaryLogin::fetch_maybe(conn, &id)? {
                // Check if it has expired
                if tl.expired() {
                    tl.delete(conn)?;
                    Err(status_code_error!(
                        StatusCode::GONE,
                        "The code has expired ({})",
                        code.as_str()
                    ))
                } else {
                    Ok(tl)
                }
            } else {
                Err(bad_request_error!("Code does not exist ({})", code.as_str()))
            }?;

            // Get the user to log in with this code
            let user = User::fetch_by_id(conn, &temporary_login.user_id)?;
            // Delete the code now it's been used
            temporary_login.delete(conn)?;

            Ok::<_, ServerError<_>>(user)
        })
        .await??;

    // Log the user in
    session.set_user_state(&user).await?;

    // Redirect the user to the index page
    Ok(Redirect::to(ClientRoutes::Profile.path()))
}
