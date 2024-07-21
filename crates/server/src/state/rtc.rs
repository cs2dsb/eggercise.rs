use std::{ops::Deref, sync::Arc};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use dashmap::DashMap;
use shared::types::{
    rtc::{PeerId, RoomId},
    websocket::RoomPeer,
};
use tracing::error;

use crate::AppState;

#[derive(Debug, Clone, Default)]
pub struct RtcRoomState(Arc<DashMap<RoomId, Vec<RoomPeer>>>);

impl RtcRoomState {
    pub fn add(&self, room_id: RoomId, peer: RoomPeer) {
        match self.0.try_entry(room_id.clone()) {
            None => error!("Failed to lock RtcRoomState for key: {room_id:?}"),
            Some(e) => {
                let mut vec = e.or_default();
                // TODO: Replace with a Set
                if !vec.contains(&peer) {
                    vec.push(peer);
                }
            },
        }
    }

    pub fn remove(&self, room_id: RoomId, peer_id: PeerId) {
        match self.0.try_entry(room_id.clone()) {
            None => error!("Failed to lock RtcRoomState for key: {room_id:?}"),
            Some(e) => e.or_default().retain(|v| v.peer_id != peer_id),
        }
    }

    /// Get the peers in the room, filtering out the provided peer
    pub fn room_peers(&self, room_id: &RoomId, peer_id: &PeerId) -> Vec<RoomPeer> {
        self.0
            .get(room_id)
            .map(|r| r.value().clone())
            .unwrap_or_default()
            .into_iter()
            .filter(|v| &v.peer_id != peer_id)
            .collect()
    }
}

impl From<DashMap<RoomId, Vec<RoomPeer>>> for RtcRoomState {
    fn from(value: DashMap<RoomId, Vec<RoomPeer>>) -> Self {
        RtcRoomState(Arc::new(value))
    }
}

impl Deref for RtcRoomState {
    type Target = DashMap<RoomId, Vec<RoomPeer>>;

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
