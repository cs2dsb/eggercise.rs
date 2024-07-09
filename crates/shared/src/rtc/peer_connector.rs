#![allow(unused)]
use super::{signalling_client, Builder, Error, PeerId, RoomId};
use crate::api::Object;

pub trait Connector<R: RoomId, P: PeerId, C: signalling_client::Client<R, P>>: Sized {
    type Builder: Builder<Self>;
    /// Create a new default builder
    fn new() -> Self::Builder;
    /// Create a builder with relevant fields pre-set from a base url
    fn with_base<T: AsRef<str>>(base_url: T) -> Self::Builder;
    fn build_signalling_client(&self) -> Result<C, Error>;
}

#[derive(Debug, Clone)]
pub struct PeerConnector {
    signalling_server: String,
    ice_servers: Vec<String>,
}

impl PeerConnector {
    pub fn new(signalling_server: String, ice_servers: Vec<String>) -> Self {
        Self { signalling_server, ice_servers }
    }

    /// Calls new with signalling and ice servers constructed from one base url
    pub fn with_base<T: AsRef<str>>(base_url: T) -> Self {
        let base = base_url.as_ref();

        Self::new(format!("{}/{}", base, Object::RtcSignalling.path()), vec![format!(
            "{}/{}",
            base,
            Object::RtcStun.path()
        )])
    }
}
