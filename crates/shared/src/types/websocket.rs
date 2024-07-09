use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::{
    api::error::JsError,
    types::rtc::{PeerId, RoomId},
};

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
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
        let payload = serde_json::to_string(&message)?;
        // TODO: binary would be preferable
        Ok(axum::extract::ws::Message::Text(payload))
    }
}

impl ServerMessage {
    #[cfg(feature = "wasm")]
    pub fn try_from(event: &web_sys::MessageEvent) -> Result<Self, MessageError> {
        // TODO: async requirement is an issue here
        // if let Ok(blob) = web_sys::Blob::try_from(event.data()) {
        // let ab = wasm_bindgen_futures::JsFuture::from(blob.array_buffer()).await?;
        // let data = web_sys::js_sys::Uint8Array::new(&ab).to_vec();
        // Ok(serde_json::from_slice(&data)?)
        // } else
        if let Ok(text) = String::try_from(event.data()) {
            Ok(serde_json::from_str(&text)?)
        } else if let Some(obj) = web_sys::js_sys::Object::try_from(&event.data()) {
            Err(MessageError::Other(format!(
                "Unexpected event.data type: {}. Only Blob and Text are supported",
                obj.to_string()
            )))
        } else {
            Err(MessageError::Other(
                "Unexpected event.data type (not even a JS Object). Only Blob and Text are \
                 supported"
                    .to_string(),
            ))
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
