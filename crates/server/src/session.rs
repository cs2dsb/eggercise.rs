use axum::{async_trait, extract::FromRequestParts, http::{request::Parts, StatusCode}};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use webauthn_rs::prelude::{PasskeyRegistration, Uuid};


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

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct SessionData {
    passkey_registration_state: Option<PasskeyRegistrationState>,
}

#[derive(Debug, Clone)]
pub struct SessionValue {
    session: Session,
    data: SessionData,
}

impl SessionValue {
    const SESSION_DATA_KEY: &'static str = "session.data";

    pub async fn take_passkey_registration_state(&mut self) -> Result<Option<PasskeyRegistrationState>, anyhow::Error> {
        let reg = self.data.passkey_registration_state.take();
        Self::update_session(&self.session, &self.data).await?;
        Ok(reg)
    }

    pub async fn set_passkey_registration_state(&mut self, passkey_registration: PasskeyRegistrationState) -> Result<(), anyhow::Error> {
        self.data.passkey_registration_state = Some(passkey_registration);
        Self::update_session(&self.session, &self.data).await?;
        Ok(())
    }

    async fn update_session(session: &Session, data: &SessionData) -> Result<(), anyhow::Error> {
        session
            .insert(Self::SESSION_DATA_KEY, data.clone())
            .await?;
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