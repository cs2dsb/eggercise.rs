use std::{any::type_name, fmt::Display};

use futures::{
    channel::mpsc::{self, SendError, UnboundedSender},
    select, SinkExt as _, StreamExt,
};
use gloo::net::websocket::{futures::WebSocket, Message, State, WebSocketError};
use leptos::{
    provide_context, spawn_local, use_context, Memo, RwSignal, Signal, SignalUpdate, SignalWith,
};
use shared::{
    api::{
        error::{FrontendError, Nothing},
        Object,
    },
    types::websocket::{ClientMessage, MessageError, ServerMessage},
};
use tracing::{debug, error, warn};

use crate::utils::location::{host, protocol};

fn compare_state(a: State, b: State) -> bool {
    use State::*;
    match (a, b) {
        (Connecting, Connecting) | (Open, Open) | (Closing, Closing) | (Closed, Closed) => true,
        _ => false,
    }
}

#[derive(Clone)]
pub struct Websocket {
    status_memo: Memo<State>,
    message_signal: RwSignal<Option<Result<ServerMessage, MessageError>>>,
    message_sender: UnboundedSender<ClientMessage>,
}

impl Websocket {
    fn new() -> Result<Self, FrontendError<Nothing>> {
        assert!(
            use_context::<Self>().is_none(),
            "Call to Websocket::new() when there's already one in the context"
        );

        let url = url()?;
        let mut socket = WebSocket::open(&url)?;

        let message_signal: RwSignal<Option<Result<ServerMessage, MessageError>>> =
            Default::default();
        let status_signal: RwSignal<State> = RwSignal::new(State::Connecting);
        let status_memo: Memo<State> = Memo::new_owning(move |old| {
            status_signal.with(move |new| match (old, new) {
                (None, new) => (*new, true),
                (Some(old), new) => (*new, compare_state(old, *new)),
            })
        });

        let (message_sender, mut message_receiver) = mpsc::unbounded();

        spawn_local(async move {
            loop {
                let mut fused_socket = (&mut socket).fuse();

                select! {
                    m = message_receiver.next() => match m {
                        None => {
                            debug!("WebSocket message sender stream closed");
                            break;
                        },
                        Some(m) => {
                            debug!("Websocket sending: {m:?}");
                            // TODO:
                            let _ = socket.send(Message::Text("blah".to_string())).await;
                        },
                    },
                    m = fused_socket.next() => match m {
                        None => {
                            debug!("Websocket closed");
                            break;
                        },
                        Some(m) => match m {
                            // TODO: is there anything more useful we can do with the send error?
                            Err(WebSocketError::ConnectionClose(c)) => {
                                if c.was_clean {
                                    warn!("Websocket closed cleanly: {} {}", c.code, c.reason);
                                } else {
                                    error!("Websocket closed uncleanly: {} {}", c.code, c.reason);
                                }
                            },
                            Err(e) => error!("Websocket error: {e:?}"),
                            Ok(m) => match ServerMessage::try_from(&m) {
                                Err(e) => {
                                    error!("Websocket MessageError: {e:?}");
                                    message_signal.update(|v| *v = Some(Err(e)));
                                },
                                Ok(m) => {
                                    debug!("WebSocket ServerMessage: {m:?}");
                                    message_signal.update(|v| *v = Some(Ok(m)));
                                },
                            },
                        },
                    },
                };

                let state = socket.state();
                match state {
                    State::Closed | State::Closing => break,
                    _ => {},
                }

                status_signal.update(|v| *v = state);
            }

            status_signal.update(|v| *v = State::Closed);
            warn!("Websocket closed");
        });

        Ok(Self { status_memo, message_signal, message_sender })
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

    pub fn status_signal(&self) -> Signal<State> {
        self.status_memo.into()
    }

    pub fn message_signal(&self) -> Signal<Option<Result<ServerMessage, MessageError>>> {
        self.message_signal.into()
    }

    pub async fn send(&mut self) -> Result<(), SendError> {
        self.message_sender.send(ClientMessage::Keepalive).await
    }
}

fn url<T: Display>() -> Result<String, FrontendError<T>> {
    let path = Object::Websocket.path();

    let host = host()?;
    let proto = protocol()?;

    let ws_proto = match proto.as_ref() {
        "http:" => "ws",
        "https:" => "wss",
        other => Err(FrontendError::Other { message: format!("Unsupported protocol: {other}") })?,
    };

    Ok(format!("{ws_proto}://{host}{path}"))
}
