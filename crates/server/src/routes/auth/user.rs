use std::time::Duration;

use axum::Json;
use chrono::Utc;
use shared::{
    api::{error::ServerError, response_errors::{FetchError, TemporaryLoginError}, Auth},
    model::{TemporaryLogin, User}, types::Uuid,
};

use crate::{db::DatabaseConnection, UserState, state::Args};

pub async fn fetch_user(
    DatabaseConnection(conn): DatabaseConnection,
    user_state: UserState,
) -> Result<Json<(User, Option<TemporaryLogin>)>, ServerError<FetchError>> {
    let res = conn
        .interact(move |conn| {
            let user = user_state.id.fetch_full_user(conn)?;
            let temporary_login = match user.temporary_login(conn)? {
                None => None,
                Some(tl) => if tl.expired() {
                    tl.delete(conn)?;
                    None
                } else {
                    Some(tl)
                }
            };

            Ok::<_, ServerError<_>>((user, temporary_login))
        })
        .await??;
    Ok(Json(res))
}

pub async fn create_temporary_login(
    DatabaseConnection(conn): DatabaseConnection,
    user_state: UserState,
    args: Args,
) -> Result<Json<TemporaryLogin>, ServerError<TemporaryLoginError>> {
    let temporary_login = conn.interact(move |conn| {
        let existing = TemporaryLogin::fetch_by_user_id(conn, &user_state.id)?;

        if let Some(existing) = existing {
            if existing.expired() {
                existing.delete(conn)?;
            } else {
                Err(TemporaryLoginError::AlreadyExists)?;
            }
        }

        let id = Uuid::new_v4();
        let expiry_date = Utc::now() + Duration::from_mins(args.temporary_login_expiry_minutes);
        let url = format!("{}{}",
            args.webauthn_origin,
            Auth::TemporaryLogin.path()
                .replace(":id", &id.hyphenated().to_string()));

        let temporary_login = TemporaryLogin::create(conn, TemporaryLogin {
            id, 
            user_id: (*user_state.id).clone(),
            expiry_date,
            url,
         })?;

        Ok::<_, ServerError<_>>(temporary_login)
    }).await??;

    Ok(Json(temporary_login))
}