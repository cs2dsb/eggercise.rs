use std::{any::type_name, fmt::Display, time::Duration};

use futures::{channel::mpsc::SendError, SinkExt as _, StreamExt};
use leptos::{
    provide_context, spawn_local, use_context, Memo, RwSignal, Signal, SignalUpdate, SignalWith,
};
use reconnecting_websocket::{Event, SocketBuilder, SocketSink, State};
use shared::{
    api::{
        error::{FrontendError, Nothing},
        Object,
    },
    types::websocket::{ClientMessage, ServerMessage},
};
use tracing::{error, info, trace};

use crate::utils::location::{host, protocol};

// Can't be 0
const BACKOFF_MIN: Duration = Duration::from_millis(100);
// Can't be greater than u32::MAX millis
const BACKOFF_MAX: Option<Duration> = Some(Duration::from_secs(60));

pub type MessageResult = Result<ServerMessage, FrontendError<Nothing>>;

#[derive(Clone)]
pub struct Websocket {
    status_memo: Memo<State>,
    // TODO: This is probably only useful for debugging. Returning a stream for further processing
    // might make more sense
    message_signal: RwSignal<Option<MessageResult>>,
    sender: SocketSink<ClientMessage>,
}

impl Websocket {
    fn new() -> Result<Self, FrontendError<Nothing>> {
        assert!(
            use_context::<Self>().is_none(),
            "Call to Websocket::new() when there's already one in the context"
        );

        let message_signal: RwSignal<Option<MessageResult>> = Default::default();
        let state_signal: RwSignal<State> = RwSignal::new(State::Connecting);
        let status_memo: Memo<State> = Memo::new_owning(move |old| {
            state_signal.with(move |new| match (old, new) {
                (None, new) => (*new, true),
                (Some(old), new) => (*new, old == *new),
            })
        });

        let (mut socket, sender) = {
            let socket = SocketBuilder::new(url()?)
                .set_backoff_min(BACKOFF_MIN)
                .set_backoff_max(BACKOFF_MAX)
                .open()?;
            let sender = socket.get_sink();
            (socket, sender)
        };

        spawn_local(async move {
            loop {
                if let Some(event) = socket.next().await {
                    use Event::*;
                    match event {
                        State(s) => {
                            trace!("Websocket state changed: {s:?}");
                            state_signal.update(|v| *v = s);
                        },
                        Message(m) => match m {
                            Ok(m) => {
                                trace!("Websocket message: {m:?}");
                                message_signal.update(|v| *v = Some(Ok(m)));
                            },
                            Err(e) => {
                                error!("Websocket error: {e:?}");
                                message_signal.update(|v| *v = Some(Err(e.into())));
                            },
                        },
                    }
                } else {
                    info!("Reconnecting websocket closed");
                    break;
                }
            }
        });

        let handle = Self { status_memo, message_signal: message_signal.clone(), sender };

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

    pub fn message_signal(&self) -> Signal<Option<MessageResult>> {
        self.message_signal.into()
    }

    pub async fn send(&mut self) -> Result<(), SendError> {
        self.sender.send(ClientMessage::Keepalive).await
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
