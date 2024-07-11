#![allow(unused)]
use std::marker::PhantomData;

use crate::{
    api::Object,
    rtc::{peer_connector, signalling_client, Builder, Error, PeerId, RoomId, SignallingClient},
};

#[derive(Debug, Clone)]
pub struct PeerConnectorBuilder<R: RoomId, P: PeerId> {
    ice_servers: Vec<String>,
    _r: PhantomData<R>,
    _p: PhantomData<P>,
}

impl<R: RoomId, P: PeerId> Default for PeerConnectorBuilder<R, P> {
    fn default() -> Self {
        Self { ice_servers: Default::default(), _r: PhantomData, _p: PhantomData }
    }
}

impl<R: RoomId, P: PeerId> PeerConnectorBuilder<R, P> {
    pub fn set_ice_servers(mut self, ice_servers: Vec<String>) -> Self {
        self.ice_servers = ice_servers;
        self
    }

    pub fn add_ice_server<T: Into<String>>(mut self, ice_server: T) -> Self {
        self.ice_servers.push(ice_server.into());
        self
    }
}

impl<R: RoomId, P: PeerId> Builder<PeerConnector<R, P>> for PeerConnectorBuilder<R, P> {
    fn build(self) -> Result<PeerConnector<R, P>, Error> {
        let Self { ice_servers, .. } = self;
        Ok(PeerConnector { ice_servers, ..Default::default() })
    }
}

/// Native peer connector
#[derive(Debug, Clone)]
pub struct PeerConnector<R: RoomId, P: PeerId> {
    ice_servers: Vec<String>,
    _r: PhantomData<R>,
    _p: PhantomData<P>,
}

impl<R: RoomId, P: PeerId> Default for PeerConnector<R, P> {
    fn default() -> Self {
        Self { ice_servers: Default::default(), _r: PhantomData, _p: PhantomData }
    }
}

impl<R: RoomId, P: PeerId> peer_connector::Connector<R, P, SignallingClient<R, P>>
    for PeerConnector<R, P>
{
    type Builder = PeerConnectorBuilder<R, P>;

    fn new() -> Self::Builder {
        PeerConnectorBuilder::default()
    }

    fn with_base<T: AsRef<str>>(base_url: T) -> Self::Builder {
        let base = base_url.as_ref();

        Self::new().add_ice_server(format!("{}/{}", base, Object::RtcStun.path()))
    }

    fn build_signalling_client(&self) -> Result<SignallingClient<R, P>, Error> {
        todo!()
    }
}
