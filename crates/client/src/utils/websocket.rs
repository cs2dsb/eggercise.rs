use std::any::type_name;

use leptos::{leptos_dom::helpers::location, provide_context, use_context};
use shared::api::{
    error::{FrontendError, Nothing, ResultContext},
    Object,
};
use tracing::debug;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{MessageEvent, WebSocket};

#[derive(Clone)]
pub struct Websocket {
    #[allow(unused)]
    socket: WebSocket,
}

impl Websocket {
    fn new() -> Result<Websocket, FrontendError<Nothing>> {
        assert!(
            use_context::<Self>().is_none(),
            "Call to Websocket::new() when there's already one in the context"
        );

        let path = Object::Websocket.path();

        let loc = location();
        let host = loc
            .host()
            // TODO: can we get rid of this manual map by implementing ResultContext/ErrorContext
            // for Into<FEE>
            .map_err(FrontendError::from)
            .context("location.host")?;
        let proto = loc
            .protocol()
            .map_err(FrontendError::from)
            .context("location.protocol")?;

        let ws_proto = match proto.as_ref() {
            "http:" => "ws",
            "https:" => "wss",
            other => panic!("Unsupported protocol: {other}"),
        };

        let url = format!("{ws_proto}://{host}{path}");

        let socket = WebSocket::new(&url)?;

        let callback =
            Closure::wrap(Box::new(move |event| Self::on_message(event)) as Box<dyn FnMut(_)>);

        // Set the callback
        socket.set_onmessage(Some(callback.as_ref().unchecked_ref()));

        // Prevent it from being dropped
        callback.forget();

        Ok(Self {
            socket,
        })
    }

    fn on_message(event: MessageEvent) {
        debug!("Got websocket event: {:?}", event.data());
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
}
