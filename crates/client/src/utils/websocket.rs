#![allow(unused)]
use std::{
    any::type_name,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{channel::mpsc, stream::Stream};
use gloo::events::EventListener;
use leptos::{provide_context, use_context, ReadSignal, RwSignal, SignalUpdate};
use shared::{
    api::{
        error::{FrontendError, Nothing},
        Object,
    },
    types::websocket::{MessageError, ServerMessage},
};
use tracing::{debug, error, info};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{MessageEvent, WebSocket};

use crate::utils::{
    location::{host, protocol},
    wrap_callback,
};

#[derive(Debug, Clone, Copy)]
pub enum SocketStatus {
    Connecting,
    Open,
    Closing,
    Closed,
    Error,
}

impl Default for SocketStatus {
    fn default() -> Self {
        SocketStatus::Connecting
    }
}

#[derive(Clone)]
pub struct Websocket {
    status_signal: RwSignal<SocketStatus>,
    message_signal: RwSignal<Option<Result<ServerMessage, MessageError>>>,
    inner: Arc<WebSocketInner>,
}

struct WebSocketInner {
    socket: WebSocket,
    // Only retained so they get dropped correctly if this struct is dropped
    _listeners: [EventListener; 4],
}

impl Websocket {
    fn new() -> Result<Self, FrontendError<Nothing>> {
        assert!(
            use_context::<Self>().is_none(),
            "Call to Websocket::new() when there's already one in the context"
        );

        let path = Object::Websocket.path();

        let host = host()?;
        let proto = protocol()?;

        let ws_proto = match proto.as_ref() {
            "http:" => "ws",
            "https:" => "wss",
            other => Err(FrontendError::Other {
                message: format!("Unsupported protocol: {other}"),
            })?,
        };

        let url = format!("{ws_proto}://{host}{path}");

        let socket = WebSocket::new(&url)?;

        let status_signal: RwSignal<SocketStatus> = Default::default();
        let message_signal: RwSignal<Option<Result<ServerMessage, MessageError>>> =
            Default::default();

        let message_listener = EventListener::new(&socket, "message", move |event| {
            let event = event
                .dyn_ref::<MessageEvent>()
                .expect("on message Event should be MessageEvent");
            message_signal.update(|v| *v = Some(ServerMessage::try_from(event)))
        });

        let open_listener = EventListener::new(&socket, "open", move |_| {
            status_signal.update(|v| *v = SocketStatus::Open)
        });

        let close_listener = EventListener::new(&socket, "close", move |_| {
            status_signal.update(|v| *v = SocketStatus::Closed)
        });

        let error_listener = EventListener::new(&socket, "error", move |_| {
            status_signal.update(|v| *v = SocketStatus::Error)
        });

        Ok(Self {
            status_signal,
            message_signal,
            inner: Arc::new(WebSocketInner {
                socket,
                _listeners: [
                    message_listener,
                    open_listener,
                    close_listener,
                    error_listener,
                ],
            }),
        })
    }

    pub fn provide_context() -> Result<(), FrontendError<Nothing>> {
        if use_context::<Self>().is_none() {
            let ws = Self::new()?;
            provide_context(ws);
        }
        Ok(())
    }

    pub fn use_websocket() -> Self {
        use_context::<Self>().expect(&format!("{} missing from context", type_name::<Self>()))
    }

    pub fn status_signal(&self) -> ReadSignal<SocketStatus> {
        self.status_signal.read_only()
    }

    pub fn message_signal(&self) -> ReadSignal<Option<Result<ServerMessage, MessageError>>> {
        self.message_signal.read_only()
    }
}
