use gloo::net::websocket::WebSocketError;
use wasm_bindgen::JsValue;

use crate::api::error::JsError;

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
