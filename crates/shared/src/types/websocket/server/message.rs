#[cfg(feature = "wasm")]
use gloo::net::websocket::Message as WebSocketMessage;
use serde::{Deserialize, Serialize};

use super::{ServerRtc, ServerUser};
use crate::types::websocket::MessageError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerMessage {
    /// Rtc related messages
    Rtc(ServerRtc),
    /// User related messages
    User(ServerUser),
}

#[cfg(feature = "backend")]
impl TryFrom<ServerMessage> for axum::extract::ws::Message {
    type Error = MessageError;

    fn try_from(message: ServerMessage) -> Result<Self, Self::Error> {
        let payload = serde_json::to_vec(&message)?;
        Ok(axum::extract::ws::Message::Binary(payload))
    }
}

#[cfg(feature = "wasm")]
impl TryFrom<WebSocketMessage> for ServerMessage {
    type Error = MessageError;

    fn try_from(message: WebSocketMessage) -> Result<Self, Self::Error> {
        match message {
            WebSocketMessage::Text(text) => Ok(serde_json::from_str(&text)?),
            WebSocketMessage::Bytes(bytes) => Ok(serde_json::from_slice(&bytes)?),
        }
    }
}
