use serde::{Deserialize, Serialize};

use super::ServerMessage;
use crate::types::{
    rtc::PeerId,
    websocket::{IceCandidate, Sdp},
};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerRtc {
    /// Assign the peer an easy to read name mainly for making debug logs easier to read
    Petname(String),
    /// Room peers
    RoomPeers(Vec<PeerId>),
    /// An offer or answer relayed from a peer
    PeerSdp { sdp: Sdp, peer: PeerId },
    /// An ice candidate
    IceCandidate { candidate: IceCandidate, peer: PeerId },
}

impl From<ServerRtc> for ServerMessage {
    fn from(value: ServerRtc) -> Self {
        Self::Rtc(value)
    }
}
