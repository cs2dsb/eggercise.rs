use std::{any::type_name, fmt::Display, time::Duration};

use futures::{
    channel::mpsc::{self, SendError, UnboundedSender},
    select, FutureExt as _, SinkExt as _, StreamExt,
};
use gloo::{
    net::websocket::{futures::WebSocket, State, WebSocketError},
    timers::future::TimeoutFuture,
};
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

const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

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
        let socket = WebSocket::open(&url)?;

        let message_signal: RwSignal<Option<Result<ServerMessage, MessageError>>> =
            Default::default();
        let status_signal: RwSignal<State> = RwSignal::new(State::Connecting);
        let status_memo: Memo<State> = Memo::new_owning(move |old| {
            status_signal.with(move |new| match (old, new) {
                (None, new) => (*new, true),
                (Some(old), new) => (*new, compare_state(old, *new)),
            })
        });

        let (message_sender, mut message_receiver): (UnboundedSender<ClientMessage>, _) =
            mpsc::unbounded();

        let message_sender_ = message_sender.clone();
        spawn_local(async move {
            let mut fused_socket = socket.fuse();
            loop {
                select! {
                    m = message_receiver.next() => match m {
                        None => {
                            debug!("message sender stream closed");
                            break;
                        },
                        Some(m) => {
                            debug!("sending: {m:?}");
                            let payload = match m.try_into() {
                                Err(e) => {
                                    error!("ClientMessage to WSMessage: {e:?}");
                                    message_signal.update(|v| *v = Some(Err(e)));
                                    continue;
                                },
                                Ok(v) => v,
                            };
                            if let Err(e) = fused_socket.send(payload).await {
                                error!("socket.send: {e:?}");
                                message_signal.update(|v| *v = Some(Err(e.into())));
                            }
                        },
                    },
                    m = fused_socket.next() => match m {
                        None => {
                            debug!("next==none, closed");
                            break;
                        },
                        Some(m) => match m {
                            // TODO: is there anything more useful we can do with the send error?
                            Err(WebSocketError::ConnectionClose(c)) => {
                                if c.was_clean {
                                    warn!("closed cleanly: {} {}", c.code, c.reason);
                                } else {
                                    error!("closed uncleanly: {} {}", c.code, c.reason);
                                }
                            },
                            Err(e) => error!("Error: {e:?}"),
                            Ok(m) => match ServerMessage::try_from(&m) {
                                Err(e) => {
                                    error!("MessageError: {e:?}");
                                    message_signal.update(|v| *v = Some(Err(e)));
                                },
                                Ok(m) => {
                                    debug!("ServerMessage: {m:?}");
                                    message_signal.update(|v| *v = Some(Ok(m)));
                                },
                            },
                        },
                    },
                    _ = TimeoutFuture::new(KEEPALIVE_INTERVAL.as_millis() as u32).fuse() => {
                        if let Err(e) = message_sender_.unbounded_send(ClientMessage::Keepalive) {
                            // TODO: anything else we should do?
                            error!("unbounded_send: {e:?}");
                        }
                    },
                };

                let state = fused_socket.get_ref().state();
                match state {
                    State::Closed | State::Closing => break,
                    _ => {},
                }

                status_signal.update(|v| *v = state);
            }

            status_signal.update(|v| *v = State::Closed);
            debug!("closed");
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
