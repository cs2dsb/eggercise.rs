#[cfg(feature = "wasm")]
use gloo::net::websocket::Message as WebSocketMessage;
use gloo::net::websocket::WebSocketError;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{
    api::error::JsError,
    types::rtc::{PeerId, RoomId},
};

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("SocketClosed: {clean_exit:?}")]
    SocketClosed { clean_exit: bool },
    #[error("de/serialize error: {0:?}")]
    Json(serde_json::Error),
    #[error("{0:?}")]
    Js(String),
    #[error("{0:?}")]
    Other(String),
}

impl From<serde_json::Error> for MessageError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<JsValue> for MessageError {
    fn from(value: JsValue) -> Self {
        let js_error = JsError::from(value);
        Self::Js(js_error.to_string())
    }
}

#[cfg(feature = "wasm")]
impl From<WebSocketError> for MessageError {
    fn from(value: WebSocketError) -> Self {
        Self::Other(format!("WebSocket: {value:?}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Rtc related messages
    Rtc(ServerRtc),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerRtc {
    /// Server allocated user id
    PeerId(PeerId),
}

impl From<ServerRtc> for ServerMessage {
    fn from(value: ServerRtc) -> Self {
        Self::Rtc(value)
    }
}

impl From<PeerId> for ServerMessage {
    fn from(value: PeerId) -> Self {
        Self::Rtc(ServerRtc::PeerId(value))
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Keepalive message to prevent the websocket being dropped (common for
    /// reverse proxies)
    Keepalive,
    /// Rtc related messages
    Rtc(ClientRtc),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRtc {
    /// Request to join a room
    Join(RoomId),
}

impl From<RoomId> for ClientMessage {
    fn from(value: RoomId) -> Self {
        Self::Rtc(ClientRtc::Join(value))
    }
}

#[cfg(feature = "wasm")]
impl TryFrom<ClientMessage> for WebSocketMessage {
    type Error = MessageError;

    fn try_from(message: ClientMessage) -> Result<WebSocketMessage, Self::Error> {
        let payload = serde_json::to_vec(&message)?;
        Ok(WebSocketMessage::Bytes(payload))
    }
}
