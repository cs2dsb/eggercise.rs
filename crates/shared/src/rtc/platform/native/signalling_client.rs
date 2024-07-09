#![allow(unused)]
use std::marker::PhantomData;

use crate::{
    api::Object,
    rtc::{peer_connector, signalling_client, Builder, Error, PeerId, RoomId},
};

#[derive(Debug)]
pub struct SignallingClientBuilder<R: RoomId, P: PeerId> {
    _r: PhantomData<R>,
    _p: PhantomData<P>,
}

impl<R: RoomId, P: PeerId> Default for SignallingClientBuilder<R, P> {
    fn default() -> Self {
        Self {
            _r: PhantomData,
            _p: PhantomData,
        }
    }
}

impl<R: RoomId, P: PeerId> Builder<SignallingClient<R, P>> for SignallingClientBuilder<R, P> {
    fn build(self) -> Result<SignallingClient<R, P>, Error> {
        Ok(Default::default())
    }
}

/// Native signalling client
#[derive(Debug)]
pub struct SignallingClient<R: RoomId, P: PeerId> {
    _r: PhantomData<R>,
    _p: PhantomData<P>,
}

impl<R: RoomId, P: PeerId> Default for SignallingClient<R, P> {
    fn default() -> Self {
        Self {
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
