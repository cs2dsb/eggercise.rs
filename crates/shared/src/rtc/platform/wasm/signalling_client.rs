use std::{marker::PhantomData, time::Duration};

use crate::{
    api::Object,
    rtc::{
        peer_connector,
        signalling_client::{self, Client as _},
        Builder, Error, PeerId, RoomId,
    },
};

pub const DEFAULT_SIGNALLING_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct SignallingClientBuilder<R: RoomId, P: PeerId> {
    url: Option<String>,
    timeout: Duration,
    _r: PhantomData<R>,
    _p: PhantomData<P>,
}

impl<R: RoomId, P: PeerId> Default for SignallingClientBuilder<R, P> {
    fn default() -> Self {
        Self {
            url: Default::default(),
            timeout: DEFAULT_SIGNALLING_TIMEOUT,
            _r: PhantomData,
            _p: PhantomData,
        }
    }
}

impl<R: RoomId, P: PeerId> SignallingClientBuilder<R, P> {
    pub fn set_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl<R: RoomId, P: PeerId> Builder<SignallingClient<R, P>> for SignallingClientBuilder<R, P> {
    fn build(self) -> Result<SignallingClient<R, P>, Error> {
        let SignallingClientBuilder {
            url,
            timeout,
            ..
        } = self;

        let url = url.ok_or(Error::Builder {
            message: "url not set",
        })?;
        Ok(SignallingClient {
            url,
            timeout,
            ..Default::default()
        })
    }
}

/// Wasm signalling client
#[derive(Debug)]
pub struct SignallingClient<R: RoomId, P: PeerId> {
    url: String,
    timeout: Duration,
    _r: PhantomData<R>,
    _p: PhantomData<P>,
}

impl<R: RoomId, P: PeerId> Default for SignallingClient<R, P> {
    fn default() -> Self {
        Self {
            url: Default::default(),
            timeout: DEFAULT_SIGNALLING_TIMEOUT,
            _r: PhantomData,
            _p: PhantomData,
        }
    }
}

impl<R: RoomId, P: PeerId> signalling_client::Client<R, P> for SignallingClient<R, P> {
    type Builder = SignallingClientBuilder<R, P>;
    fn new() -> Self::Builder {
        Default::default()
    }
}
