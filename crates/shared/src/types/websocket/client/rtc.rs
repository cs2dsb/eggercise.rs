#[cfg(feature = "wasm")]
use gloo::net::websocket::Message as WebSocketMessage;
use serde::{Deserialize, Serialize};

use super::ClientMessage;
use crate::types::{rtc::PeerId, websocket::Offer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRtc {
    /// Offer to connect to a peer
    Offer { offer: Offer, peer: PeerId },
    /// Answer to an offer
    Answer { answer: Offer, peer: PeerId },
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
