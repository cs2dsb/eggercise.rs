use std::{any::type_name, fmt::Display, time::Duration};

use futures::{channel::mpsc, StreamExt};
use leptos::{
    provide_context, spawn_local, store_value, use_context, RwSignal, Signal, SignalUpdate,
    StoredValue,
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

use super::rtc::RtcSource;
use crate::utils::location::{host, protocol};

// Can't be 0
const BACKOFF_MIN: Duration = Duration::from_millis(100);
// Can't be greater than u32::MAX millis
const BACKOFF_MAX: Option<Duration> = Some(Duration::from_secs(60));

pub type MessageResult = Result<ServerMessage, FrontendError<Nothing>>;

#[derive(Clone)]
pub struct Websocket {
    state_signal: RwSignal<State>,
    sender: SocketSink<ClientMessage>,

    rtc_source: StoredValue<Option<RtcSource>>,

    #[cfg(feature = "debug-signals")]
    message_signal: RwSignal<Vec<MessageResult>>,
}

impl Websocket {
    fn new() -> Result<Self, FrontendError<Nothing>> {
        assert!(
            use_context::<Self>().is_none(),
            "Call to Websocket::new() when there's already one in the context"
        );

        let state_signal: RwSignal<State> = RwSignal::new(State::Connecting);

        let (mut socket, sender) = {
            let socket = SocketBuilder::new(url()?)
                .set_backoff_min(BACKOFF_MIN)
                .set_backoff_max(BACKOFF_MAX)
                .open()?;
            let sender = socket.get_sink();
            (socket, sender)
        };

        #[cfg(feature = "debug-signals")]
        let message_signal: RwSignal<Vec<MessageResult>> = Default::default();

        let (rtc_sender, rtc_receiver) = mpsc::unbounded();
        let rtc_source = store_value(Some(rtc_receiver.into()));

        spawn_local(async move {
            loop {
                if let Some(event) = socket.next().await {
                    use Event::*;
                    match event {
                        State(s) => {
                            trace!("Websocket state changed: {s:?}");
                            state_signal.update(|v| *v = s);
                        },
                        Message(m) => {
                            let m = m.map_err(FrontendError::from);

                            #[cfg(feature = "debug-signals")]
                            {
                                let m = m.clone();
                                message_signal.update(|v| {
                                    v.push(m);
                                });
                            }

                            match m {
                                Ok(m) => {
                                    trace!("Websocket message: {m:?}");
                                    match m {
                                        ServerMessage::Rtc(r) => {
                                            if let Err(e) = rtc_sender.unbounded_send(r) {
                                                error!("rtc_sender.unbounded_send err: {e:?}");
                                            }
                                        },
                                        ServerMessage::User(_uu) => {},
                                    }
                                },
                                Err(e) => {
                                    error!("Websocket error: {e:?}");
                                },
                            };
                        },
                    }
                } else {
                    info!("Reconnecting websocket closed");
                    break;
                }
            }
        });

        let handle = Self {
            state_signal,
            sender,
            rtc_source,

            #[cfg(feature = "debug-signals")]
            message_signal,
        };

        Ok(handle)
    }

    pub fn provide_context() -> Result<(), FrontendError<Nothing>> {
        if use_context::<Self>().is_none() {
            let ws = Self::new()?;
            provide_context(ws);
        }
        Ok(())
    }

    pub fn take_rtc_source() -> Option<RtcSource> {
        let sv = Self::use_websocket().rtc_source;
        sv.try_update_value(|v| v.take()).flatten()
    }

    pub fn use_websocket() -> Self {
        use_context::<Self>().expect(&format!("{} missing from context", type_name::<Self>()))
    }

    pub fn status_signal(&self) -> Signal<State> {
        self.state_signal.into()
    }

    #[cfg(feature = "debug-signals")]
    pub fn message_signal(&self) -> Signal<Vec<MessageResult>> {
        self.message_signal.into()
    }

    pub fn get_sender() -> SocketSink<ClientMessage> {
        Self::use_websocket().sender.clone()
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
