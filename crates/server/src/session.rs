use std::{
    error::Error,
    fmt::Display,
};

use axum::{async_trait, extract::FromRequestParts, http::{header::{ACCEPT, CONTENT_TYPE}, request::Parts, StatusCode}, response::{IntoResponse, Response}, Json};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use shared::{api::error::{ Nothing, ResultContext, ServerError}, model::{User, UserId}, types::Uuid};
use tower_sessions::{
    Session,
    session::Error as SessionError,
};
use tracing::error;
use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PasskeyRegistrationState {
    pub username: String,
    pub id: Uuid,
    pub passkey_registration: PasskeyRegistration,
}

impl PasskeyRegistrationState {
    pub fn new(username: String, id: Uuid, passkey_registration: PasskeyRegistration) -> Self {
        Self {
            username,
            id,
            passkey_registration,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PasskeyAuthenticationState {
    pub user_id: Uuid,
    pub passkey_authentication: PasskeyAuthentication,
}

impl PasskeyAuthenticationState {
    pub fn new(user_id: Uuid, passkey_authentication: PasskeyAuthentication) -> Self {
        Self {
            user_id,
            passkey_authentication,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserState {
    pub id: UserId,
}

impl From<&User> for UserState {
    fn from(value: &User) -> Self {
        Self {
            id: value.into()
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SessionData {
    passkey_registration_state: Option<PasskeyRegistrationState>,
    passkey_authentication_state: Option<PasskeyAuthenticationState>,
    user_state: Option<UserState>,
}

#[derive(Debug, Clone)]
pub struct SessionValue {
    session: Session,
    data: SessionData,
}

impl SessionValue {
    const SESSION_DATA_KEY: &'static str = "session.data";

    pub async fn take_passkey_registration_state<T: Error>(&mut self) -> Result<Option<PasskeyRegistrationState>, ServerError<T>> {
        let reg = self.data.passkey_registration_state.take();
        Self::update_session(&self.session, &self.data).await?;
        Ok(reg)
    }

    pub async fn set_passkey_registration_state<T: Error>(&mut self, passkey_registration: PasskeyRegistrationState) -> Result<(), ServerError<T>> {
        self.data.passkey_registration_state = Some(passkey_registration);
        Self::update_session(&self.session, &self.data).await?;
        Ok(())
    }

    pub async fn take_passkey_authentication_state<T: Error>(&mut self) -> Result<Option<PasskeyAuthenticationState>, ServerError<T>> {
        let reg = self.data.passkey_authentication_state.take();
        Self::update_session(&self.session, &self.data).await?;
        Ok(reg)
    }

    pub async fn set_passkey_authentication_state<T: Error>(&mut self, passkey_authentication: PasskeyAuthenticationState) -> Result<(), ServerError<T>> {
        self.data.passkey_authentication_state = Some(passkey_authentication);
        Self::update_session(&self.session, &self.data).await?;
        Ok(())
    }

    pub async fn take_user_state<T: Error>(&mut self) -> Result<Option<UserState>, ServerError<T>> {
        let user_id = self.data.user_state.take();
        Self::update_session(&self.session, &self.data).await?;
        Ok(user_id)
    }

    pub async fn set_user_state<T: Error>(&mut self, user: &User) -> Result<(), ServerError<T>> {
        self.data.user_state = Some(user.into());
        Self::update_session(&self.session, &self.data).await?;
        Ok(())
    }

    async fn update_session<T: Error>(session: &Session, data: &SessionData) -> Result<(), ServerError<T>> {
        session
            .insert(Self::SESSION_DATA_KEY, data.clone())
            .await
            .map_err(|e| match e {
                SessionError::SerdeJson(e) => ServerError::Json { message: e.to_string() },
                SessionError::Store(e) => ServerError::Other { message: e.to_string() },
            })
            .context("Updating session")?;
        Ok(())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SessionValue 
where 
    S: Send + Sync
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = 
            Session::from_request_parts(req, state)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;

        let data: SessionData = session
            .get(Self::SESSION_DATA_KEY)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?
            .unwrap_or_default();

        Ok(Self { session, data })
    }
}

pub struct JsonOrText<T: Serialize + Display> {
    json: bool,
    code: StatusCode, 
    body: T,
}

impl<T: Serialize + Display> JsonOrText<T> {
    pub fn new(json: bool, code: StatusCode, body: T) -> Self {
        Self { json, code, body }
    }
}

impl<T: Serialize + Display> IntoResponse for JsonOrText<T> {
    fn into_response(self) -> Response {
        let Self { json, code, body } = self;

        if json {
            (code, Json(body)).into_response()
        } else {
            (code, format!("{}", body)).into_response()
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserState
where 
    S: Send + Sync
{
    type Rejection = JsonOrText<ServerError<Nothing>>;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let accept_json = req.headers
            .get(ACCEPT)
            .map(|v| v
                .to_str()
                .map(|v| {
                    v.contains(mime::APPLICATION_JSON.essence_str())
                })
                .unwrap_or(false));

        let content_type_is_json = req.headers
            .get(CONTENT_TYPE)
            .map(|v| v
                .to_str()
                .map(|v| {
                    v == mime::APPLICATION_JSON.essence_str()
                })
                .unwrap_or(false));

        let json_reply = match (accept_json, content_type_is_json) {
            (Some(false), _) => false,
            (None, Some(false)) => false,
            (Some(true), _) => true,
            (None, Some(true)) => true,
            (None, None) => true,
        };

        macro_rules! not_logged_in {
            // TODO: need to return a sensible struct that the client can deserialize
            () => { JsonOrText::new(json_reply, StatusCode::UNAUTHORIZED, ServerError::<Nothing>::Unauthorized { message: format!("Not logged in (session)")}) };
        } 

        let session_value = SessionValue::from_request_parts(req, state)
            .map_err(|e| {
                error!("Failed to extract SessionValue: {:?}", e);
                not_logged_in!()
            })
            .await?;

        if let Some(user_id) = session_value.data.user_state {
            Ok(user_id)
        } else {
            Err(not_logged_in!())
        }
    }
}