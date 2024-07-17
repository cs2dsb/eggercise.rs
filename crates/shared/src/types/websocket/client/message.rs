#[cfg(feature = "backend")]
use axum::extract::ws::Message as AxumMessage;
#[cfg(feature = "wasm")]
use gloo::net::websocket::Message as WebSocketMessage;
use serde::{Deserialize, Serialize};

use super::ClientRtc;
use crate::types::websocket::MessageError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Rtc related messages
    Rtc(ClientRtc),
}

#[cfg(feature = "wasm")]
impl TryFrom<ClientMessage> for WebSocketMessage {
    type Error = MessageError;

    fn try_from(message: ClientMessage) -> Result<WebSocketMessage, Self::Error> {
        let payload = serde_json::to_vec(&message)?;
        Ok(WebSocketMessage::Bytes(payload))
    }
}

#[cfg(feature = "backend")]
impl TryFrom<AxumMessage> for ClientMessage {
    type Error = MessageError;

    fn try_from(message: AxumMessage) -> Result<Self, Self::Error> {
        use AxumMessage::*;
        match message {
            Text(text) => Ok(serde_json::from_str(&text)?),
            Binary(bytes) => Ok(serde_json::from_slice(&bytes)?),
            other => Err(MessageError::Other(format!(
                "Unexpected message type {other:?} for ClientMessage"
            ))),
        }
    }
}
