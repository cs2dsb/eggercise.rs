use std::{ops::Deref, sync::Arc};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use shared::{
    rtc::{PeerConnector, PeerMap, SignallingClient},
    types::rtc::{PeerId, RoomId},
};

use crate::AppState;

#[derive(Debug, Clone)]
pub struct PeerConnectorState(Arc<PeerConnector<RoomId, PeerId>>);

impl From<PeerConnector<RoomId, PeerId>> for PeerConnectorState {
    fn from(value: PeerConnector<RoomId, PeerId>) -> Self {
        PeerConnectorState(Arc::new(value))
    }
}

impl Deref for PeerConnectorState {
    type Target = PeerConnector<RoomId, PeerId>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRef<AppState> for PeerConnectorState {
    fn from_ref(state: &AppState) -> Self {
        state.rtc_connector.clone()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for PeerConnectorState
where
    S: Send + Sync,
    PeerConnectorState: FromRef<S>,
{
    type Rejection = (StatusCode, String);
    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(PeerConnectorState::from_ref(state).into())
    }
}

#[derive(Debug, Clone, Default)]
pub struct PeerMapState(Arc<PeerMap>);

impl From<PeerMap> for PeerMapState {
    fn from(value: PeerMap) -> Self {
        PeerMapState(Arc::new(value))
    }
}

impl Deref for PeerMapState {
    type Target = PeerMap;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRef<AppState> for PeerMapState {
    fn from_ref(state: &AppState) -> Self {
        state.rtc_peers.clone()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for PeerMapState
where
    S: Send + Sync,
    PeerMapState: FromRef<S>,
{
    type Rejection = (StatusCode, String);
    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(PeerMapState::from_ref(state).into())
    }
}

#[derive(Debug, Clone)]
pub struct SignallingClientState(Arc<SignallingClient<RoomId, PeerId>>);

impl From<SignallingClient<RoomId, PeerId>> for SignallingClientState {
    fn from(value: SignallingClient<RoomId, PeerId>) -> Self {
        SignallingClientState(Arc::new(value))
    }
}

impl Deref for SignallingClientState {
    type Target = SignallingClient<RoomId, PeerId>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRef<AppState> for SignallingClientState {
    fn from_ref(state: &AppState) -> Self {
        state.rtc_signalling_client.clone()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SignallingClientState
where
    S: Send + Sync,
    SignallingClientState: FromRef<S>,
{
    type Rejection = (StatusCode, String);
    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(SignallingClientState::from_ref(state).into())
    }
}
