#[cfg(feature = "wasm")]
use gloo::net::websocket::Message as WebSocketMessage;
use serde::{Deserialize, Serialize};

use super::ClientMessage;
use crate::types::{
    rtc::PeerId,
    websocket::{IceCandidate, Sdp},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRtc {
    /// Peer informing server of it's id
    Announce { peer_id: PeerId },
    /// Offer or answer to connect to a peer
    Sdp { sdp: Sdp, peer: PeerId },
    /// An ice candidate
    IceCandidate { candidate: IceCandidate, peer: PeerId },
}

impl From<ClientRtc> for ClientMessage {
    fn from(value: ClientRtc) -> Self {
        Self::Rtc(value)
    }
}

#[cfg(feature = "wasm")]
impl TryFrom<ClientRtc> for WebSocketMessage {
    type Error = <WebSocketMessage as TryFrom<ClientMessage>>::Error;

    fn try_from(message: ClientRtc) -> Result<WebSocketMessage, Self::Error> {
        let message: ClientMessage = message.into();
        message.try_into()
    }
}
