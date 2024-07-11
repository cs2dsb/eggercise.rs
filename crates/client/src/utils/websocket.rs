use std::{any::type_name, fmt::Display, time::Duration};

use exponential_backoff::Backoff;
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
use tracing::{debug, error, info, warn};

use crate::utils::location::{host, protocol};

const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);
// Can't be 0
const BACKOFF_MIN: Duration = Duration::from_millis(100);
// Can't be greater than u32::MAX millis
const BACKOFF_MAX: Option<Duration> = Some(Duration::from_secs(60));
const BACKOFF_RETRIES: u32 = u32::MAX;

fn compare_state(a: State, b: State) -> bool {
    use State::*;
    match (a, b) {
        (Connecting, Connecting) | (Open, Open) | (Closing, Closing) | (Closed, Closed) => true,
        _ => false,
    }
}

pub type MessageResult = Result<ServerMessage, MessageError>;
pub type ConnectionResult = Result<MessageResult, FrontendError<Nothing>>;

#[derive(Clone)]
pub struct Websocket {
    status_memo: Memo<State>,
    // TODO: This is probably only useful for debugging. Returning a stream for further processing
    // might make more sense
    message_signal: RwSignal<Option<ConnectionResult>>,
    message_sender: UnboundedSender<ClientMessage>,
}

impl Websocket {
    fn new() -> Result<Self, FrontendError<Nothing>> {
        assert!(
            use_context::<Self>().is_none(),
            "Call to Websocket::new() when there's already one in the context"
        );

        let message_signal: RwSignal<Option<ConnectionResult>> = Default::default();
        let status_signal: RwSignal<State> = RwSignal::new(State::Connecting);
        let status_memo: Memo<State> = Memo::new_owning(move |old| {
            status_signal.with(move |new| match (old, new) {
                (None, new) => (*new, true),
                (Some(old), new) => (*new, compare_state(old, *new)),
            })
        });

        let (message_sender, mut message_receiver): (UnboundedSender<ClientMessage>, _) =
            mpsc::unbounded();

        let handle = Self {
            status_memo,
            message_signal: message_signal.clone(),
            message_sender: message_sender.clone(),
        };

        // Done outside the task initially because the only errors this returns are fatal
        // Could also be a panic but this way gives us an opportunity to display the error in leptos
        info!("Websocket connecting...");
        let mut socket = Some(WebSocket::open(&url()?)?);

        spawn_local(async move {
            // Reconnect loop
            let backoff = Backoff::new(BACKOFF_RETRIES, BACKOFF_MIN, BACKOFF_MAX);
            let mut retry = 0_u32;

            loop {
                let message_sender = message_sender.clone();
                let message_receiver = &mut message_receiver;
                let backoff = &backoff;
                let retry = &mut retry;

                let r = async move {
                    let socket = &mut socket;

                    if socket.is_none() {
                        if let Some(timeout) = backoff.next(*retry) {
                            debug!("Backoff retry: {retry}, timeout: {:.3}", timeout.as_secs_f32());
                            TimeoutFuture::new(timeout.as_millis() as u32).await;
                        }
                        *retry += 1;
                        info!("Websocket connecting...");
                        // TODO: add a more aggressive timeout on connect. If the server isn't responding it seems to take quite a while for the gloo socket
                        //       to wake and return None. Also investigate if this is a bug with gloo or how I'm polling it
                        let inner = WebSocket::open(&url()?)?;
                        *socket = Some(inner);
                    }

                    // Unwrap safe because we just did an is_none check
                    let fuse_ref = socket.as_mut().unwrap();
                    let mut fused_socket = fuse_ref.fuse();
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
                                            message_signal.update(|v| *v = Some(Ok(Err(e))));
                                            continue;
                                        },
                                        Ok(v) => v,
                                    };
                                    if let Err(e) = fused_socket.send(payload).await {
                                        error!("socket.send: {e:?}");
                                        message_signal.update(|v| *v = Some(Ok(Err(e.into()))));
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
                                            message_signal.update(|v| *v = Some(Ok(Err(e))));
                                        },
                                        Ok(m) => {
                                            debug!("ServerMessage: {m:?}");
                                            message_signal.update(|v| *v = Some(Ok(Ok(m))));
                                        },
                                    },
                                },
                            },
                            _ = TimeoutFuture::new(KEEPALIVE_INTERVAL.as_millis() as u32).fuse() => {
                                if let Err(e) = message_sender.unbounded_send(ClientMessage::Keepalive) {
                                    // TODO: anything else we should do?
                                    error!("unbounded_send: {e:?}");
                                }
                            },
                        };

                        let state = fused_socket.get_ref().state();
                        match state {
                            State::Closed | State::Closing => break,
                            // Reset the retry count to reset the exponential backoff
                            State::Open => *retry = 0,
                            _ => {},
                        }

                        status_signal.update(|v| *v = state);
                    }

                    status_signal.update(|v| *v = State::Closed);
                    debug!("closed");

                    Ok::<_, FrontendError<Nothing>>(())
                }.await;

                // Drop the old socket
                socket = None;

                if let Err(e) = r {
                    message_signal.update(|v| *v = Some(Err(e)));
                }
            }
        });

        Ok(handle)
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

    pub fn message_signal(&self) -> Signal<Option<ConnectionResult>> {
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
