use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};

use crate::AppState;

#[derive(Debug)]
pub struct Webauthn(Arc<webauthn_rs::Webauthn>);

impl From<Arc<webauthn_rs::Webauthn>> for Webauthn {
    fn from(webauthn: Arc<webauthn_rs::Webauthn>) -> Self {
        Webauthn(webauthn)
    }
}

impl Deref for Webauthn {
    type Target = Arc<webauthn_rs::Webauthn>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Webauthn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Webauthn
where
    S: Send + Sync,
    Arc<webauthn_rs::Webauthn>: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let webauthn = <Arc<webauthn_rs::Webauthn>>::from_ref(state);
        Ok(webauthn.into())
    }
}

impl FromRef<AppState> for Arc<webauthn_rs::Webauthn> {
    fn from_ref(state: &AppState) -> Self {
        state.webauthn.clone()
    }
}
