use std::{ops::Deref, sync::Arc};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use dashmap::DashMap;
use shared::types::rtc::{PeerId, RoomId};
use tracing::error;

use crate::AppState;

#[derive(Debug, Clone, Default)]
pub struct RtcRoomState(Arc<DashMap<RoomId, Vec<PeerId>>>);

impl RtcRoomState {
    pub fn add(&self, room_id: RoomId, peer_id: PeerId) {
        match self.0.try_entry(room_id.clone()) {
            None => error!("Failed to lock RtcRoomState for key: {room_id:?}"),
            Some(e) => {
                let mut vec = e.or_default();
                // Replace with a Set
                if !vec.contains(&peer_id) {
                    vec.push(peer_id);
                }
            },
        }
    }

    pub fn remove(&self, room_id: RoomId, peer_id: PeerId) {
        match self.0.try_entry(room_id.clone()) {
            None => error!("Failed to lock RtcRoomState for key: {room_id:?}"),
            Some(e) => e.or_default().retain(|v| *v != peer_id),
        }
    }

    pub fn room_peers(&self, room_id: &RoomId, peer_id: &PeerId) -> Vec<PeerId> {
        self.0
            .get(room_id)
            .map(|r| r.value().clone())
            .unwrap_or_default()
            .into_iter()
            .filter(|v| v != peer_id)
            .collect()
    }
}

impl From<DashMap<RoomId, Vec<PeerId>>> for RtcRoomState {
    fn from(value: DashMap<RoomId, Vec<PeerId>>) -> Self {
        RtcRoomState(Arc::new(value))
    }
}

impl Deref for RtcRoomState {
    type Target = DashMap<RoomId, Vec<PeerId>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRef<AppState> for RtcRoomState {
    fn from_ref(state: &AppState) -> Self {
        state.rtc_room_state.clone()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RtcRoomState
where
    S: Send + Sync,
    RtcRoomState: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(RtcRoomState::from_ref(state).into())
    }
}

// #[derive(Debug, Clone)]
// pub struct PeerConnectorState(Arc<PeerConnector<RoomId, PeerId>>);

// impl From<PeerConnector<RoomId, PeerId>> for PeerConnectorState {
//     fn from(value: PeerConnector<RoomId, PeerId>) -> Self {
//         PeerConnectorState(Arc::new(value))
//     }
// }

// impl Deref for PeerConnectorState {
//     type Target = PeerConnector<RoomId, PeerId>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl FromRef<AppState> for PeerConnectorState {
//     fn from_ref(state: &AppState) -> Self {
//         state.rtc_connector.clone()
//     }
// }

// #[async_trait]
// impl<S> FromRequestParts<S> for PeerConnectorState
// where
//     S: Send + Sync,
//     PeerConnectorState: FromRef<S>,
// {
//     type Rejection = (StatusCode, String);

//     async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         Ok(PeerConnectorState::from_ref(state).into())
//     }
// }

// #[derive(Debug, Clone, Default)]
// pub struct PeerMapState(Arc<PeerMap>);

// impl From<PeerMap> for PeerMapState {
//     fn from(value: PeerMap) -> Self {
//         PeerMapState(Arc::new(value))
//     }
// }

// impl Deref for PeerMapState {
//     type Target = PeerMap;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl FromRef<AppState> for PeerMapState {
//     fn from_ref(state: &AppState) -> Self {
//         state.rtc_peers.clone()
//     }
// }

// #[async_trait]
// impl<S> FromRequestParts<S> for PeerMapState
// where
//     S: Send + Sync,
//     PeerMapState: FromRef<S>,
// {
//     type Rejection = (StatusCode, String);

//     async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         Ok(PeerMapState::from_ref(state).into())
//     }
// }

// #[derive(Debug, Clone)]
// pub struct SignallingClientState(Arc<SignallingClient<RoomId, PeerId>>);

// impl From<SignallingClient<RoomId, PeerId>> for SignallingClientState {
//     fn from(value: SignallingClient<RoomId, PeerId>) -> Self {
//         SignallingClientState(Arc::new(value))
//     }
// }

// impl Deref for SignallingClientState {
//     type Target = SignallingClient<RoomId, PeerId>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl FromRef<AppState> for SignallingClientState {
//     fn from_ref(state: &AppState) -> Self {
//         state.rtc_signalling_client.clone()
//     }
// }

// #[async_trait]
// impl<S> FromRequestParts<S> for SignallingClientState
// where
//     S: Send + Sync,
//     SignallingClientState: FromRef<S>,
// {
//     type Rejection = (StatusCode, String);

//     async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         Ok(SignallingClientState::from_ref(state).into())
//     }
// }
