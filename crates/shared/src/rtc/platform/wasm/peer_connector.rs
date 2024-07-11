use std::{marker::PhantomData, time::Duration};

use crate::{
    api::Object,
    rtc::{
        peer_connector,
        signalling_client::{self, Client as _},
        Builder, Error, PeerId, RoomId, SignallingClient,
    },
};

#[derive(Debug, Clone)]
pub struct PeerConnectorBuilder<R: RoomId, P: PeerId> {
    signalling_server: Option<String>,
    ice_servers: Vec<String>,
    _r: PhantomData<R>,
    _p: PhantomData<P>,
}

impl<R: RoomId, P: PeerId> Default for PeerConnectorBuilder<R, P> {
    fn default() -> Self {
        Self {
            signalling_server: Default::default(),
            ice_servers: Default::default(),
            _r: PhantomData,
            _p: PhantomData,
        }
    }
}

impl<R: RoomId, P: PeerId> PeerConnectorBuilder<R, P> {
    pub fn set_signalling_server(mut self, signalling_server: String) -> Self {
        self.signalling_server = Some(signalling_server);
        self
    }

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
        let Self { signalling_server, ice_servers, .. } = self;

        let signalling_server =
            signalling_server.ok_or(Error::Builder { message: "signalling server must be set" })?;
        if ice_servers.len() == 0 {
            Err(Error::Builder { message: "at least once ice server must be set" })?;
        }

        Ok(PeerConnector { signalling_server, ice_servers, ..Default::default() })
    }
}

/// Wasm peer connector
#[derive(Debug, Clone)]
pub struct PeerConnector<R: RoomId, P: PeerId> {
    signalling_server: String,
    ice_servers: Vec<String>,
    _r: PhantomData<R>,
    _p: PhantomData<P>,
}

impl<R: RoomId, P: PeerId> Default for PeerConnector<R, P> {
    fn default() -> Self {
        Self {
            signalling_server: Default::default(),
            ice_servers: Default::default(),
            _r: PhantomData,
            _p: PhantomData,
        }
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

        Self::new()
            .set_signalling_server(format!("{}/{}", base, Object::RtcSignalling.path()))
            .add_ice_server(format!("{}/{}", base, Object::RtcStun.path()))
    }

    fn build_signalling_client(&self) -> Result<SignallingClient<R, P>, Error> {
        SignallingClient::new().set_url(self.signalling_server.clone()).build()
    }
}
